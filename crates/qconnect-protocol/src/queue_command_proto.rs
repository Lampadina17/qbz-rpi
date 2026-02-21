#[derive(Clone, Copy, PartialEq, Eq, Debug, ::prost::Enumeration)]
#[repr(i32)]
pub enum QConnectMessageType {
    MessageTypeCtrlSrvrClearQueue = 65,
    MessageTypeCtrlSrvrQueueLoadTracks = 66,
    MessageTypeCtrlSrvrQueueInsertTracks = 67,
    MessageTypeCtrlSrvrQueueAddTracks = 68,
    MessageTypeCtrlSrvrQueueRemoveTracks = 69,
    MessageTypeCtrlSrvrQueueReorderTracks = 70,
    MessageTypeCtrlSrvrSetShuffleMode = 71,
    MessageTypeCtrlSrvrSetQueueState = 75,
    MessageTypeCtrlSrvrAskForQueueState = 76,
    MessageTypeCtrlSrvrSetAutoplayMode = 78,
    MessageTypeCtrlSrvrAutoplayLoadTracks = 79,
    MessageTypeCtrlSrvrAutoplayRemoveTracks = 80,
    MessageTypeSrvrCtrlQueueErrorMessage = 88,
    MessageTypeSrvrCtrlQueueCleared = 89,
    MessageTypeSrvrCtrlQueueState = 90,
    MessageTypeSrvrCtrlQueueTracksLoaded = 91,
    MessageTypeSrvrCtrlQueueTracksInserted = 92,
    MessageTypeSrvrCtrlQueueTracksAdded = 93,
    MessageTypeSrvrCtrlQueueTracksRemoved = 94,
    MessageTypeSrvrCtrlQueueTracksReordered = 95,
    MessageTypeSrvrCtrlShuffleModeSet = 96,
    MessageTypeSrvrCtrlAutoplayModeSet = 102,
    MessageTypeSrvrCtrlAutoplayTracksLoaded = 103,
    MessageTypeSrvrCtrlAutoplayTracksRemoved = 104,
    MessageTypeSrvrCtrlQueueTracksAddedFromAutoplay = 105,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueueVersionRef {
    #[prost(int32, optional, tag = "1")]
    pub major: Option<i32>,
    #[prost(int32, optional, tag = "2")]
    pub minor: Option<i32>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ErrorMessage {
    #[prost(int32, optional, tag = "1")]
    pub code: Option<i32>,
    #[prost(string, optional, tag = "2")]
    pub message: Option<String>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueueTrack {
    #[prost(int32, optional, tag = "1")]
    pub queue_item_id: Option<i32>,
    #[prost(int32, optional, tag = "2")]
    pub track_id: Option<i32>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueueTrackWithContext {
    #[prost(int32, optional, tag = "1")]
    pub queue_item_id: Option<i32>,
    #[prost(int32, optional, tag = "2")]
    pub track_id: Option<i32>,
    #[prost(bytes = "vec", optional, tag = "3")]
    pub context_uuid: Option<Vec<u8>>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SetQueueTrackWithContext {
    #[prost(int32, optional, tag = "1")]
    pub track_id: Option<i32>,
    #[prost(bytes = "vec", optional, tag = "2")]
    pub context_uuid: Option<Vec<u8>>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ClearQueueMessage {
    #[prost(message, optional, tag = "1")]
    pub queue_version_ref: Option<QueueVersionRef>,
    #[prost(bytes = "vec", optional, tag = "2")]
    pub action_uuid: Option<Vec<u8>>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueueLoadTracksMessage {
    #[prost(message, optional, tag = "1")]
    pub queue_version_ref: Option<QueueVersionRef>,
    #[prost(bytes = "vec", optional, tag = "2")]
    pub action_uuid: Option<Vec<u8>>,
    #[prost(int32, repeated, packed = "false", tag = "3")]
    pub track_ids: Vec<i32>,
    #[prost(int32, optional, tag = "4")]
    pub queue_position: Option<i32>,
    #[prost(int32, optional, tag = "5")]
    pub shuffle_seed: Option<i32>,
    #[prost(int32, optional, tag = "6")]
    pub shuffle_pivot_index: Option<i32>,
    #[prost(bool, optional, tag = "7")]
    pub shuffle_mode: Option<bool>,
    #[prost(bytes = "vec", optional, tag = "8")]
    pub context_uuid: Option<Vec<u8>>,
    #[prost(bool, optional, tag = "9")]
    pub autoplay_reset: Option<bool>,
    #[prost(bool, optional, tag = "10")]
    pub autoplay_loading: Option<bool>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueueInsertTracksMessage {
    #[prost(message, optional, tag = "1")]
    pub queue_version_ref: Option<QueueVersionRef>,
    #[prost(bytes = "vec", optional, tag = "2")]
    pub action_uuid: Option<Vec<u8>>,
    #[prost(int32, repeated, packed = "false", tag = "3")]
    pub track_ids: Vec<i32>,
    #[prost(int32, optional, tag = "4")]
    pub insert_after: Option<i32>,
    #[prost(int32, optional, tag = "5")]
    pub shuffle_seed: Option<i32>,
    #[prost(bytes = "vec", optional, tag = "6")]
    pub context_uuid: Option<Vec<u8>>,
    #[prost(bool, optional, tag = "7")]
    pub autoplay_reset: Option<bool>,
    #[prost(bool, optional, tag = "8")]
    pub autoplay_loading: Option<bool>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueueAddTracksMessage {
    #[prost(message, optional, tag = "1")]
    pub queue_version_ref: Option<QueueVersionRef>,
    #[prost(bytes = "vec", optional, tag = "2")]
    pub action_uuid: Option<Vec<u8>>,
    #[prost(int32, repeated, packed = "false", tag = "3")]
    pub track_ids: Vec<i32>,
    #[prost(int32, optional, tag = "4")]
    pub shuffle_seed: Option<i32>,
    #[prost(bytes = "vec", optional, tag = "5")]
    pub context_uuid: Option<Vec<u8>>,
    #[prost(bool, optional, tag = "6")]
    pub autoplay_reset: Option<bool>,
    #[prost(bool, optional, tag = "7")]
    pub autoplay_loading: Option<bool>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueueRemoveTracksMessage {
    #[prost(message, optional, tag = "1")]
    pub queue_version_ref: Option<QueueVersionRef>,
    #[prost(bytes = "vec", optional, tag = "2")]
    pub action_uuid: Option<Vec<u8>>,
    #[prost(int32, repeated, packed = "false", tag = "3")]
    pub queue_item_ids: Vec<i32>,
    #[prost(bool, optional, tag = "4")]
    pub autoplay_reset: Option<bool>,
    #[prost(bool, optional, tag = "5")]
    pub autoplay_loading: Option<bool>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueueReorderTracksMessage {
    #[prost(message, optional, tag = "1")]
    pub queue_version_ref: Option<QueueVersionRef>,
    #[prost(bytes = "vec", optional, tag = "2")]
    pub action_uuid: Option<Vec<u8>>,
    #[prost(int32, repeated, packed = "false", tag = "3")]
    pub queue_item_ids: Vec<i32>,
    #[prost(int32, optional, tag = "4")]
    pub insert_after: Option<i32>,
    #[prost(bool, optional, tag = "5")]
    pub autoplay_reset: Option<bool>,
    #[prost(bool, optional, tag = "6")]
    pub autoplay_loading: Option<bool>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SetShuffleModeMessage {
    #[prost(message, optional, tag = "1")]
    pub queue_version_ref: Option<QueueVersionRef>,
    #[prost(bytes = "vec", optional, tag = "2")]
    pub action_uuid: Option<Vec<u8>>,
    #[prost(bool, optional, tag = "3")]
    pub shuffle_mode: Option<bool>,
    #[prost(int32, optional, tag = "4")]
    pub shuffle_seed: Option<i32>,
    #[prost(int32, optional, tag = "5")]
    pub shuffle_pivot_queue_item_id: Option<i32>,
    #[prost(bool, optional, tag = "6")]
    pub autoplay_reset: Option<bool>,
    #[prost(bool, optional, tag = "7")]
    pub autoplay_loading: Option<bool>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SetAutoplayModeMessage {
    #[prost(message, optional, tag = "1")]
    pub queue_version_ref: Option<QueueVersionRef>,
    #[prost(bytes = "vec", optional, tag = "2")]
    pub action_uuid: Option<Vec<u8>>,
    #[prost(bool, optional, tag = "3")]
    pub autoplay_mode: Option<bool>,
    #[prost(bool, optional, tag = "4")]
    pub autoplay_reset: Option<bool>,
    #[prost(bool, optional, tag = "5")]
    pub autoplay_loading: Option<bool>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AutoplayLoadTracksMessage {
    #[prost(message, optional, tag = "1")]
    pub queue_version_ref: Option<QueueVersionRef>,
    #[prost(bytes = "vec", optional, tag = "2")]
    pub action_uuid: Option<Vec<u8>>,
    #[prost(int32, repeated, packed = "false", tag = "3")]
    pub track_ids: Vec<i32>,
    #[prost(bytes = "vec", optional, tag = "4")]
    pub context_uuid: Option<Vec<u8>>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AutoplayRemoveTracksMessage {
    #[prost(message, optional, tag = "1")]
    pub queue_version_ref: Option<QueueVersionRef>,
    #[prost(bytes = "vec", optional, tag = "2")]
    pub action_uuid: Option<Vec<u8>>,
    #[prost(int32, repeated, packed = "false", tag = "3")]
    pub queue_item_ids: Vec<i32>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SetQueueStateMessage {
    #[prost(message, optional, tag = "1")]
    pub queue_version_ref: Option<QueueVersionRef>,
    #[prost(bytes = "vec", optional, tag = "2")]
    pub action_uuid: Option<Vec<u8>>,
    #[prost(message, repeated, tag = "3")]
    pub tracks: Vec<SetQueueTrackWithContext>,
    #[prost(bool, optional, tag = "4")]
    pub shuffle_mode: Option<bool>,
    #[prost(int32, repeated, packed = "false", tag = "5")]
    pub shuffled_track_indexes: Vec<i32>,
    #[prost(bool, optional, tag = "6")]
    pub autoplay_mode: Option<bool>,
    #[prost(bool, optional, tag = "7")]
    pub autoplay_loading: Option<bool>,
    #[prost(message, repeated, tag = "8")]
    pub autoplay_tracks: Vec<SetQueueTrackWithContext>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AskForQueueStateMessage {
    #[prost(message, optional, tag = "1")]
    pub queue_version_ref: Option<QueueVersionRef>,
    #[prost(bytes = "vec", optional, tag = "2")]
    pub action_uuid: Option<Vec<u8>>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueueErrorMessage {
    #[prost(message, optional, tag = "1")]
    pub queue_version: Option<QueueVersionRef>,
    #[prost(bytes = "vec", optional, tag = "2")]
    pub action_uuid: Option<Vec<u8>>,
    #[prost(message, optional, tag = "3")]
    pub error: Option<ErrorMessage>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueueClearedMessage {
    #[prost(message, optional, tag = "1")]
    pub queue_version: Option<QueueVersionRef>,
    #[prost(bytes = "vec", optional, tag = "2")]
    pub action_uuid: Option<Vec<u8>>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueueStateMessage {
    #[prost(message, optional, tag = "1")]
    pub queue_version: Option<QueueVersionRef>,
    #[prost(bytes = "vec", optional, tag = "2")]
    pub action_uuid: Option<Vec<u8>>,
    #[prost(message, repeated, tag = "3")]
    pub tracks: Vec<QueueTrackWithContext>,
    #[prost(bool, optional, tag = "4")]
    pub shuffle_mode: Option<bool>,
    #[prost(int32, repeated, packed = "false", tag = "5")]
    pub shuffled_track_indexes: Vec<i32>,
    #[prost(bool, optional, tag = "6")]
    pub autoplay_mode: Option<bool>,
    #[prost(bool, optional, tag = "7")]
    pub autoplay_loading: Option<bool>,
    #[prost(message, repeated, tag = "8")]
    pub autoplay_tracks: Vec<QueueTrackWithContext>,
    #[prost(bytes = "vec", optional, tag = "100")]
    pub queue_hash: Option<Vec<u8>>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueueTracksLoadedMessage {
    #[prost(message, optional, tag = "1")]
    pub queue_version: Option<QueueVersionRef>,
    #[prost(bytes = "vec", optional, tag = "2")]
    pub action_uuid: Option<Vec<u8>>,
    #[prost(message, repeated, tag = "3")]
    pub tracks: Vec<QueueTrack>,
    #[prost(int32, optional, tag = "4")]
    pub queue_position: Option<i32>,
    #[prost(int32, optional, tag = "5")]
    pub shuffle_seed: Option<i32>,
    #[prost(int32, optional, tag = "6")]
    pub shuffle_pivot_queue_item_id: Option<i32>,
    #[prost(bool, optional, tag = "7")]
    pub shuffle_mode: Option<bool>,
    #[prost(bytes = "vec", optional, tag = "8")]
    pub context_uuid: Option<Vec<u8>>,
    #[prost(bool, optional, tag = "9")]
    pub autoplay_reset: Option<bool>,
    #[prost(bool, optional, tag = "10")]
    pub autoplay_loading: Option<bool>,
    #[prost(bytes = "vec", optional, tag = "100")]
    pub queue_hash: Option<Vec<u8>>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueueTracksInsertedMessage {
    #[prost(message, optional, tag = "1")]
    pub queue_version: Option<QueueVersionRef>,
    #[prost(bytes = "vec", optional, tag = "2")]
    pub action_uuid: Option<Vec<u8>>,
    #[prost(message, repeated, tag = "3")]
    pub tracks: Vec<QueueTrack>,
    #[prost(int32, optional, tag = "4")]
    pub insert_after: Option<i32>,
    #[prost(int32, optional, tag = "5")]
    pub shuffle_seed: Option<i32>,
    #[prost(bytes = "vec", optional, tag = "6")]
    pub context_uuid: Option<Vec<u8>>,
    #[prost(bool, optional, tag = "7")]
    pub autoplay_reset: Option<bool>,
    #[prost(bool, optional, tag = "8")]
    pub autoplay_loading: Option<bool>,
    #[prost(bytes = "vec", optional, tag = "100")]
    pub queue_hash: Option<Vec<u8>>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueueTracksAddedMessage {
    #[prost(message, optional, tag = "1")]
    pub queue_version: Option<QueueVersionRef>,
    #[prost(bytes = "vec", optional, tag = "2")]
    pub action_uuid: Option<Vec<u8>>,
    #[prost(message, repeated, tag = "3")]
    pub tracks: Vec<QueueTrack>,
    #[prost(int32, optional, tag = "4")]
    pub shuffle_seed: Option<i32>,
    #[prost(bytes = "vec", optional, tag = "5")]
    pub context_uuid: Option<Vec<u8>>,
    #[prost(bool, optional, tag = "6")]
    pub autoplay_reset: Option<bool>,
    #[prost(bool, optional, tag = "7")]
    pub autoplay_loading: Option<bool>,
    #[prost(bytes = "vec", optional, tag = "100")]
    pub queue_hash: Option<Vec<u8>>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueueTracksRemovedMessage {
    #[prost(message, optional, tag = "1")]
    pub queue_version: Option<QueueVersionRef>,
    #[prost(bytes = "vec", optional, tag = "2")]
    pub action_uuid: Option<Vec<u8>>,
    #[prost(int32, repeated, packed = "false", tag = "3")]
    pub queue_item_ids: Vec<i32>,
    #[prost(bool, optional, tag = "4")]
    pub autoplay_reset: Option<bool>,
    #[prost(bool, optional, tag = "5")]
    pub autoplay_loading: Option<bool>,
    #[prost(bytes = "vec", optional, tag = "100")]
    pub queue_hash: Option<Vec<u8>>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueueTracksReorderedMessage {
    #[prost(message, optional, tag = "1")]
    pub queue_version: Option<QueueVersionRef>,
    #[prost(bytes = "vec", optional, tag = "2")]
    pub action_uuid: Option<Vec<u8>>,
    #[prost(int32, repeated, packed = "false", tag = "3")]
    pub queue_item_ids: Vec<i32>,
    #[prost(int32, optional, tag = "4")]
    pub insert_after: Option<i32>,
    #[prost(bool, optional, tag = "5")]
    pub autoplay_reset: Option<bool>,
    #[prost(bool, optional, tag = "6")]
    pub autoplay_loading: Option<bool>,
    #[prost(bytes = "vec", optional, tag = "100")]
    pub queue_hash: Option<Vec<u8>>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ShuffleModeSetMessage {
    #[prost(message, optional, tag = "1")]
    pub queue_version: Option<QueueVersionRef>,
    #[prost(bytes = "vec", optional, tag = "2")]
    pub action_uuid: Option<Vec<u8>>,
    #[prost(bool, optional, tag = "3")]
    pub shuffle_mode: Option<bool>,
    #[prost(int32, optional, tag = "4")]
    pub shuffle_seed: Option<i32>,
    #[prost(int32, optional, tag = "5")]
    pub shuffle_pivot_queue_item_id: Option<i32>,
    #[prost(bool, optional, tag = "6")]
    pub autoplay_reset: Option<bool>,
    #[prost(bool, optional, tag = "7")]
    pub autoplay_loading: Option<bool>,
    #[prost(bytes = "vec", optional, tag = "100")]
    pub queue_hash: Option<Vec<u8>>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AutoplayModeSetMessage {
    #[prost(message, optional, tag = "1")]
    pub queue_version: Option<QueueVersionRef>,
    #[prost(bytes = "vec", optional, tag = "2")]
    pub action_uuid: Option<Vec<u8>>,
    #[prost(bool, optional, tag = "3")]
    pub autoplay_mode: Option<bool>,
    #[prost(bool, optional, tag = "4")]
    pub autoplay_reset: Option<bool>,
    #[prost(bool, optional, tag = "5")]
    pub autoplay_loading: Option<bool>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AutoplayTracksLoadedMessage {
    #[prost(message, optional, tag = "1")]
    pub queue_version: Option<QueueVersionRef>,
    #[prost(bytes = "vec", optional, tag = "2")]
    pub action_uuid: Option<Vec<u8>>,
    #[prost(message, repeated, tag = "3")]
    pub tracks: Vec<QueueTrack>,
    #[prost(bytes = "vec", optional, tag = "4")]
    pub context_uuid: Option<Vec<u8>>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AutoplayTracksRemovedMessage {
    #[prost(message, optional, tag = "1")]
    pub queue_version: Option<QueueVersionRef>,
    #[prost(bytes = "vec", optional, tag = "2")]
    pub action_uuid: Option<Vec<u8>>,
    #[prost(int32, repeated, packed = "false", tag = "3")]
    pub queue_item_ids: Vec<i32>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueueTracksAddedFromAutoplayMessage {
    #[prost(message, optional, tag = "1")]
    pub queue_version: Option<QueueVersionRef>,
    #[prost(int32, repeated, packed = "false", tag = "2")]
    pub queue_item_ids: Vec<i32>,
    #[prost(bytes = "vec", optional, tag = "100")]
    pub queue_hash: Option<Vec<u8>>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QConnectMessages {
    #[prost(uint64, optional, tag = "1")]
    pub messages_time: Option<u64>,
    #[prost(int32, optional, tag = "2")]
    pub messages_id: Option<i32>,
    #[prost(message, repeated, tag = "3")]
    pub messages: Vec<QConnectMessage>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QConnectMessage {
    #[prost(int32, optional, tag = "1")]
    pub message_type: Option<i32>,
    #[prost(message, optional, tag = "65")]
    pub ctrl_srvr_clear_queue: Option<ClearQueueMessage>,
    #[prost(message, optional, tag = "66")]
    pub ctrl_srvr_queue_load_tracks: Option<QueueLoadTracksMessage>,
    #[prost(message, optional, tag = "67")]
    pub ctrl_srvr_queue_insert_tracks: Option<QueueInsertTracksMessage>,
    #[prost(message, optional, tag = "68")]
    pub ctrl_srvr_queue_add_tracks: Option<QueueAddTracksMessage>,
    #[prost(message, optional, tag = "69")]
    pub ctrl_srvr_queue_remove_tracks: Option<QueueRemoveTracksMessage>,
    #[prost(message, optional, tag = "70")]
    pub ctrl_srvr_queue_reorder_tracks: Option<QueueReorderTracksMessage>,
    #[prost(message, optional, tag = "71")]
    pub ctrl_srvr_set_shuffle_mode: Option<SetShuffleModeMessage>,
    #[prost(message, optional, tag = "75")]
    pub ctrl_srvr_set_queue_state: Option<SetQueueStateMessage>,
    #[prost(message, optional, tag = "76")]
    pub ctrl_srvr_ask_for_queue_state: Option<AskForQueueStateMessage>,
    #[prost(message, optional, tag = "78")]
    pub ctrl_srvr_set_autoplay_mode: Option<SetAutoplayModeMessage>,
    #[prost(message, optional, tag = "79")]
    pub ctrl_srvr_autoplay_load_tracks: Option<AutoplayLoadTracksMessage>,
    #[prost(message, optional, tag = "80")]
    pub ctrl_srvr_autoplay_remove_tracks: Option<AutoplayRemoveTracksMessage>,
    #[prost(message, optional, tag = "88")]
    pub srvr_ctrl_queue_error_message: Option<QueueErrorMessage>,
    #[prost(message, optional, tag = "89")]
    pub srvr_ctrl_queue_cleared: Option<QueueClearedMessage>,
    #[prost(message, optional, tag = "90")]
    pub srvr_ctrl_queue_state: Option<QueueStateMessage>,
    #[prost(message, optional, tag = "91")]
    pub srvr_ctrl_queue_tracks_loaded: Option<QueueTracksLoadedMessage>,
    #[prost(message, optional, tag = "92")]
    pub srvr_ctrl_queue_tracks_inserted: Option<QueueTracksInsertedMessage>,
    #[prost(message, optional, tag = "93")]
    pub srvr_ctrl_queue_tracks_added: Option<QueueTracksAddedMessage>,
    #[prost(message, optional, tag = "94")]
    pub srvr_ctrl_queue_tracks_removed: Option<QueueTracksRemovedMessage>,
    #[prost(message, optional, tag = "95")]
    pub srvr_ctrl_queue_tracks_reordered: Option<QueueTracksReorderedMessage>,
    #[prost(message, optional, tag = "96")]
    pub srvr_ctrl_shuffle_mode_set: Option<ShuffleModeSetMessage>,
    #[prost(message, optional, tag = "102")]
    pub srvr_ctrl_autoplay_mode_set: Option<AutoplayModeSetMessage>,
    #[prost(message, optional, tag = "103")]
    pub srvr_ctrl_autoplay_tracks_loaded: Option<AutoplayTracksLoadedMessage>,
    #[prost(message, optional, tag = "104")]
    pub srvr_ctrl_autoplay_tracks_removed: Option<AutoplayTracksRemovedMessage>,
    #[prost(message, optional, tag = "105")]
    pub srvr_ctrl_queue_tracks_added_from_autoplay: Option<QueueTracksAddedFromAutoplayMessage>,
}
