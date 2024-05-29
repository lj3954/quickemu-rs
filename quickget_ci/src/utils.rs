use once_cell::sync::Lazy;
use reqwest::Client;
use tokio::spawn;

pub async fn capture_page(url: &str) -> Option<String> {
    CLIENT.get(url).send().await.ok()?.text().await.ok()
}

pub async fn all_valid(urls: Vec<String>) -> bool {
    let futures = urls.into_iter().map(|url| {
        spawn(async move {
            let response = CLIENT.get(url).send().await.ok()?;
            Some(response.status().is_success())
        })
    });
    futures::future::join_all(futures)
        .await
        .into_iter()
        .flatten()
        .all(|r| r.unwrap_or(false))
}

static CLIENT: Lazy<Client> = Lazy::new(Client::new);
