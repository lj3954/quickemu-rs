use anyhow::{bail, Context, Result};
use clap::ValueEnum;
use itertools::Itertools;
use quickemu_core::data::Arch;
use quickget_core::{data_structures::Config, ConfigSearch, ConfigSearchError, QuickgetConfig};
use serde::Serialize;
use std::io::{stdout, Write};

async fn create_instance(refresh: bool) -> Result<ConfigSearch, ConfigSearchError> {
    if refresh {
        ConfigSearch::new_refreshed().await
    } else {
        ConfigSearch::new().await
    }
}

#[derive(Debug, Clone, ValueEnum)]
pub enum ListType {
    Csv,
    Json,
}

#[derive(Serialize)]
struct QuickgetList<'a> {
    #[serde(rename = "Display Name")]
    display_name: &'a str,
    #[serde(rename = "OS")]
    os: &'a str,
    #[serde(rename = "Release")]
    release: &'a str,
    #[serde(rename = "Option")]
    option: &'a str,
    #[serde(flatten)]
    #[serde(rename = "Arch")]
    arch: &'a Arch,
    #[serde(rename = "PNG")]
    png: String,
    #[serde(rename = "SVG")]
    svg: String,
}

pub async fn list(list_type: Option<ListType>, refresh: bool) -> Result<()> {
    let instance = create_instance(refresh).await?;
    let empty_str = "";
    let list = instance.get_os_list().iter().flat_map(|os| {
        os.releases.iter().map(|config| QuickgetList {
            display_name: os.pretty_name.as_str(),
            os: os.name.as_str(),
            release: config.release.as_str(),
            option: config.edition.as_deref().unwrap_or(empty_str),
            arch: &config.arch,
            png: format!(
                "https://quickemu-project.github.io/quickemu-icons/png/{}/{}-quickemu-white-pinkbg.png",
                os.name, os.name
            ),
            svg: format!(
                "https://quickemu-project.github.io/quickemu-icons/svg/{}/{}-quickemu-white-pinkbg.svg",
                os.name, os.name
            ),
        })
    });

    match list_type {
        Some(ListType::Csv) => {
            let mut wtr = csv::Writer::from_writer(stdout());
            for item in list {
                wtr.serialize(item)?;
            }
            wtr.flush()?;
        }
        Some(ListType::Json) => {
            let list: Vec<_> = list.collect();
            let mut stdout = stdout().lock();
            serde_json::to_writer_pretty(&mut stdout, &list)?;
            stdout.flush()?;
        }
        None => {
            let mut stdout = stdout().lock();
            for item in list {
                writeln!(&mut stdout, "{} {} {} {}", item.os, item.release, item.option, item.arch)?;
            }
            stdout.flush()?;
        }
    };
    Ok(())
}

pub async fn get(args: &[String], preferred_arch: Option<&Arch>, refresh: bool) -> Result<QuickgetConfig> {
    let mut instance = create_instance(refresh).await?;
    let mut args = args.iter();

    if let Some(arch) = preferred_arch {
        instance.filter_arch_supported_os(arch)?;
    }

    let os = args.next().with_context(|| {
        format!(
            "You must specify an operating system\n - Supported Operating Systems\n{}",
            instance.list_os_names().join(" ")
        )
    })?;
    let os = instance.filter_os(os)?;
    if let Some(arch) = preferred_arch {
        os.filter_arch(arch)?;
    }

    if let Some(release) = args.next() {
        instance.filter_release(release)?;
    } else {
        bail!("You must specify a release\n{}", list_releases(&os.releases, preferred_arch));
    }

    let editions = instance.list_editions().unwrap();
    if let Some(edition) = args.next() {
        instance.filter_edition(edition)?;
    } else if let Some(editions) = editions {
        bail!("You must specify an edition\n - Editions: {}", editions.join(" "));
    }

    instance.pick_best_match().map_err(Into::into)
}

fn list_releases(configs: &[Config], arch: Option<&Arch>) -> String {
    let releases = configs
        .iter()
        .filter(|c| arch.map_or(true, |arch| c.arch == *arch))
        .map(|c| c.release.as_str())
        .unique()
        .collect::<Vec<&str>>();
    let editions = releases
        .iter()
        .map(|r| editions_list(configs, r, arch))
        .filter(|s| !s.is_empty())
        .collect::<Vec<String>>();
    if editions.is_empty() {
        format!("Releases: {}", releases.join(" "))
    } else if editions.iter().all_equal() {
        format!("Releases: {}\nEditions: {}", releases.join(" "), editions[0])
    } else {
        let max_len = releases.iter().map(|r| r.len()).max().unwrap_or_default();
        let output = releases
            .iter()
            .map(|r| format!("{r:<max_len$}  {}", editions_list(configs, r, arch)))
            .collect::<Vec<String>>()
            .join("\n");

        format!("{:<max_len$}  Editions\n{output}", "Releases")
    }
}

fn editions_list(configs: &[Config], release: &str, arch: Option<&Arch>) -> String {
    configs
        .iter()
        .filter(|c| {
            let conf_release = c.release.as_str();
            conf_release.eq_ignore_ascii_case(release) && arch.map_or(true, |arch| *arch == c.arch)
        })
        .map(|c| c.edition.as_deref().unwrap_or_default())
        .unique()
        .collect::<Vec<&str>>()
        .join(" ")
}
