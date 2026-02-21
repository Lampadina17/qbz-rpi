use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RendererCommandType {
    SrvrRndrSetState,
    SrvrRndrSetVolume,
    SrvrRndrSetActive,
    SrvrRndrSetMaxAudioQuality,
    SrvrRndrSetLoopMode,
    SrvrRndrSetShuffleMode,
    SrvrRndrMuteVolume,
}

impl RendererCommandType {
    pub const fn as_message_type(self) -> &'static str {
        match self {
            Self::SrvrRndrSetState => "MESSAGE_TYPE_SRVR_RNDR_SET_STATE",
            Self::SrvrRndrSetVolume => "MESSAGE_TYPE_SRVR_RNDR_SET_VOLUME",
            Self::SrvrRndrSetActive => "MESSAGE_TYPE_SRVR_RNDR_SET_ACTIVE",
            Self::SrvrRndrSetMaxAudioQuality => "MESSAGE_TYPE_SRVR_RNDR_SET_MAX_AUDIO_QUALITY",
            Self::SrvrRndrSetLoopMode => "MESSAGE_TYPE_SRVR_RNDR_SET_LOOP_MODE",
            Self::SrvrRndrSetShuffleMode => "MESSAGE_TYPE_SRVR_RNDR_SET_SHUFFLE_MODE",
            Self::SrvrRndrMuteVolume => "MESSAGE_TYPE_SRVR_RNDR_MUTE_VOLUME",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RendererServerCommand {
    pub command_type: RendererCommandType,
    #[serde(default)]
    pub payload: Value,
}

impl RendererServerCommand {
    pub const fn message_type(&self) -> &'static str {
        self.command_type.as_message_type()
    }
}
