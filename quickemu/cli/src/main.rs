use std::path::Path;

use quickemu_core::config::Config;
fn main() {
    let config_file = std::env::args().nth(1).expect("config file");
    let config = Config::parse(Path::new(&config_file)).expect("config file");
    println!("{:#?}", config);
    config.launch().expect("Couldn't launch");
}
