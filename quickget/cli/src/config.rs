use anyhow::{bail, Context, Result};
use itertools::Itertools;
use quickemu::config::Arch;
use quickget_core::{data_structures::Config, ConfigSearch, QuickgetConfig};

pub async fn get(args: &[String], preferred_arch: Option<&Arch>) -> Result<QuickgetConfig> {
    let mut instance = ConfigSearch::new().await?;
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
        .map(|c| c.release.as_deref().unwrap_or_default())
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
            let conf_release = c.release.as_deref().unwrap_or_default();
            conf_release.eq_ignore_ascii_case(release) && arch.map_or(true, |arch| *arch == c.arch)
        })
        .map(|c| c.edition.as_deref().unwrap_or_default())
        .collect::<Vec<&str>>()
        .join(" ")
}
