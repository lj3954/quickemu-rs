mod config;

use anyhow::{bail, ensure, Result};
use clap::Parser;
use config::ListType;
use quickemu_core::data::Arch;
use quickget_core::{QuickgetConfig, QuickgetInstance};
use std::io::Write;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let arch = args
        .arch
        .map(|a| {
            Ok(match a.as_str() {
                "x86_64" => Arch::X86_64 { machine: Default::default() },
                "aarch64" | "AArch64" => Arch::AArch64 { machine: Default::default() },
                "riscv64" => Arch::Riscv64 { machine: Default::default() },
                _ => bail!("Invalid architecture: {a}"),
            })
        })
        .transpose()?;

    if let Some(list_type) = args.list {
        ensure!(
            args.other.is_empty(),
            "An operating system must not be specified for list operations"
        );
        return config::list(list_type, args.refresh).await;
    }

    let config = config::get(&args.other, arch.as_ref(), args.refresh).await?;
    println!("{config:#?}");
    let file = create_config(config).await?;
    println!("Completed. File {file:?}");

    Ok(())
}

async fn create_config(config: QuickgetConfig) -> Result<std::fs::File> {
    let mut instance = QuickgetInstance::new(config, std::env::current_dir().unwrap())?;
    instance.create_vm_dir(true)?;
    let downloads = instance.get_downloads();
    let docker_commands = instance.get_docker_commands();
    for mut command in docker_commands {
        let status = command.status()?;
        if !status.success() {
            anyhow::bail!("Failed to run docker command: {:?}", command);
        }
    }

    let client = reqwest::Client::new();
    for download in downloads {
        let mut request = client.get(download.url);

        if let Some(headers) = download.headers {
            request = request.headers(headers);
        }
        let mut response = request.send().await?;
        let length = response.content_length().unwrap_or_default();

        let progress = indicatif::ProgressBar::new(length);
        progress.set_style(
            indicatif::ProgressStyle::with_template("{bar:30.blue/red} ({percent}%) {bytes:>12.green} / {total_bytes:<12.green} {bytes_per_sec:>13.blue} - ETA: {eta_precise}")
                .unwrap()
                .progress_chars("━╾╴─"),
        );

        let mut file = std::fs::File::create(download.path)?;
        while let Some(chunk) = response.chunk().await? {
            progress.inc(chunk.len() as u64);
            file.write_all(&chunk)?;
        }
    }

    instance.create_config().map_err(Into::into)
}

#[derive(Debug, Parser)]
#[clap(group = clap::ArgGroup::new("actions").multiple(false))]
struct Args {
    #[clap(short, long)]
    arch: Option<String>,
    #[command(flatten)]
    verbose: clap_verbosity_flag::Verbosity<clap_verbosity_flag::WarnLevel>,
    #[clap(short, long)]
    refresh: bool,
    #[clap(short, long, group = "actions")]
    open_homepage: bool,
    #[clap(short, long, group = "actions")]
    url: bool,
    #[clap(short, long, group = "actions")]
    download_only: bool,
    #[clap(short, long, group = "actions")]
    list: Option<Option<ListType>>,
    other: Vec<String>,
}
