use once_cell::sync::Lazy;
use reqwest::{Client, StatusCode};
use tokio::{spawn, sync::Semaphore};

pub async fn capture_page(url: &str) -> Option<String> {
    #![allow(dead_code)]
    let permit = CLIENT.semaphore.acquire().await.ok()?;
    let text = CLIENT.client.get(url).send().await.ok()?.text().await.ok()?;
    drop(permit);
    Some(text)
}

pub async fn all_valid(urls: Vec<String>) -> bool {
    let futures = urls.into_iter().map(|url| {
        spawn(async move {
            let permit = CLIENT.semaphore.acquire().await.ok()?;
            let response = CLIENT
                .client
                .get(&url)
                .send()
                .await
                .inspect_err(|e| {
                    log::warn!("Failed to make request to URL {}: {}", url, e);
                })
                .ok()?;
            let successful = response.status().is_success() || response.status() == StatusCode::TOO_MANY_REQUESTS;
            if !successful {
                log::warn!("Failed to resolve URL: {}", url);
            }
            drop(permit);
            Some(successful)
        })
    });
    futures::future::join_all(futures)
        .await
        .into_iter()
        .flatten()
        .all(|r| r.unwrap_or(true))
}

struct ReqwestClient {
    client: Client,
    semaphore: Semaphore,
}

static CLIENT: Lazy<ReqwestClient> = Lazy::new(|| {
    let client = Client::new();
    let semaphore = Semaphore::new(70);
    ReqwestClient { client, semaphore }
});
