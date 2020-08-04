use isahc::prelude::*;
use regex::Regex;
use std::fs;
use tokio::time;
use tokio_postgres::{Client, NoTls};

async fn connect() -> Client {
    let conf = fs::read_to_string("configuration.toml")
        .expect("Failed to read configuration from configuration.toml");
    let (client, connection) = tokio_postgres::connect(&conf, NoTls)
        .await
        .expect("Can't connect to db");
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });
    client
}

async fn gather_cities(db_client: &Client) -> Vec<(i32, String)> {
    db_client
        .query("select * from city", &[])
        .await
        .expect("Failed to gather cities from db")
        .into_iter()
        .map(|r| (r.get(0), r.get(1)))
        .collect()
}

async fn insert_data(db_client: &Client, presence: f32, id_city: i32) {
    if let Err(e) = db_client
        .query(
            "insert into presence (time, presence, id_city) values (now(), $1, $2);",
            &[&presence, &id_city],
        )
        .await
    {
        println!("Failed to insert in db, error: {:?}", e);
    }
}

async fn gather_data(url: &str, cities: &[(i32, String)], db_client: &Client) {
    let mut data = match isahc::get(url) {
        Ok(d) => d,
        Err(e) => {
            println!("Failed to gather data, error: {:?}", e);
            return;
        }
    };
    let data = data.text().expect("Can't convert data to text");
    let data = data.as_bytes();
    let text = html2text::from_read(data, 80);
    for (id_city, city) in cities {
        let r = format!(r"(?s){}.*?\n(?P<presence>\d+,*\d*)%\n", city);
        let re: Regex = Regex::new(&r).expect("Failed to parse Regex");
        match re.captures(&text) {
            Some(c) => match c.name("presence") {
                Some(p) => {
                    let p = p.as_str();
                    let p: f32 = p
                        .replace(",", ".")
                        .parse()
                        .unwrap_or_else(|_| panic!("Presence is not a float {:?}", p));
                    insert_data(db_client, p, *id_city).await;
                }
                None => {
                    println!("No presence for city {:?}", city);
                }
            },
            None => {
                println!("{:?} not found", city);
            }
        };
    }
}

#[tokio::main]
async fn main() {
    let db_client = connect().await;
    let url = "https://srself.mcfit.com/disponibilita/disponibilita.aspx";
    let cities = gather_cities(&db_client).await;

    let mut interval = time::interval(time::Duration::from_secs(60 * 10));
    loop {
        interval.tick().await;
        gather_data(&url, &cities, &db_client).await;
    }
}
