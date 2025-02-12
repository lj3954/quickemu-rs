pub mod data_structures;

#[cfg(feature = "quickget")]
mod config_search;
#[cfg(feature = "quickget")]
mod error;
#[cfg(feature = "quickget")]
mod instance;
#[cfg(feature = "quickget")]
pub use config_search::{ConfigSearch, QuickgetConfig};
#[cfg(feature = "quickget")]
pub use error::{ConfigSearchError, DLError};
#[cfg(feature = "quickget")]
pub use instance::{QGDockerSource, QGDownload, QuickgetInstance};
#[cfg(feature = "quickget")]
mod i18n;
