use tokio::sync::{Mutex, Semaphore};
use std::sync::Arc;
use reqwest::{Client, Error};
use scraper::{Html, Selector};

use async_crawler::url_manager::UrlManager;

async fn fetch_url(client: &Client, url: &str) -> Result<String,Error> {
    let response = client.get(url).send().await?;
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
    let semaphore = Arc::new(Semaphore::new(10));
    let client = Arc::new(Client::new());
    {
        let mut manager = url_manager.lock().await;
        manager.add_url(vec![String::from("https://www.dek-d.com/tcas/62043/")]);
    }
    let mut handles = vec![];

    for _ in 0..10 {
        let manager_clone = url_manager.clone();
        let semaphore_clone = semaphore.clone();
        let client_clone = client.clone();
        let handle = tokio::spawn(async move {
            loop {
                let permit = semaphore_clone.acquire().await.expect("Failed to acquire semaphore permit");
                let url = {
                    let mut manager = manager_clone.lock().await;
                    manager.get_next_url()
                };

                match url {
                    Some(url) => {
                        match fetch_url(&client_clone, &url).await {
                            Ok(body) => {
                                let links = extract_links(&body);
                                println!("Extracted {} links from {}", links.len(), url);
                                let mut manager = manager_clone.lock().await;
                                manager.add_url(links);
                            }
                            Err(e) => {
                                eprintln!("Failed to fetch {}: {}", url, e);
                            }
                        }
                    },
                    None => {
                        drop(permit);
                        break;
                    }
                }
                drop(permit);
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