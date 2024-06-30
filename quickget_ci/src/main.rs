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
    let futures = [
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
        spawn(linux::Antix.to_os()),
        spawn(linux::Archcraft.to_os()),
        spawn(linux::Elementary.to_os()),
        spawn(linux::ArchLinux.to_os()),
        spawn(linux::ArcoLinux.to_os()),
        spawn(linux::ArtixLinux.to_os()),
        spawn(linux::AthenaOS.to_os()),
        spawn(linux::Batocera.to_os()),
        spawn(linux::Bazzite.to_os()),
        spawn(linux::BigLinux.to_os()),
        spawn(linux::BlendOS.to_os()),
        spawn(linux::Bodhi.to_os()),
    ];

    let distros = futures::future::join_all(futures)
        .await
        .into_iter()
        .flatten()
        .flatten()
        .collect::<Vec<OS>>()
        .distro_sort();

    if let Ok(output) = serde_json::to_string_pretty(&distros) {
        println!("{}", output);
    }

    let output = serde_json::to_string(&distros).unwrap();

    output.write_with_compression("quickget_data.json", CompressionType::None);
    output.write_with_compression("quickget_data.json.gz", CompressionType::Gzip);
    output.write_with_compression("quickget_data.json.zst", CompressionType::Zstd);
}

trait DistroSort {
    fn distro_sort(self) -> Self;
}

impl DistroSort for Vec<OS> {
    fn distro_sort(mut self) -> Self {
        self.sort_unstable_by(|a, b| a.name.cmp(&b.name));
        self.iter_mut().for_each(|d| {
            d.releases.sort_unstable_by(|a, b| {
                if let (Some(release_a), Some(release_b)) = (&a.release, &b.release) {
                    let (release_a, release_b) = (release_a.trim_start_matches('v'), release_b.trim_start_matches('v'));
                    let (mut a, mut b) = (release_a.split('.'), release_b.split('.'));
                    while let (Some(a), Some(b)) = (a.next(), b.next()) {
                        if let (Ok(a), Ok(b)) = (a.parse::<u64>(), b.parse::<u64>()) {
                            let comparison = b.cmp(&a);
                            if comparison != std::cmp::Ordering::Equal {
                                return comparison;
                            }
                        } else {
                            break;
                        }
                    }
                }
                b.release.cmp(&a.release).then(a.edition.cmp(&b.edition))
            })
        });
        self
    }
}

enum CompressionType {
    None,
    Gzip,
    Zstd,
}

trait WriteCompressedData {
    fn write_with_compression(&self, filename: &str, compression: CompressionType);
}

impl WriteCompressedData for String {
    fn write_with_compression(&self, filename: &str, compression: CompressionType) {
        let mut file = File::create(filename).unwrap();
        let data = self.as_bytes();
        match compression {
            CompressionType::None => file.write_all(data).unwrap(),
            CompressionType::Gzip => {
                let mut compressor = libdeflater::Compressor::new(libdeflater::CompressionLvl::best());
                let mut output = vec![0; compressor.gzip_compress_bound(data.len())];
                let final_size = compressor.gzip_compress(data, &mut output).unwrap();
                output.resize(final_size, 0);
                file.write_all(&output).unwrap();
            }
            CompressionType::Zstd => zstd::stream::copy_encode(data, file, 22).unwrap(),
        }
    }
}
