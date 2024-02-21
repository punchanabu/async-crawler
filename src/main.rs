use std::collections::HashSet;

use reqwest::Error;
use scraper::{Html, Selector};

async fn fetch_url(url: &str) -> Result<String,Error> {
    let response = reqwest::get(url).await?;
    let body = response.text().await?;
    Ok(body)
}


fn extract_links(html: &str) -> Vec<String> {
    let document = Html::parse_document(html);
    let selector = Selector::parse("a").unwrap();
    document.select(&selector)
        .filter_map(|element| {
            let href = element.value().attr("href")?;
            if href.starts_with("http") {
                Some(href.to_string())
            } else {
                None
            }
        })
        .collect()
}


struct UrlManager {
    visited: HashSet<String>,
    to_visit: HashSet<String>
}

impl UrlManager {
    fn new() -> Self {
        UrlManager {
            visited: HashSet::new(),
            to_visit: HashSet::new()
        }
    }

    fn add_url(&mut self, urls: Vec<String>) {
        for url in urls {
            if !self.visited.contains(&url) {
                self.to_visit.insert(url);
            }
        }
    }

    fn get_next_url(&mut self) -> Option<String> {
        self.to_visit.iter().next().cloned().map(|url| {
            self.to_visit.remove(&url);
            self.visited.insert(url.clone());
            url
        })
    }
}


#[tokio::main]
async fn main() {
    let mut url_manager = UrlManager::new();
    let start_url = "https://www.rust-lang.org/";
    url_manager.add_url(vec![start_url.to_string()]);
    
    while let Some(url) = url_manager.get_next_url() {
        match fetch_url(&url).await {
            Ok(html) => {
                let links = extract_links(&html);
                println!("Fetched {} with {} links", url, links.len());
                url_manager.add_url(links);
            },
            Err(e) => {
                println!("Error fetching {}: {}", url, e);
            }
        }
    }
}