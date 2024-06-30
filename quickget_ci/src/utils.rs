#![allow(dead_code)]
use once_cell::sync::Lazy;
use reqwest::{StatusCode, Url};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use serde::Deserialize;
use std::collections::HashMap;
use tokio::sync::Semaphore;

pub async fn capture_page(input: &str) -> Option<String> {
    let url: Url = input.parse().ok()?;
    let url_permit = match CLIENT.url_permits.get(url.host_str()?) {
        Some(semaphore) => Some(semaphore.acquire().await.ok()?),
        None => None,
    };

    let permit = CLIENT.semaphore.acquire().await.ok()?;
    let response = CLIENT.client.get(url).send().await.ok()?;

    let status = response.status();
    let output = if status.is_success() {
        response.text().await.ok().filter(|text| !text.is_empty())
    } else {
        log::warn!("Failed to capture page: {}, {}", input, status);
        None
    };

    drop(permit);
    if let Some(url_permit) = url_permit {
        drop(url_permit);
    }
    output
}

pub async fn all_valid(urls: Vec<String>) -> bool {
    let futures = urls.into_iter().map(|input| async move {
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
                log::error!("Failed to make request to URL {}: {}", input, e);
            })
            .ok()?;
        let status = response.status();
        let successful = status.is_success() || status == StatusCode::TOO_MANY_REQUESTS;

        if !successful {
            log::warn!("Failed to resolve URL {}: {}", input, status);
        }
        drop(permit);
        if let Some(url_permit) = url_permit {
            drop(url_permit);
        }
        Some(successful)
    });
    futures::future::join_all(futures).await.into_iter().all(|r| r.unwrap_or(true))
}

struct ReqwestClient {
    client: ClientWithMiddleware,
    semaphore: Semaphore,
    url_permits: HashMap<&'static str, Semaphore>,
}

static CLIENT: Lazy<ReqwestClient> = Lazy::new(|| {
    let retries = ExponentialBackoff::builder().build_with_max_retries(3);
    let client = reqwest::ClientBuilder::new().user_agent("quickemu-rs/1.0").build().unwrap();
    let client = ClientBuilder::new(client)
        .with(RetryTransientMiddleware::new_with_policy(retries))
        .build();
    let semaphore = Semaphore::new(150);
    let url_permits = HashMap::from([("sourceforge.net", Semaphore::new(5))]);
    ReqwestClient { client, semaphore, url_permits }
});

pub trait GatherData {
    type Output;
    async fn gather_data(url: &str) -> Option<Self::Output>;
}

pub struct GithubAPI;
impl GatherData for GithubAPI {
    type Output = Vec<GithubAPIValue>;
    async fn gather_data(url: &str) -> Option<Self::Output> {
        let data = capture_page(url).await?;
        serde_json::from_str(&data).ok()
    }
}
#[derive(Deserialize)]
pub struct GithubAPIValue {
    pub tag_name: String,
    pub assets: Vec<GithubAsset>,
    pub prerelease: bool,
}
#[derive(Deserialize)]
pub struct GithubAsset {
    pub name: String,
    pub browser_download_url: String,
}
