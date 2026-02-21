//! qconnect-protocol
//!
//! Wire-level contracts for Qobuz Connect WS commands/events.

mod command;
mod decoder;
mod error;
mod event;
mod mapper;
mod queue_command_proto;
mod renderer;
mod wire;

pub use command::{QueueCommand, QueueCommandType};
pub use decoder::decode_queue_server_events;
pub use decoder::decode_renderer_server_commands;
pub use error::ProtocolError;
pub use event::{QueueEventType, QueueServerEvent};
pub use mapper::{build_qconnect_outbound_envelope, encode_queue_command_batch};
pub use renderer::{RendererCommandType, RendererServerCommand};
pub use wire::{
    build_outbound_envelope, decode_inbound_json, encode_outbound_json,
    encode_outbound_payload_bytes, parse_inbound_event, InboundEnvelope, OutboundEnvelope,
    QCONNECT_BACKEND_DESTINATION, QCONNECT_SERVICE,
};
