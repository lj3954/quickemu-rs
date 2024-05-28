mod bsd;
mod store_data;
mod utils;

fn main() {
    env_logger::Builder::new().filter_level(log::LevelFilter::Debug).init();
    let freebsd = bsd::FreeBSD {};
    let os: store_data::OS = freebsd.into();
    println!("{}", serde_json::to_string(&os).unwrap());
    println!("\n\n\nPRETTY:\n\n");
    println!("{}", serde_json::to_string_pretty(&os).unwrap());
}
