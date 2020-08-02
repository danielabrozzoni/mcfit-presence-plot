use curl::easy::Easy;
use regex::Regex;
use std::fs::File;
use std::io::{BufRead, BufReader};
use tokio::time;

fn gather_cities() -> Vec<String> {
    let file = File::open("cities.txt").expect("Failed to open cities.txt");
    let mut res = Vec::new();
    for (index, line) in BufReader::new(file).lines().enumerate() {
        if let Ok(l) = line {
            res.push(l);
        } else {
            panic!("Failed to read line {}: {:?}", index - 1, line);
        }
    }
    res
}

async fn gather_data(url: &str, cities: &Vec<String>) {
    let mut easy = Easy::new();
    easy.url(url).expect("Failed to parse URL");
    // easy.transfer() allows write_function to capture stack-local data
    let mut easy = easy.transfer();

    easy.write_function(move |data| {
        let text = html2text::from_read(data, 80);
        for city in cities {
            let r = format!(r"(?s){}.*?\n(?P<percentage>\d+,*\d*)%\n", city);
            let re: Regex = Regex::new(&r).expect("Failed to parse Regex");
            if let Some(c) = re.captures(&text) {
                if let Some(p) = c.name("percentage") {
                    println!("{} {}", city, p.as_str());
                }
            }
        }
        Ok(data.len())
    })
    .unwrap();
    easy.perform().unwrap();
}

#[tokio::main]
async fn main() {
    let url = "https://srself.mcfit.com/disponibilita/disponibilita.aspx";
    let cities = gather_cities();
    let mut interval = time::interval(time::Duration::from_secs(60 * 10));
    loop {
        interval.tick().await;
        gather_data(&url, &cities).await;
    }
}
