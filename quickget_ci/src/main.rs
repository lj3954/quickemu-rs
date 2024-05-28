mod bsd;
mod linux;
mod store_data;
mod utils;

use store_data::ToOS;

#[tokio::main]
async fn main() {
    env_logger::Builder::new().filter_level(log::LevelFilter::Debug).init();
    let freebsd = bsd::FreeBSD {};
    let os: store_data::OS = freebsd.to_os().await;
    println!("{}", serde_json::to_string(&os).unwrap());
    println!("\n\n\nPRETTY:\n\n");
    println!("{}", serde_json::to_string_pretty(&os).unwrap());
}
