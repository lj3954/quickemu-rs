use once_cell::sync::Lazy;
use reqwest::{Client, StatusCode, Url};
use std::collections::HashMap;
use tokio::{spawn, sync::Semaphore};

pub async fn capture_page(input: &str) -> Option<String> {
    #![allow(dead_code)]
    let url: Url = input.parse().ok()?;
    let url_permit = match CLIENT.url_permits.get(url.host_str()?) {
        Some(semaphore) => Some(semaphore.acquire().await.ok()?),
        None => None,
    };

    let permit = CLIENT.semaphore.acquire().await.ok()?;
    let response = CLIENT.client.get(url).send().await.ok()?;

    let output = if response.status().is_success() {
        response.text().await.ok().filter(|text| !text.is_empty())
    } else {
        log::warn!("Failed to capture page: {}", input);
        None
    };

    drop(permit);
    if let Some(url_permit) = url_permit {
        drop(url_permit);
    }
    output
}

pub async fn all_valid(urls: Vec<String>) -> bool {
    let futures = urls.into_iter().map(|input| {
        spawn(async move {
            let url: Url = input.parse().ok()?;
            let url_permit = match CLIENT.url_permits.get(url.host_str()?) {
                Some(semaphore) => Some(semaphore.acquire().await.ok()?),
                None => None,
            };
            let permit = CLIENT.semaphore.acquire().await.ok()?;

            let response = CLIENT
                .client
                .get(url)
                .send()
                .await
                .inspect_err(|e| {
                    log::warn!("Failed to make request to URL {}: {}", input, e);
                })
                .ok()?;
            let successful = response.status().is_success() || response.status() == StatusCode::TOO_MANY_REQUESTS;
            if !successful {
                log::warn!("Failed to resolve URL: {}", input);
            }
            drop(permit);
            if let Some(url_permit) = url_permit {
                drop(url_permit);
            }
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
    url_permits: HashMap<&'static str, Semaphore>,
}

static CLIENT: Lazy<ReqwestClient> = Lazy::new(|| {
    let client = Client::new();
    let semaphore = Semaphore::new(70);
    let url_permits = HashMap::from([("sourceforge.net", Semaphore::new(5))]);
    ReqwestClient { client, semaphore, url_permits }
});
