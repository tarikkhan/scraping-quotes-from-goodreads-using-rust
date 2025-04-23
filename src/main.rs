use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::{fs::File, io::Write};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut id: u32 = 0;
    let mut results = Vec::new();
    let number_of_pages = 2; // Define how many pages to go through

    for num in 1..=number_of_pages {
        let url = format!("https://www.goodreads.com/quotes?page={}", num);
        let page_result = match quote_search_single_page(&url, id).await {
            Ok(value) => value,
            Err(e) => {
                let error_msg = format!("Error on page number {} with status: {}", num, e);
                return Err(error_msg.into());
            }
        };
        id = page_result.1;
        results.push(page_result.0);
    }

    let results = results.into_iter().flatten().collect::<Vec<Quote>>();

    let serialized = serde_json::to_string(&results).unwrap();
    let mut filename = File::create("quote_collection.json")?;
    filename.write_all(serialized.as_bytes())?;

    Ok(())
}

async fn quote_search_single_page(
    url: &str,
    mut id: u32,
) -> Result<(Vec<Quote>, u32), Box<dyn std::error::Error>> {
    let mut quote_organized = Vec::new();

    let res = reqwest::get(url).await?;
    if !res.status().is_success() {
        println!("{}", res.status());
        let error_msg = format!("Request failed with status code: {}", res.status());
        return Err(error_msg.into());
    }

    let res = res.text().await?;

    let selector = Selector::parse("div.quoteText")?;
    let author_selector = Selector::parse("span.authorOrTitle")?;

    let document = Html::parse_document(&res);

    let quotes = document.select(&selector);

    let mut collection = Vec::new();

    for item in quotes {
        collection.push(item);
    }

    for item in collection {
        let author = item
            .select(&author_selector)
            .next()
            .unwrap()
            .text()
            .collect::<String>();

        let quote = item
            .text()
            .collect::<String>()
            .split_once("―")
            .unwrap()
            .0
            .to_owned();
        let quote_struct = Quote::new(id, author.trim().to_owned(), quote.trim().to_owned()); //trim() returns &str

        quote_organized.push(quote_struct);
        id = id + 1;
    }

    Ok((quote_organized, id))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Quote {
    quote: String,
    author: String,
    id: u32,
}

impl Quote {
    pub fn new(id: u32, author: String, quote: String) -> Self {
        Quote { quote, author, id }
    }
}
