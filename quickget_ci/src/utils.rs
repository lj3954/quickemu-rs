use once_cell::sync::Lazy;
use reqwest::Client;

pub async fn capture_page(url: &str) -> Option<String> {
    CLIENT.get(url).send().await.ok()?.text().await.ok()
}

static CLIENT: Lazy<Client> = Lazy::new(|| Client::new());
