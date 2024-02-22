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

    use super::*;

    #[test]
    fn extract_link() {
        let html = r#"
            <html>
                <body>
                    <a href="http://google.com">Google</a>
                    <a href="http://yahoo.com">Yahoo</a>
                    <a href="http://bing.com">Bing</a>
                </body>
            </html>
        "#;

        let links = extract_links(html);
        
        // expect links to equals to 3
        assert_eq!(links.len(), 3);

        // expect links to contains the following
        assert_eq!(links[0], "http://google.com");
        assert_eq!(links[1], "http://yahoo.com");
        assert_eq!(links[2], "http://bing.com");
        
        // expects links to not contains the following
        assert_ne!(links[0], "http://facebook.com");
        assert_ne!(links[1], "http://twitter.com"); 
        assert_ne!(links[2], "http://instagram.com");
    }
    
    #[tokio::test]
    async fn test_crawl_task_success() {
        // setup your mock server or HTTP response
        let mock_server = mockito::mock("GET", "/test-page")
            .with_status(200)
            .with_body(r#"
                <html>
                    <body>
                        <a href="http://google.com">Google</a>
                        <a href="http://yahoo.com">Yahoo</a>
                        <a href="http://bing.com">Bing</a>
                    </body>
                </html>
            "#)
            .create();


        let client = Arc::new(Client::new());
        let url_manager = Arc::new(Mutex::new(UrlManager::new()));
        let url = mockito::server_url() + "/test-page";

        crawl_task(client, url_manager.clone(), url).await;

        // verify that links were added to the UrlManager
        let manager = url_manager.lock().await;
        assert!(!manager.is_empty(), "UrlManager should have linked Added");

        mock_server.assert();
    }
    
}