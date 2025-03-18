use std::str::FromStr;

use clap::{builder::ValueParser, ArgGroup, Parser, ValueEnum};
use quickemu_core::data::{default_spice_port, Display, DisplayType, Resolution, Viewer};

#[cfg(not(target_os = "macos"))]
use quickemu_core::data::Access;

#[derive(Debug, Parser)]
pub(crate) struct DisplayArgs {
    #[clap(long)]
    display: Option<CliDisplayType>,
    #[cfg(not(target_os = "macos"))]
    #[clap(long, value_parser = Access::from_str)]
    access: Option<Access>,
    #[cfg(not(target_os = "macos"))]
    #[clap(long)]
    spice_port: Option<u16>,
    #[cfg(not(target_os = "macos"))]
    #[clap(long)]
    viewer: Option<CliViewer>,
    #[clap(flatten)]
    resolution: Option<ResolutionArgs>,
}

#[derive(Debug, Parser)]
struct ResolutionArgs {
    #[clap(long, requires = "height")]
    width: Option<u32>,
    #[clap(long, requires = "width")]
    height: Option<u32>,
    #[clap(long, conflicts_with_all = ["width", "height"])]
    fullscreen: bool,
}

impl From<ResolutionArgs> for Resolution {
    fn from(args: ResolutionArgs) -> Resolution {
        if args.fullscreen {
            Resolution::FullScreen
        } else if let (Some(width), Some(height)) = (args.width, args.height) {
            Resolution::Custom { width, height }
        } else {
            panic!("Empty resolution args were somehow constructed");
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
        }

        if let Some(resolution) = self.resolution {
            config.resolution = resolution.into();
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
