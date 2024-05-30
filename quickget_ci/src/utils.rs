use once_cell::sync::Lazy;
use reqwest::Client;
use tokio::spawn;

pub async fn capture_page(url: &str) -> Option<String> {
    #![allow(dead_code)]
    CLIENT.get(url).send().await.ok()?.text().await.ok()
}

pub async fn all_valid(urls: Vec<String>) -> bool {
    let futures = urls.into_iter().map(|url| {
        spawn(async move {
            let response = CLIENT
                .get(&url)
                .send()
                .await
                .inspect_err(|e| {
                    log::warn!("Failed to make request to URL {}: {}", url, e);
                })
                .ok()?;
            if !response.status().is_success() {
                log::warn!("Failed to resolve URL: {}", url);
            }
            Some(response.status().is_success())
        })
    });
    futures::future::join_all(futures)
        .await
        .into_iter()
        .flatten()
        .all(|r| r.unwrap_or(true))
}

static CLIENT: Lazy<Client> = Lazy::new(Client::new);
