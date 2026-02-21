//! qconnect-app
//!
//! Application adapter that composes qconnect core + protocol + transport.

mod app;
mod error;
mod events;
mod feature_flags;
mod state;

pub use app::QconnectApp;
pub use error::QconnectAppError;
pub use events::{NoOpEventSink, QconnectAppEvent, QconnectEventSink};
pub use feature_flags::{
    QBZ_QCONNECT_PANEL_SWITCH, QBZ_QCONNECT_QUEUE_MODEL, QBZ_QCONNECT_STRICT_DOMAIN_ISOLATION,
    QBZ_QCONNECT_TRANSPORT,
};
pub use qconnect_core::{QConnectQueueState, QConnectRendererState, RendererCommand};
pub use qconnect_protocol::QueueCommandType;
pub use state::QconnectRuntimeState;
