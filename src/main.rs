use tokio::sync::{Mutex, Semaphore};
use std::sync::Arc;
use reqwest::Error;
use scraper::{Html, Selector};

use async_crawler::url_manager::UrlManager;

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


#[tokio::main]
async fn main() {
let url_manager = Arc::new(Mutex::new(UrlManager::new()));
    

    let mut handles = vec![];

    for _ in 0..10 {
        let manager_clone = url_manager.clone();

        let handle = tokio::spawn(async move {
            loop {

                let url = {
                    let mut manager = manager_clone.lock().await;
                    manager.get_next_url()
                };

                match url {
                    Some(url) => {
                        match fetch_url(&url).await {
                            Ok(body) => {
                                let links = extract_links(&body);
                                let mut manager = manager_clone.lock().await;
                                manager.add_url(links);
                            }
                            Err(e) => {
                                eprintln!("Failed to fetch {}: {}", url, e);
                            }
                        }
                    },
                    None => break,
                }
            }

        });


        handles.push(handle);
    }


    for handle in handles {
        let _ = handle.await;
    }
}

#[cfg(test)]
mod test {
    // TODO: Add tests
}