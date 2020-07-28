use curl::easy::Easy;
use regex::Regex;
use tokio::time;

async fn gather_data<'a>(url: &str) {
    let mut easy = Easy::new();
    easy.url(url).unwrap();
    easy.write_function(move |data| {
        let text = html2text::from_read(data, 80);
        let re: Regex = Regex::new(r"(?s)Brescia.*(?P<percentage>\d+,\d*)%").unwrap();
        if let Some(c) = re.captures(&text) {
            println!("{:?}", c.name("percentage").unwrap().as_str());
        }
        Ok(data.len())
    })
    .unwrap();
    easy.perform().unwrap();
}

#[tokio::main]
async fn main() {
    let url = "https://srself.mcfit.com/disponibilita/disponibilita.aspx";
    let mut interval = time::interval(time::Duration::from_secs(60*10));
    loop {
        interval.tick().await;
        gather_data(&url).await;
    }
}
