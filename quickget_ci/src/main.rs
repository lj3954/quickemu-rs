mod bsd;
mod linux;
mod store_data;
mod utils;

use store_data::{ToOS, OS};
use tokio::spawn;

#[tokio::main]
async fn main() {
    env_logger::Builder::new().filter_level(log::LevelFilter::Debug).init();
    let futures = vec![spawn(linux::Ubuntu {}.to_os()), spawn(bsd::FreeBSD {}.to_os())];

    let distros = futures::future::join_all(futures)
        .await
        .into_iter()
        .flatten()
        .collect::<Vec<OS>>();
    println!("{}", serde_json::to_string(&distros).unwrap());
    println!("\n\n\nPRETTY:\n\n");
    println!("{}", serde_json::to_string_pretty(&distros).unwrap());
}
