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

    let running_tasks = Arc::new(Mutex::new(0));
    while {
        let manager = url_manager.lock().await;
        !manager.is_empty() || *running_tasks.lock().await > 0
    } {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await; // Prevent tight loop
        if semaphore.available_permits() > 0 {
            let manager_clone = url_manager.clone();
            let semaphore_clone = semaphore.clone();
            let client_clone = client.clone();
            let running_tasks_clone = running_tasks.clone();

            let url_option = {
                let mut manager = manager_clone.lock().await;
                manager.get_next_url()
            };

            if let Some(url) = url_option {
                *running_tasks_clone.lock().await += 1;
                tokio::spawn(async move {
                    let _permit = semaphore_clone.acquire().await.expect("Failed to acquire semaphore");
                    crawl_task(client_clone, manager_clone, url).await;
                    *running_tasks_clone.lock().await -= 1;
                });
            }
        }
    }
}

async fn crawl_task(client: Arc<Client>, url_manager: Arc<Mutex<UrlManager>>, url: String) {
    match fetch_url(&client, &url).await {
        Ok(body) => {
            let links = extract_links(&body);
            println!("Found {} links at {}", links.len(), url);
            let mut manager = url_manager.lock().await;
            manager.add_url(links);
        }
        Err(e) => {
            eprintln!("Failed to fetch {}: {}", url, e);
        }
    }
}

#[cfg(test)]
mod test {
    // TODO: Add tests duay na
}