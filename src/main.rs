ีuse tokio::sync::{Mutex, Semaphore};
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
    // limit to 10 concurrent request inorder to not get by ip banned
    let semaphore = Arc::new(Semaphore::new(10)); 

    let mut handles = vec![];

    for _ in 0..10 {
        let manager_clone = url_manager.clone();
        let semaphore_clone = semaphore.clone();

        let handle = tokio::spawn(async move {
            let _permit = semaphore_clone.acquire().await.expect("Failed to acquire semaphore");

            let url = {
                let mut manager = manager_clone.lock().await;
                manager.get_next_url();
            };

            if Some(url) = url {
                fetch_url(&url).await.expect("Failed to fetch the url");
                let links = extract_links(&html);
                // TODO: add the new urls to the manager
                let mut manager = manager_clone.lock().await;
                manager.add_url(links);
            }

        });


        handles.push(handle);
    }


    for handle in handles {
        handle.await.expect("Failed to run the task");
    }
}

#[cfg(test)]
mod test {
    // TODO: Add tests
}