use qconnect_core::PendingActionError;
use qconnect_protocol::ProtocolError;
use qconnect_transport_ws::WsTransportError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum QconnectAppError {
    #[error(transparent)]
    Pending(#[from] PendingActionError),
    #[error(transparent)]
    Protocol(#[from] ProtocolError),
    #[error(transparent)]
    Transport(#[from] WsTransportError),
}
