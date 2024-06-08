mod bsd;
mod linux;
mod store_data;
mod utils;

use std::{fs::File, io::Write};

use store_data::{ToOS, OS};
use tokio::spawn;

#[tokio::main]
async fn main() {
    env_logger::Builder::new().filter_level(log::LevelFilter::Debug).init();
    let futures = vec![
        spawn(bsd::FreeBSD.to_os()),
        spawn(linux::Ubuntu.to_os()),
        spawn(linux::UbuntuServer.to_os()),
        spawn(linux::UbuntuUnity.to_os()),
        spawn(linux::Lubuntu.to_os()),
        spawn(linux::Kubuntu.to_os()),
        spawn(linux::UbuntuMATE.to_os()),
        spawn(linux::UbuntuBudgie.to_os()),
        spawn(linux::UbuntuStudio.to_os()),
        spawn(linux::UbuntuKylin.to_os()),
        spawn(linux::Edubuntu.to_os()),
        spawn(linux::Xubuntu.to_os()),
        spawn(linux::UbuntuCinnamon.to_os()),
        spawn(linux::NixOS.to_os()),
        spawn(linux::Alma.to_os()),
        spawn(linux::Alpine.to_os()),
    ];

    let mut distros = futures::future::join_all(futures)
        .await
        .into_iter()
        .flatten()
        .collect::<Vec<OS>>();

    sort_distros(&mut distros);

    if let Ok(output) = serde_json::to_string_pretty(&distros) {
        println!("{}", output);
    }

    // Placeholder: Disable dead code warning for url only
    let _ = store_data::WebSource::url_only("");

    let output = serde_json::to_string(&distros).unwrap();
    let mut file = File::create("quickget_data.json").unwrap();
    file.write_all(output.as_bytes()).unwrap();
}

fn sort_distros(distros: &mut [OS]) {
    distros.sort_unstable_by(|a, b| a.name.cmp(&b.name));
    distros.iter_mut().for_each(|d| {
        d.releases.sort_unstable_by(|a, b| {
            if let (Some(release_a), Some(release_b)) = (&a.release, &b.release) {
                let (mut a, mut b) = (release_a.split('.'), release_b.split('.'));
                while let (Some(a), Some(b)) = (a.next(), b.next()) {
                    if let (Ok(a), Ok(b)) = (a.parse::<u64>(), b.parse::<u64>()) {
                        let comparison = b.cmp(&a);
                        if comparison != std::cmp::Ordering::Equal {
                            return comparison;
                        }
                    }
                }
            }
            b.release.cmp(&a.release).then(a.edition.cmp(&b.edition))
        })
    });
}
