mod bsd;
mod linux;
mod store_data;
mod utils;

use std::io::Write;

use store_data::{ToOS, OS};
use tokio::spawn;

#[tokio::main]
async fn main() {
    env_logger::Builder::new().filter_level(log::LevelFilter::Debug).init();
    let futures = vec![
        spawn(bsd::FreeBSD {}.to_os()),
        spawn(linux::Ubuntu {}.to_os()),
        spawn(linux::UbuntuServer {}.to_os()),
        spawn(linux::UbuntuUnity {}.to_os()),
        spawn(linux::Lubuntu {}.to_os()),
        spawn(linux::Kubuntu {}.to_os()),
        spawn(linux::UbuntuMATE {}.to_os()),
        spawn(linux::UbuntuBudgie {}.to_os()),
        spawn(linux::UbuntuStudio {}.to_os()),
        spawn(linux::UbuntuKylin {}.to_os()),
        spawn(linux::Edubuntu {}.to_os()),
        spawn(linux::Xubuntu {}.to_os()),
        spawn(linux::UbuntuCinnamon {}.to_os()),
    ];

    let distros = futures::future::join_all(futures)
        .await
        .into_iter()
        .flatten()
        .collect::<Vec<OS>>();

    if let Ok(output) = serde_json::to_string_pretty(&distros) {
        println!("{}", output);
    }

    let output = serde_json::to_string(&distros).unwrap();
    let mut file = std::fs::File::create_new("quickget_data.json").unwrap();
    file.write_all(output.as_bytes()).unwrap();
}
