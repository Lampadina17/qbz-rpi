//! AirPlay casting module (discovery + scaffolding)

pub mod device;
pub mod discovery;

pub use crate::AirPlayError;
pub use device::{AirPlayConnection, AirPlayMetadata, AirPlayStatus};
pub use discovery::{AirPlayDiscovery, DiscoveredAirPlayDevice};
