pub mod data_structures;

mod config_search;
mod error;
mod instance;
pub use config_search::{ConfigSearch, QuickgetConfig};
pub use instance::{QGDockerSource, QGDownload, QuickgetInstance};
