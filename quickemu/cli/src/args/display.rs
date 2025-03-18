use std::str::FromStr;

use clap::{Parser, ValueEnum};
use quickemu_core::data::{default_spice_port, Display, DisplayType, Resolution};

#[cfg(not(target_os = "macos"))]
use quickemu_core::data::{Access, Viewer};

#[derive(Debug, Parser)]
pub(crate) struct DisplayArgs {
    #[clap(long)]
    display: Option<CliDisplayType>,
    #[clap(long)]
    braille: bool,
    #[cfg(not(target_os = "macos"))]
    #[clap(long, value_parser = Access::from_str)]
    access: Option<Access>,
    #[cfg(not(target_os = "macos"))]
    #[clap(long)]
    spice_port: Option<u16>,
    #[cfg(not(target_os = "macos"))]
    #[clap(long)]
    viewer: Option<CliViewer>,
    #[clap(long, value_parser = CliResolution::parse, help = "A resolution ('width'x'height') or 'fullscreen'")]
    resolution: Option<Resolution>,
}

struct CliResolution;
impl CliResolution {
    fn parse(input: &str) -> Result<Resolution, String> {
        if input == "fullscreen" {
            Ok(Resolution::FullScreen)
        } else {
            let (width, height) = input.split_once('x').ok_or("Resolution is not formatted as expected")?;
            let width = width.parse().map_err(|e| format!("Invalid width: {e}"))?;
            let height = height.parse().map_err(|e| format!("Invalid height: {e}"))?;
            Ok(Resolution::Custom { width, height })
        }
    }
}

impl DisplayArgs {
    pub(crate) fn edit_config(self, config: &mut Display) {
        if let Some(display) = self.display {
            config.display_type = display.into();
        }

        #[cfg(not(target_os = "macos"))]
        if let DisplayType::Spice { access, viewer, spice_port } = &mut config.display_type {
            if let Some(selected_access) = self.access {
                *access = selected_access;
            }
            if let Some(selected_viewer) = self.viewer {
                *viewer = selected_viewer.into();
            }
            if let Some(selected_spice_port) = self.spice_port {
                *spice_port = selected_spice_port;
            }
        } else if self.spice_port.is_some() || self.viewer.is_some() || self.access.is_some() {
            panic!("Cannot specify Spice-specific options (spice port, access, viewer) when using a non-Spice display type");
        }

        if self.braille {
            config.braille = true;
        }
        if let Some(resolution) = self.resolution {
            config.resolution = resolution;
        }
    }
}

// #[derive(Debug, Parser)]
// struct Test {
//     #[clap(long, value_parser = parse_display)]
//     display: DisplayArgs,
// }
//
// pub(crate) fn parse_display(s: &str) -> Result<DisplayArgs, String> {
//     let args = DisplayArgs::try_parse_from
//
//     Ok(args)
// }

#[derive(Debug, Clone, ValueEnum)]
enum CliDisplayType {
    None,
    Sdl,
    Gtk,
    #[cfg(not(target_os = "macos"))]
    Spice,
    #[cfg(not(target_os = "macos"))]
    SpiceApp,
    #[cfg(target_os = "macos")]
    Cocoa,
}

impl From<CliDisplayType> for DisplayType {
    fn from(value: CliDisplayType) -> Self {
        match value {
            CliDisplayType::None => DisplayType::None,
            CliDisplayType::Sdl => DisplayType::Sdl,
            CliDisplayType::Gtk => DisplayType::Gtk,
            #[cfg(not(target_os = "macos"))]
            CliDisplayType::Spice => DisplayType::Spice {
                access: Access::default(),
                viewer: Viewer::default(),
                spice_port: default_spice_port(),
            },
            #[cfg(not(target_os = "macos"))]
            CliDisplayType::SpiceApp => DisplayType::SpiceApp,
            #[cfg(target_os = "macos")]
            CliDisplayType::Cocoa => DisplayType::Cocoa,
        }
    }
}

#[cfg(not(target_os = "macos"))]
#[derive(Debug, Clone, ValueEnum)]
enum CliViewer {
    None,
    Spicy,
    Remote,
}

#[cfg(not(target_os = "macos"))]
impl From<CliViewer> for Viewer {
    fn from(value: CliViewer) -> Self {
        match value {
            CliViewer::None => Viewer::None,
            CliViewer::Spicy => Viewer::Spicy,
            CliViewer::Remote => Viewer::Remote,
        }
    }
}
