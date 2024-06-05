use anyhow::{bail, Result};
use itertools::Itertools;
use quickget_ci::{Arch, Config, OS};

pub trait FindEntry {
    fn find_entry(self, input: &[String], arch: Option<Arch>) -> Result<Config>
    where
        Self: Sized;
    fn os_list(&self) -> String;
}

impl FindEntry for Vec<OS> {
    fn find_entry(mut self, input: &[String], arch: Option<Arch>) -> Result<Config> {
        let matches_confirmed_arch = |c: &Config| {
            if let Some(ref arch) = &arch {
                if &c.arch != arch {
                    return false;
                }
            }
            true
        };
        let print_available_arches = |os: &OS| {
            let (arch_list, len) = os.arch_list();
            if len == 1 {
                "".to_owned()
            } else {
                format!(
                    "\n\n{} also supports the following architectures (available releases may vary): {}",
                    os.pretty_name, arch_list
                )
            }
        };
        let (print_arch, preferred_arch) = match arch {
            Some(ref arch) => (" ".to_owned() + &arch.to_string(), arch.clone()),
            None => (Default::default(), Arch::from_physical()),
        };
        if input.is_empty() {
            bail!(
                "You must specify an Operating System.\n\nAvailable Operating systems: {}",
                self.os_list()
            );
        }
        let Some(os_index) = self.iter().position(|os| os.name.eq_ignore_ascii_case(&input[0])) else {
            bail!(
                "Specified Operating System {} is not available.\nAvailable Operating systems: {}",
                input[0],
                self.os_list()
            );
        };
        let os = self.get(os_index).unwrap();
        if let Some(arch) = arch.as_ref() {
            if !os.supports_arch(arch) {
                bail!(
                    "{} does not support the {} architecture.\nSupported architectures: {}\n\nOperating systems with {} support: {}",
                    os.pretty_name,
                    arch.to_string(),
                    os.arch_list().0,
                    arch.to_string(),
                    self.iter()
                        .filter(|os| os.supports_arch(arch))
                        .map(|os| os.name.as_str())
                        .collect::<Vec<&str>>()
                        .join(" ")
                );
            }
        }
        let release = input.get(1).map(|r| r.as_str()).unwrap_or_default();
        let matching_releases: Vec<(usize, &Config)> = os
            .releases
            .iter()
            .enumerate()
            .filter(|(_, c)| {
                let conf_release = c.release.as_deref().unwrap_or_default();
                matches_confirmed_arch(c) && conf_release.eq_ignore_ascii_case(release)
            })
            .collect();

        if matching_releases.is_empty() {
            let releases_list = os.releases_list(&preferred_arch);
            if release.is_empty() {
                bail!(
                    "You must specify a release for {}{print_arch}.\n\n{releases_list}{}",
                    os.pretty_name,
                    print_available_arches(&os),
                );
            } else {
                bail!(
                    "{release} is not a supported {}{print_arch} release.\n\n{releases_list}{}",
                    os.pretty_name,
                    print_available_arches(&os),
                );
            }
        }

        let edition = input.get(2).map(|r| r.as_str()).unwrap_or_default();
        let matching_editions: Vec<(usize, &Config)> = matching_releases
            .into_iter()
            .filter(|(_, c)| {
                let conf_edition = c.edition.as_deref().unwrap_or_default();
                matches_confirmed_arch(c) && conf_edition.eq_ignore_ascii_case(edition)
            })
            .collect();

        if matching_editions.is_empty() {
            let editions_list = os.editions_list(release, &preferred_arch);
            if edition.is_empty() {
                bail!(
                    "You must specify an edition for {}{print_arch} {release}.\n\nEditions: {editions_list}{}",
                    os.pretty_name,
                    print_available_arches(&os),
                );
            } else {
                bail!(
                    "{edition} is not a supported {}{print_arch} {release} edition.\n\nEditions: {editions_list}{}",
                    os.pretty_name,
                    print_available_arches(&os),
                );
            }
        }

        Ok(if matching_editions.len() > 1 {
            if arch.is_none() {
                let pos = matching_editions
                    .iter()
                    .find_map(|(i, c)| if c.arch == preferred_arch { Some(*i) } else { None })
                    .unwrap_or_default();
                self[os_index].releases.remove(pos)
            } else {
                bail!("Multiple configurations were somehow found. Please file an issue with more information.");
            }
        } else {
            let index = matching_editions[0].0;
            self[os_index].releases.remove(index)
        })
    }
    fn os_list(&self) -> String {
        self.iter().map(|os| os.name.as_str()).collect::<Vec<&str>>().join(" ")
    }
}

pub trait FromPhysical {
    fn from_physical() -> Self;
}

impl FromPhysical for Arch {
    fn from_physical() -> Self {
        let arch = std::env::consts::ARCH;
        match arch {
            "aarch64" => Arch::aarch64,
            "riscv64" => Arch::riscv64,
            _ => Arch::x86_64,
        }
    }
}

pub trait ListAvailable {
    fn releases_list(&self, arch: &Arch) -> String;
    fn editions_list(&self, release: &str, arch: &Arch) -> String;
    fn arch_list(&self) -> (String, usize);
    fn supports_arch(&self, arch: &Arch) -> bool;
}

impl ListAvailable for OS {
    fn releases_list(&self, arch: &Arch) -> String {
        let releases = self
            .releases
            .iter()
            .filter(|c| c.arch == *arch)
            .map(|c| c.release.as_deref().unwrap_or_default())
            .unique()
            .collect::<Vec<&str>>();
        let editions = releases
            .iter()
            .map(|r| self.editions_list(r, arch))
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
                .map(|r| format!("{r:<max_len$}  {}", self.editions_list(r, arch)))
                .collect::<Vec<String>>()
                .join("\n");

            format!("{:<max_len$}  Editions\n{output}", "Releases")
        }
    }
    fn editions_list(&self, release: &str, arch: &Arch) -> String {
        self.releases
            .iter()
            .filter(|c| {
                let conf_release = c.release.as_deref().unwrap_or_default();
                conf_release.eq_ignore_ascii_case(release) && c.arch == *arch
            })
            .map(|c| c.edition.as_deref().unwrap_or_default())
            .collect::<Vec<&str>>()
            .join(" ")
    }
    fn arch_list(&self) -> (String, usize) {
        let arches = self
            .releases
            .iter()
            .map(|c| c.arch.to_string())
            .unique()
            .collect::<Vec<String>>();
        let len = arches.len();
        (arches.join(" "), len)
    }
    fn supports_arch(&self, arch: &Arch) -> bool {
        self.releases.iter().any(|c| c.arch == *arch)
    }
}
