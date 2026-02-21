use qconnect_core::QueueVersion;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueueEventType {
    SrvrCtrlQueueState,
    SrvrCtrlQueueTracksAdded,
    SrvrCtrlQueueTracksLoaded,
    SrvrCtrlQueueTracksInserted,
    SrvrCtrlQueueTracksRemoved,
    SrvrCtrlQueueTracksReordered,
    SrvrCtrlQueueCleared,
    SrvrCtrlShuffleModeSet,
    SrvrCtrlAutoplayModeSet,
    SrvrCtrlAutoplayTracksLoaded,
    SrvrCtrlAutoplayTracksRemoved,
    SrvrCtrlQueueTracksAddedFromAutoplay,
    SrvrCtrlQueueErrorMessage,
}

impl QueueEventType {
    pub const fn as_message_type(self) -> &'static str {
        match self {
            Self::SrvrCtrlQueueState => "MESSAGE_TYPE_SRVR_CTRL_QUEUE_STATE",
            Self::SrvrCtrlQueueTracksAdded => "MESSAGE_TYPE_SRVR_CTRL_QUEUE_TRACKS_ADDED",
            Self::SrvrCtrlQueueTracksLoaded => "MESSAGE_TYPE_SRVR_CTRL_QUEUE_TRACKS_LOADED",
            Self::SrvrCtrlQueueTracksInserted => "MESSAGE_TYPE_SRVR_CTRL_QUEUE_TRACKS_INSERTED",
            Self::SrvrCtrlQueueTracksRemoved => "MESSAGE_TYPE_SRVR_CTRL_QUEUE_TRACKS_REMOVED",
            Self::SrvrCtrlQueueTracksReordered => "MESSAGE_TYPE_SRVR_CTRL_QUEUE_TRACKS_REORDERED",
            Self::SrvrCtrlQueueCleared => "MESSAGE_TYPE_SRVR_CTRL_QUEUE_CLEARED",
            Self::SrvrCtrlShuffleModeSet => "MESSAGE_TYPE_SRVR_CTRL_SHUFFLE_MODE_SET",
            Self::SrvrCtrlAutoplayModeSet => "MESSAGE_TYPE_SRVR_CTRL_AUTOPLAY_MODE_SET",
            Self::SrvrCtrlAutoplayTracksLoaded => "MESSAGE_TYPE_SRVR_CTRL_AUTOPLAY_TRACKS_LOADED",
            Self::SrvrCtrlAutoplayTracksRemoved => "MESSAGE_TYPE_SRVR_CTRL_AUTOPLAY_TRACKS_REMOVED",
            Self::SrvrCtrlQueueTracksAddedFromAutoplay => {
                "MESSAGE_TYPE_SRVR_CTRL_QUEUE_TRACKS_ADDED_FROM_AUTOPLAY"
            }
            Self::SrvrCtrlQueueErrorMessage => "MESSAGE_TYPE_SRVR_CTRL_QUEUE_ERROR_MESSAGE",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueServerEvent {
    pub event_type: QueueEventType,
    pub action_uuid: Option<String>,
    pub queue_version: Option<QueueVersion>,
    #[serde(default)]
    pub payload: Value,
}

impl QueueServerEvent {
    pub const fn message_type(&self) -> &'static str {
        self.event_type.as_message_type()
    }
}
