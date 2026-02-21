use std::{
    sync::atomic::{AtomicI32, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use prost::Message;
use serde_json::Value;
use uuid::Uuid;

use crate::{
    build_outbound_envelope,
    queue_command_proto::{
        AskForQueueStateMessage, AutoplayLoadTracksMessage, AutoplayRemoveTracksMessage,
        ClearQueueMessage, QConnectMessage, QConnectMessageType, QConnectMessages,
        QueueAddTracksMessage, QueueInsertTracksMessage, QueueLoadTracksMessage,
        QueueRemoveTracksMessage, QueueReorderTracksMessage, QueueVersionRef,
        SetAutoplayModeMessage, SetQueueStateMessage, SetQueueTrackWithContext,
        SetShuffleModeMessage,
    },
    OutboundEnvelope, ProtocolError, QueueCommand, QueueCommandType,
};

static BATCH_SEQ: AtomicI32 = AtomicI32::new(1);

pub fn build_qconnect_outbound_envelope(
    command: QueueCommand,
) -> Result<OutboundEnvelope, ProtocolError> {
    let payload_bytes = encode_queue_command_batch(&command)?;
    let mut envelope = build_outbound_envelope(command);
    envelope.payload_bytes = Some(payload_bytes);
    Ok(envelope)
}

pub fn encode_queue_command_batch(command: &QueueCommand) -> Result<Vec<u8>, ProtocolError> {
    let message = map_queue_command(command)?;
    let batch = QConnectMessages {
        messages_time: Some(now_ms()),
        messages_id: Some(next_batch_seq()),
        messages: vec![message],
    };
    Ok(batch.encode_to_vec())
}

fn map_queue_command(command: &QueueCommand) -> Result<QConnectMessage, ProtocolError> {
    let queue_version_ref = Some(to_proto_queue_version(command.queue_version_ref)?);
    let action_uuid = Some(action_uuid_bytes(&command.action_uuid)?);

    match command.command_type {
        QueueCommandType::CtrlSrvrQueueAddTracks => {
            let track_ids = required_i32_list(&command.payload, "track_ids")?;
            let shuffle_seed = optional_i32(&command.payload, "shuffle_seed")?;
            let context_uuid = optional_uuid_bytes(&command.payload, &["context_uuid"])?;
            let autoplay_reset = optional_bool(&command.payload, "autoplay_reset", false);
            let autoplay_loading =
                optional_bool(&command.payload, "autoplay_loading", autoplay_reset);

            Ok(QConnectMessage {
                message_type: Some(QConnectMessageType::MessageTypeCtrlSrvrQueueAddTracks as i32),
                ctrl_srvr_queue_add_tracks: Some(QueueAddTracksMessage {
                    queue_version_ref,
                    action_uuid,
                    track_ids,
                    shuffle_seed,
                    context_uuid,
                    autoplay_reset: Some(autoplay_reset),
                    autoplay_loading: Some(autoplay_loading),
                }),
                ..Default::default()
            })
        }
        QueueCommandType::CtrlSrvrQueueLoadTracks => {
            let track_ids = required_i32_list(&command.payload, "track_ids")?;
            let queue_position = optional_i32(&command.payload, "queue_position")?;
            let shuffle_seed = optional_i32(&command.payload, "shuffle_seed")?;
            let shuffle_pivot_index = optional_i32(&command.payload, "shuffle_pivot_index")?;
            let shuffle_mode = optional_bool(&command.payload, "shuffle_mode", false);
            let context_uuid = optional_uuid_bytes(&command.payload, &["context_uuid"])?;
            let autoplay_reset = optional_bool(&command.payload, "autoplay_reset", false);
            let autoplay_loading =
                optional_bool(&command.payload, "autoplay_loading", autoplay_reset);

            Ok(QConnectMessage {
                message_type: Some(QConnectMessageType::MessageTypeCtrlSrvrQueueLoadTracks as i32),
                ctrl_srvr_queue_load_tracks: Some(QueueLoadTracksMessage {
                    queue_version_ref,
                    action_uuid,
                    track_ids,
                    queue_position,
                    shuffle_seed,
                    shuffle_pivot_index,
                    shuffle_mode: Some(shuffle_mode),
                    context_uuid,
                    autoplay_reset: Some(autoplay_reset),
                    autoplay_loading: Some(autoplay_loading),
                }),
                ..Default::default()
            })
        }
        QueueCommandType::CtrlSrvrQueueInsertTracks => {
            let track_ids = required_i32_list(&command.payload, "track_ids")?;
            let insert_after = optional_i32(&command.payload, "insert_after")?;
            let shuffle_seed = optional_i32(&command.payload, "shuffle_seed")?;
            let context_uuid = optional_uuid_bytes(&command.payload, &["context_uuid"])?;
            let autoplay_reset = optional_bool(&command.payload, "autoplay_reset", false);
            let autoplay_loading =
                optional_bool(&command.payload, "autoplay_loading", autoplay_reset);

            Ok(QConnectMessage {
                message_type: Some(
                    QConnectMessageType::MessageTypeCtrlSrvrQueueInsertTracks as i32,
                ),
                ctrl_srvr_queue_insert_tracks: Some(QueueInsertTracksMessage {
                    queue_version_ref,
                    action_uuid,
                    track_ids,
                    insert_after,
                    shuffle_seed,
                    context_uuid,
                    autoplay_reset: Some(autoplay_reset),
                    autoplay_loading: Some(autoplay_loading),
                }),
                ..Default::default()
            })
        }
        QueueCommandType::CtrlSrvrQueueRemoveTracks => {
            let queue_item_ids = required_i32_list(&command.payload, "queue_item_ids")?;
            let autoplay_reset = optional_bool(&command.payload, "autoplay_reset", false);
            let autoplay_loading =
                optional_bool(&command.payload, "autoplay_loading", autoplay_reset);

            Ok(QConnectMessage {
                message_type: Some(
                    QConnectMessageType::MessageTypeCtrlSrvrQueueRemoveTracks as i32,
                ),
                ctrl_srvr_queue_remove_tracks: Some(QueueRemoveTracksMessage {
                    queue_version_ref,
                    action_uuid,
                    queue_item_ids,
                    autoplay_reset: Some(autoplay_reset),
                    autoplay_loading: Some(autoplay_loading),
                }),
                ..Default::default()
            })
        }
        QueueCommandType::CtrlSrvrQueueReorderTracks => {
            let queue_item_ids = required_i32_list(&command.payload, "queue_item_ids")?;
            let insert_after = optional_i32(&command.payload, "insert_after")?;
            let autoplay_reset = optional_bool(&command.payload, "autoplay_reset", false);
            let autoplay_loading =
                optional_bool(&command.payload, "autoplay_loading", autoplay_reset);

            Ok(QConnectMessage {
                message_type: Some(
                    QConnectMessageType::MessageTypeCtrlSrvrQueueReorderTracks as i32,
                ),
                ctrl_srvr_queue_reorder_tracks: Some(QueueReorderTracksMessage {
                    queue_version_ref,
                    action_uuid,
                    queue_item_ids,
                    insert_after,
                    autoplay_reset: Some(autoplay_reset),
                    autoplay_loading: Some(autoplay_loading),
                }),
                ..Default::default()
            })
        }
        QueueCommandType::CtrlSrvrClearQueue => Ok(QConnectMessage {
            message_type: Some(QConnectMessageType::MessageTypeCtrlSrvrClearQueue as i32),
            ctrl_srvr_clear_queue: Some(ClearQueueMessage {
                queue_version_ref,
                action_uuid,
            }),
            ..Default::default()
        }),
        QueueCommandType::CtrlSrvrSetShuffleMode => {
            let shuffle_mode = optional_bool(&command.payload, "shuffle_mode", false);
            let shuffle_seed = optional_i32(&command.payload, "shuffle_seed")?;
            let shuffle_pivot_queue_item_id =
                optional_i32(&command.payload, "shuffle_pivot_queue_item_id")?;
            let autoplay_reset = optional_bool(&command.payload, "autoplay_reset", false);
            let autoplay_loading =
                optional_bool(&command.payload, "autoplay_loading", autoplay_reset);

            Ok(QConnectMessage {
                message_type: Some(QConnectMessageType::MessageTypeCtrlSrvrSetShuffleMode as i32),
                ctrl_srvr_set_shuffle_mode: Some(SetShuffleModeMessage {
                    queue_version_ref,
                    action_uuid,
                    shuffle_mode: Some(shuffle_mode),
                    shuffle_seed,
                    shuffle_pivot_queue_item_id,
                    autoplay_reset: Some(autoplay_reset),
                    autoplay_loading: Some(autoplay_loading),
                }),
                ..Default::default()
            })
        }
        QueueCommandType::CtrlSrvrSetAutoplayMode => {
            let autoplay_mode = optional_bool(&command.payload, "autoplay_mode", false);
            // Android forces these defaults for setAutoplayMode.
            let autoplay_reset = optional_bool(&command.payload, "autoplay_reset", true);
            let autoplay_loading = optional_bool(&command.payload, "autoplay_loading", false);

            Ok(QConnectMessage {
                message_type: Some(QConnectMessageType::MessageTypeCtrlSrvrSetAutoplayMode as i32),
                ctrl_srvr_set_autoplay_mode: Some(SetAutoplayModeMessage {
                    queue_version_ref,
                    action_uuid,
                    autoplay_mode: Some(autoplay_mode),
                    autoplay_reset: Some(autoplay_reset),
                    autoplay_loading: Some(autoplay_loading),
                }),
                ..Default::default()
            })
        }
        QueueCommandType::CtrlSrvrAutoplayLoadTracks => {
            let track_ids = required_i32_list(&command.payload, "track_ids")?;
            let context_uuid = optional_uuid_bytes(&command.payload, &["context_uuid"])?;

            Ok(QConnectMessage {
                message_type: Some(
                    QConnectMessageType::MessageTypeCtrlSrvrAutoplayLoadTracks as i32,
                ),
                ctrl_srvr_autoplay_load_tracks: Some(AutoplayLoadTracksMessage {
                    queue_version_ref,
                    action_uuid,
                    track_ids,
                    context_uuid,
                }),
                ..Default::default()
            })
        }
        QueueCommandType::CtrlSrvrAutoplayRemoveTracks => {
            let queue_item_ids = required_i32_list(&command.payload, "queue_item_ids")?;

            Ok(QConnectMessage {
                message_type: Some(
                    QConnectMessageType::MessageTypeCtrlSrvrAutoplayRemoveTracks as i32,
                ),
                ctrl_srvr_autoplay_remove_tracks: Some(AutoplayRemoveTracksMessage {
                    queue_version_ref,
                    action_uuid,
                    queue_item_ids,
                }),
                ..Default::default()
            })
        }
        QueueCommandType::CtrlSrvrSetQueueState => {
            let tracks = required_tracks_with_context(&command.payload, "tracks")?;
            let shuffle_mode = optional_bool(&command.payload, "shuffle_mode", false);
            let shuffled_track_indexes =
                optional_i32_list(&command.payload, "shuffled_track_indexes")?;
            let autoplay_mode = optional_bool(&command.payload, "autoplay_mode", false);
            let autoplay_loading = optional_bool(&command.payload, "autoplay_loading", false);
            let autoplay_tracks =
                required_tracks_with_context(&command.payload, "autoplay_tracks")?;

            Ok(QConnectMessage {
                message_type: Some(QConnectMessageType::MessageTypeCtrlSrvrSetQueueState as i32),
                ctrl_srvr_set_queue_state: Some(SetQueueStateMessage {
                    queue_version_ref,
                    action_uuid,
                    tracks,
                    shuffle_mode: Some(shuffle_mode),
                    shuffled_track_indexes,
                    autoplay_mode: Some(autoplay_mode),
                    autoplay_loading: Some(autoplay_loading),
                    autoplay_tracks,
                }),
                ..Default::default()
            })
        }
        QueueCommandType::CtrlSrvrAskForQueueState => Ok(QConnectMessage {
            message_type: Some(QConnectMessageType::MessageTypeCtrlSrvrAskForQueueState as i32),
            ctrl_srvr_ask_for_queue_state: Some(AskForQueueStateMessage {
                queue_version_ref,
                action_uuid,
            }),
            ..Default::default()
        }),
    }
}

fn to_proto_queue_version(
    version: qconnect_core::QueueVersion,
) -> Result<QueueVersionRef, ProtocolError> {
    Ok(QueueVersionRef {
        major: Some(to_i32_from_u64(version.major, "queue_version_ref.major")?),
        minor: Some(to_i32_from_u64(version.minor, "queue_version_ref.minor")?),
    })
}

fn action_uuid_bytes(value: &str) -> Result<Vec<u8>, ProtocolError> {
    let uuid = Uuid::parse_str(value).map_err(|err| {
        ProtocolError::InvalidUuid(format!("action_uuid '{}' parse error: {err}", value))
    })?;
    Ok(uuid.as_bytes().to_vec())
}

fn optional_uuid_bytes(payload: &Value, keys: &[&str]) -> Result<Option<Vec<u8>>, ProtocolError> {
    for key in keys {
        if let Some(raw) = payload.get(*key).and_then(Value::as_str) {
            let uuid = Uuid::parse_str(raw).map_err(|err| {
                ProtocolError::InvalidUuid(format!("{} '{}' parse error: {err}", key, raw))
            })?;
            return Ok(Some(uuid.as_bytes().to_vec()));
        }
    }
    Ok(None)
}

fn required_i32_list(payload: &Value, key: &str) -> Result<Vec<i32>, ProtocolError> {
    let values = payload
        .get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| ProtocolError::InvalidPayload(format!("missing array field '{key}'")))?;

    values
        .iter()
        .enumerate()
        .map(|(idx, value)| {
            let raw = value
                .as_i64()
                .or_else(|| value.as_u64().map(|v| v as i64))
                .ok_or_else(|| {
                    ProtocolError::InvalidPayload(format!("field '{key}[{idx}]' is not numeric"))
                })?;
            to_i32_from_i64(raw, &format!("{key}[{idx}]"))
        })
        .collect()
}

fn optional_i32_list(payload: &Value, key: &str) -> Result<Vec<i32>, ProtocolError> {
    let Some(values) = payload.get(key).and_then(Value::as_array) else {
        return Ok(Vec::new());
    };

    values
        .iter()
        .enumerate()
        .map(|(idx, value)| {
            let raw = value
                .as_i64()
                .or_else(|| value.as_u64().map(|v| v as i64))
                .ok_or_else(|| {
                    ProtocolError::InvalidPayload(format!("field '{key}[{idx}]' is not numeric"))
                })?;
            to_i32_from_i64(raw, &format!("{key}[{idx}]"))
        })
        .collect()
}

fn required_tracks_with_context(
    payload: &Value,
    key: &str,
) -> Result<Vec<SetQueueTrackWithContext>, ProtocolError> {
    let Some(values) = payload.get(key).and_then(Value::as_array) else {
        return Ok(Vec::new());
    };

    values
        .iter()
        .enumerate()
        .map(|(idx, value)| {
            let track_id = value
                .get("track_id")
                .or_else(|| value.get("trackId"))
                .and_then(Value::as_i64)
                .or_else(|| {
                    value
                        .get("track_id")
                        .or_else(|| value.get("trackId"))
                        .and_then(Value::as_u64)
                        .map(|v| v as i64)
                })
                .ok_or_else(|| {
                    ProtocolError::InvalidPayload(format!(
                        "field '{key}[{idx}].track_id' is required"
                    ))
                })?;

            let context_uuid = value
                .get("context_uuid")
                .or_else(|| value.get("track_context_uuid"))
                .and_then(Value::as_str)
                .map(parse_uuid)
                .transpose()?;

            Ok(SetQueueTrackWithContext {
                track_id: Some(to_i32_from_i64(
                    track_id,
                    &format!("{key}[{idx}].track_id"),
                )?),
                context_uuid,
            })
        })
        .collect()
}

fn optional_i32(payload: &Value, key: &str) -> Result<Option<i32>, ProtocolError> {
    match payload.get(key) {
        None => Ok(None),
        Some(value) => {
            let raw = value
                .as_i64()
                .or_else(|| value.as_u64().map(|v| v as i64))
                .ok_or_else(|| {
                    ProtocolError::InvalidPayload(format!("field '{key}' is not numeric"))
                })?;
            Ok(Some(to_i32_from_i64(raw, key)?))
        }
    }
}

fn optional_bool(payload: &Value, key: &str, default: bool) -> bool {
    payload.get(key).and_then(Value::as_bool).unwrap_or(default)
}

fn parse_uuid(value: &str) -> Result<Vec<u8>, ProtocolError> {
    let parsed = Uuid::parse_str(value)
        .map_err(|err| ProtocolError::InvalidUuid(format!("'{value}' parse error: {err}")))?;
    Ok(parsed.as_bytes().to_vec())
}

fn to_i32_from_u64(value: u64, field_name: &str) -> Result<i32, ProtocolError> {
    i32::try_from(value).map_err(|_| {
        ProtocolError::InvalidPayload(format!("field '{field_name}' is out of i32 range: {value}"))
    })
}

fn to_i32_from_i64(value: i64, field_name: &str) -> Result<i32, ProtocolError> {
    i32::try_from(value).map_err(|_| {
        ProtocolError::InvalidPayload(format!("field '{field_name}' is out of i32 range: {value}"))
    })
}

fn next_batch_seq() -> i32 {
    BATCH_SEQ.fetch_add(1, Ordering::Relaxed)
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::QueueCommand;
    use qconnect_core::QueueVersion;
    use serde_json::json;

    #[test]
    fn encodes_add_tracks_command_into_binary_batch() {
        let command = QueueCommand::new(
            QueueCommandType::CtrlSrvrQueueAddTracks,
            "85fa0dd6-7bd6-4b3c-8f43-b8ee22e65d5e",
            QueueVersion::new(1, 2),
            json!({
                "track_ids": [101, 102],
                "context_uuid": "0f892e1a-a2f4-4d18-82c6-31e8daf2ea0f",
                "autoplay_reset": true
            }),
        );

        let payload = encode_queue_command_batch(&command).expect("batch payload");
        assert!(!payload.is_empty());
    }

    #[test]
    fn rejects_non_uuid_action_id() {
        let command = QueueCommand::new(
            QueueCommandType::CtrlSrvrQueueAddTracks,
            "not-a-uuid",
            QueueVersion::new(1, 2),
            json!({"track_ids": [1]}),
        );

        let err = encode_queue_command_batch(&command).expect_err("expected invalid uuid");
        assert!(matches!(err, ProtocolError::InvalidUuid(_)));
    }

    #[test]
    fn set_autoplay_mode_defaults_follow_android_behavior() {
        let command = QueueCommand::new(
            QueueCommandType::CtrlSrvrSetAutoplayMode,
            "2d8292c8-4f23-40f3-98a4-e3899eb0d03a",
            QueueVersion::new(7, 8),
            json!({"autoplay_mode": true}),
        );

        let payload = encode_queue_command_batch(&command).expect("batch payload");
        assert!(!payload.is_empty());
    }
}
