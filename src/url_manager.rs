use std::collections::HashSet;

pub struct UrlManager {
    visited: HashSet<String>,
    to_visit: HashSet<String>,
}

impl UrlManager {
    pub fn new() -> Self {
        UrlManager {
            visited: HashSet::new(),
            to_visit: HashSet::new()
        }
    }

    pub fn add_url(&mut self, urls: Vec<String>) {
        for url in urls {
            if !self.visited.contains(&url) {
                self.to_visit.insert(url);
            }
        }
    }

    pub fn get_next_url(&mut self) -> Option<String> {
        self.to_visit.iter().next().cloned().map(|url| {
            self.to_visit.remove(&url);
            self.visited.insert(url.clone());
            url
        })
    }

    pub fn is_empty(&self) -> bool {
        self.to_visit.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn url_management() {
        let mut manager = UrlManager::new();
        
        // ensure that inital state is empty
        assert!(manager.get_next_url().is_none());

        // add url and ensure that it's return by get_next_url
        manager.add_url(vec!["http://google.com".into()]);
        assert_eq!(manager.get_next_url(), Some("http://google.com".into()));

        // after get_next_url, it should not be in to_visit
        assert!(manager.get_next_url().is_none());

        // adding the same url should not increase the size of to_visit or visited
        manager.add_url(vec!["http://google.com".into()]);
        assert!(manager.get_next_url().is_none());

        // ensure the url is marked as visited
        assert!(manager.visited.contains("http://google.com"));
    }
}