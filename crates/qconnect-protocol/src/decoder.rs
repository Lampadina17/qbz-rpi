use prost::Message;
use qconnect_core::QueueVersion;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::{
    queue_command_proto::{
        AutoplayModeSetMessage, AutoplayTracksLoadedMessage, AutoplayTracksRemovedMessage,
        QConnectMessage, QConnectMessageType, QConnectMessages, QueueClearedMessage,
        QueueErrorMessage, QueueStateMessage, QueueTrack, QueueTrackWithContext,
        QueueTracksAddedFromAutoplayMessage, QueueTracksAddedMessage, QueueTracksInsertedMessage,
        QueueTracksLoadedMessage, QueueTracksRemovedMessage, QueueTracksReorderedMessage,
        QueueVersionRef, ShuffleModeSetMessage,
    },
    ProtocolError, QueueEventType, QueueServerEvent,
};

pub fn decode_queue_server_events(payload: &[u8]) -> Result<Vec<QueueServerEvent>, ProtocolError> {
    let batch = QConnectMessages::decode(payload)?;
    let mut events = Vec::new();

    for message in batch.messages {
        if let Some(event) = decode_queue_server_event(message)? {
            events.push(event);
        }
    }

    Ok(events)
}

fn decode_queue_server_event(
    message: QConnectMessage,
) -> Result<Option<QueueServerEvent>, ProtocolError> {
    let message_type = resolve_message_type(&message);
    let Some(message_type) = message_type else {
        return Ok(None);
    };

    let event = match message_type {
        code if code == QConnectMessageType::MessageTypeSrvrCtrlQueueErrorMessage as i32 => {
            let Some(payload) = message.srvr_ctrl_queue_error_message else {
                return Ok(None);
            };
            map_queue_error(payload)?
        }
        code if code == QConnectMessageType::MessageTypeSrvrCtrlQueueCleared as i32 => {
            let Some(payload) = message.srvr_ctrl_queue_cleared else {
                return Ok(None);
            };
            map_queue_cleared(payload)?
        }
        code if code == QConnectMessageType::MessageTypeSrvrCtrlQueueState as i32 => {
            let Some(payload) = message.srvr_ctrl_queue_state else {
                return Ok(None);
            };
            map_queue_state(payload)?
        }
        code if code == QConnectMessageType::MessageTypeSrvrCtrlQueueTracksLoaded as i32 => {
            let Some(payload) = message.srvr_ctrl_queue_tracks_loaded else {
                return Ok(None);
            };
            map_tracks_loaded(payload)?
        }
        code if code == QConnectMessageType::MessageTypeSrvrCtrlQueueTracksInserted as i32 => {
            let Some(payload) = message.srvr_ctrl_queue_tracks_inserted else {
                return Ok(None);
            };
            map_tracks_inserted(payload)?
        }
        code if code == QConnectMessageType::MessageTypeSrvrCtrlQueueTracksAdded as i32 => {
            let Some(payload) = message.srvr_ctrl_queue_tracks_added else {
                return Ok(None);
            };
            map_tracks_added(payload)?
        }
        code if code == QConnectMessageType::MessageTypeSrvrCtrlQueueTracksRemoved as i32 => {
            let Some(payload) = message.srvr_ctrl_queue_tracks_removed else {
                return Ok(None);
            };
            map_tracks_removed(payload)?
        }
        code if code == QConnectMessageType::MessageTypeSrvrCtrlQueueTracksReordered as i32 => {
            let Some(payload) = message.srvr_ctrl_queue_tracks_reordered else {
                return Ok(None);
            };
            map_tracks_reordered(payload)?
        }
        code if code == QConnectMessageType::MessageTypeSrvrCtrlShuffleModeSet as i32 => {
            let Some(payload) = message.srvr_ctrl_shuffle_mode_set else {
                return Ok(None);
            };
            map_shuffle_mode_set(payload)?
        }
        code if code == QConnectMessageType::MessageTypeSrvrCtrlAutoplayModeSet as i32 => {
            let Some(payload) = message.srvr_ctrl_autoplay_mode_set else {
                return Ok(None);
            };
            map_autoplay_mode_set(payload)?
        }
        code if code == QConnectMessageType::MessageTypeSrvrCtrlAutoplayTracksLoaded as i32 => {
            let Some(payload) = message.srvr_ctrl_autoplay_tracks_loaded else {
                return Ok(None);
            };
            map_autoplay_tracks_loaded(payload)?
        }
        code if code == QConnectMessageType::MessageTypeSrvrCtrlAutoplayTracksRemoved as i32 => {
            let Some(payload) = message.srvr_ctrl_autoplay_tracks_removed else {
                return Ok(None);
            };
            map_autoplay_tracks_removed(payload)?
        }
        code if code
            == QConnectMessageType::MessageTypeSrvrCtrlQueueTracksAddedFromAutoplay as i32 =>
        {
            let Some(payload) = message.srvr_ctrl_queue_tracks_added_from_autoplay else {
                return Ok(None);
            };
            map_tracks_added_from_autoplay(payload)?
        }
        _ => return Ok(None),
    };

    Ok(Some(event))
}

fn resolve_message_type(message: &QConnectMessage) -> Option<i32> {
    message.message_type.or_else(|| {
        if message.srvr_ctrl_queue_error_message.is_some() {
            return Some(QConnectMessageType::MessageTypeSrvrCtrlQueueErrorMessage as i32);
        }
        if message.srvr_ctrl_queue_cleared.is_some() {
            return Some(QConnectMessageType::MessageTypeSrvrCtrlQueueCleared as i32);
        }
        if message.srvr_ctrl_queue_state.is_some() {
            return Some(QConnectMessageType::MessageTypeSrvrCtrlQueueState as i32);
        }
        if message.srvr_ctrl_queue_tracks_loaded.is_some() {
            return Some(QConnectMessageType::MessageTypeSrvrCtrlQueueTracksLoaded as i32);
        }
        if message.srvr_ctrl_queue_tracks_inserted.is_some() {
            return Some(QConnectMessageType::MessageTypeSrvrCtrlQueueTracksInserted as i32);
        }
        if message.srvr_ctrl_queue_tracks_added.is_some() {
            return Some(QConnectMessageType::MessageTypeSrvrCtrlQueueTracksAdded as i32);
        }
        if message.srvr_ctrl_queue_tracks_removed.is_some() {
            return Some(QConnectMessageType::MessageTypeSrvrCtrlQueueTracksRemoved as i32);
        }
        if message.srvr_ctrl_queue_tracks_reordered.is_some() {
            return Some(QConnectMessageType::MessageTypeSrvrCtrlQueueTracksReordered as i32);
        }
        if message.srvr_ctrl_shuffle_mode_set.is_some() {
            return Some(QConnectMessageType::MessageTypeSrvrCtrlShuffleModeSet as i32);
        }
        if message.srvr_ctrl_autoplay_mode_set.is_some() {
            return Some(QConnectMessageType::MessageTypeSrvrCtrlAutoplayModeSet as i32);
        }
        if message.srvr_ctrl_autoplay_tracks_loaded.is_some() {
            return Some(QConnectMessageType::MessageTypeSrvrCtrlAutoplayTracksLoaded as i32);
        }
        if message.srvr_ctrl_autoplay_tracks_removed.is_some() {
            return Some(QConnectMessageType::MessageTypeSrvrCtrlAutoplayTracksRemoved as i32);
        }
        if message.srvr_ctrl_queue_tracks_added_from_autoplay.is_some() {
            return Some(
                QConnectMessageType::MessageTypeSrvrCtrlQueueTracksAddedFromAutoplay as i32,
            );
        }
        None
    })
}

fn map_queue_error(payload: QueueErrorMessage) -> Result<QueueServerEvent, ProtocolError> {
    let error_code = payload
        .error
        .as_ref()
        .and_then(|err| err.code)
        .map(|code| code.to_string())
        .unwrap_or_else(|| "remote_error".to_string());
    let error_message = payload
        .error
        .as_ref()
        .and_then(|err| err.message.clone())
        .unwrap_or_else(|| "queue_error_message".to_string());

    Ok(QueueServerEvent {
        event_type: QueueEventType::SrvrCtrlQueueErrorMessage,
        action_uuid: uuid_bytes_to_string_opt(payload.action_uuid, "queue_error.action_uuid")?,
        queue_version: queue_version_opt(payload.queue_version)?,
        payload: json!({
            "error_code": error_code,
            "error_message": error_message
        }),
    })
}

fn map_queue_cleared(payload: QueueClearedMessage) -> Result<QueueServerEvent, ProtocolError> {
    Ok(QueueServerEvent {
        event_type: QueueEventType::SrvrCtrlQueueCleared,
        action_uuid: uuid_bytes_to_string_opt(payload.action_uuid, "queue_cleared.action_uuid")?,
        queue_version: queue_version_opt(payload.queue_version)?,
        payload: json!({}),
    })
}

fn map_queue_state(payload: QueueStateMessage) -> Result<QueueServerEvent, ProtocolError> {
    let tracks = payload
        .tracks
        .into_iter()
        .map(queue_track_with_context_to_json)
        .collect::<Result<Vec<_>, _>>()?;
    let autoplay_tracks = payload
        .autoplay_tracks
        .into_iter()
        .map(queue_track_with_context_to_json)
        .collect::<Result<Vec<_>, _>>()?;

    let shuffled_indexes = payload
        .shuffled_track_indexes
        .into_iter()
        .map(i32_to_u64)
        .collect::<Result<Vec<_>, _>>()?;

    Ok(QueueServerEvent {
        event_type: QueueEventType::SrvrCtrlQueueState,
        action_uuid: uuid_bytes_to_string_opt(payload.action_uuid, "queue_state.action_uuid")?,
        queue_version: queue_version_opt(payload.queue_version)?,
        payload: json!({
            "tracks": tracks,
            "shuffle_mode": payload.shuffle_mode.unwrap_or(false),
            "shuffled_track_indexes": shuffled_indexes,
            "autoplay_mode": payload.autoplay_mode.unwrap_or(false),
            "autoplay_loading": payload.autoplay_loading.unwrap_or(false),
            "autoplay_tracks": autoplay_tracks
        }),
    })
}

fn map_tracks_loaded(payload: QueueTracksLoadedMessage) -> Result<QueueServerEvent, ProtocolError> {
    let context_uuid =
        uuid_bytes_to_string_opt(payload.context_uuid, "tracks_loaded.context_uuid")?;
    let tracks = payload
        .tracks
        .into_iter()
        .map(|track| queue_track_to_json(track, context_uuid.as_deref()))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(QueueServerEvent {
        event_type: QueueEventType::SrvrCtrlQueueTracksLoaded,
        action_uuid: uuid_bytes_to_string_opt(payload.action_uuid, "tracks_loaded.action_uuid")?,
        queue_version: queue_version_opt(payload.queue_version)?,
        payload: json!({
            "tracks": tracks,
            "queue_position": optional_i32_to_u64(payload.queue_position)?,
            "shuffle_seed": optional_i32_to_u64(payload.shuffle_seed)?,
            "shuffle_pivot_queue_item_id": optional_i32_to_u64(payload.shuffle_pivot_queue_item_id)?,
            "shuffle_mode": payload.shuffle_mode.unwrap_or(false),
            "autoplay_reset": payload.autoplay_reset.unwrap_or(false),
            "autoplay_loading": payload.autoplay_loading.unwrap_or(false)
        }),
    })
}

fn map_tracks_inserted(
    payload: QueueTracksInsertedMessage,
) -> Result<QueueServerEvent, ProtocolError> {
    let context_uuid =
        uuid_bytes_to_string_opt(payload.context_uuid, "tracks_inserted.context_uuid")?;
    let tracks = payload
        .tracks
        .into_iter()
        .map(|track| queue_track_to_json(track, context_uuid.as_deref()))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(QueueServerEvent {
        event_type: QueueEventType::SrvrCtrlQueueTracksInserted,
        action_uuid: uuid_bytes_to_string_opt(payload.action_uuid, "tracks_inserted.action_uuid")?,
        queue_version: queue_version_opt(payload.queue_version)?,
        payload: json!({
            "tracks": tracks,
            "insert_after": optional_i32_to_u64(payload.insert_after)?,
            "shuffle_seed": optional_i32_to_u64(payload.shuffle_seed)?,
            "autoplay_reset": payload.autoplay_reset.unwrap_or(false),
            "autoplay_loading": payload.autoplay_loading.unwrap_or(false)
        }),
    })
}

fn map_tracks_added(payload: QueueTracksAddedMessage) -> Result<QueueServerEvent, ProtocolError> {
    let context_uuid = uuid_bytes_to_string_opt(payload.context_uuid, "tracks_added.context_uuid")?;
    let tracks = payload
        .tracks
        .into_iter()
        .map(|track| queue_track_to_json(track, context_uuid.as_deref()))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(QueueServerEvent {
        event_type: QueueEventType::SrvrCtrlQueueTracksAdded,
        action_uuid: uuid_bytes_to_string_opt(payload.action_uuid, "tracks_added.action_uuid")?,
        queue_version: queue_version_opt(payload.queue_version)?,
        payload: json!({
            "tracks": tracks,
            "shuffle_seed": optional_i32_to_u64(payload.shuffle_seed)?,
            "autoplay_reset": payload.autoplay_reset.unwrap_or(false),
            "autoplay_loading": payload.autoplay_loading.unwrap_or(false)
        }),
    })
}

fn map_tracks_removed(
    payload: QueueTracksRemovedMessage,
) -> Result<QueueServerEvent, ProtocolError> {
    let queue_item_ids = payload
        .queue_item_ids
        .into_iter()
        .map(i32_to_u64)
        .collect::<Result<Vec<_>, _>>()?;

    Ok(QueueServerEvent {
        event_type: QueueEventType::SrvrCtrlQueueTracksRemoved,
        action_uuid: uuid_bytes_to_string_opt(payload.action_uuid, "tracks_removed.action_uuid")?,
        queue_version: queue_version_opt(payload.queue_version)?,
        payload: json!({
            "queue_item_ids": queue_item_ids,
            "autoplay_reset": payload.autoplay_reset.unwrap_or(false),
            "autoplay_loading": payload.autoplay_loading.unwrap_or(false)
        }),
    })
}

fn map_tracks_reordered(
    payload: QueueTracksReorderedMessage,
) -> Result<QueueServerEvent, ProtocolError> {
    let queue_item_ids = payload
        .queue_item_ids
        .into_iter()
        .map(i32_to_u64)
        .collect::<Result<Vec<_>, _>>()?;

    Ok(QueueServerEvent {
        event_type: QueueEventType::SrvrCtrlQueueTracksReordered,
        action_uuid: uuid_bytes_to_string_opt(payload.action_uuid, "tracks_reordered.action_uuid")?,
        queue_version: queue_version_opt(payload.queue_version)?,
        payload: json!({
            "queue_item_ids": queue_item_ids,
            "insert_after": optional_i32_to_u64(payload.insert_after)?,
            "autoplay_reset": payload.autoplay_reset.unwrap_or(false),
            "autoplay_loading": payload.autoplay_loading.unwrap_or(false)
        }),
    })
}

fn map_shuffle_mode_set(payload: ShuffleModeSetMessage) -> Result<QueueServerEvent, ProtocolError> {
    Ok(QueueServerEvent {
        event_type: QueueEventType::SrvrCtrlShuffleModeSet,
        action_uuid: uuid_bytes_to_string_opt(payload.action_uuid, "shuffle_mode_set.action_uuid")?,
        queue_version: queue_version_opt(payload.queue_version)?,
        payload: json!({
            "shuffle_mode": payload.shuffle_mode.unwrap_or(false),
            "shuffle_seed": optional_i32_to_u64(payload.shuffle_seed)?,
            "shuffle_pivot_queue_item_id": optional_i32_to_u64(payload.shuffle_pivot_queue_item_id)?,
            "autoplay_reset": payload.autoplay_reset.unwrap_or(false),
            "autoplay_loading": payload.autoplay_loading.unwrap_or(false)
        }),
    })
}

fn map_autoplay_mode_set(
    payload: AutoplayModeSetMessage,
) -> Result<QueueServerEvent, ProtocolError> {
    Ok(QueueServerEvent {
        event_type: QueueEventType::SrvrCtrlAutoplayModeSet,
        action_uuid: uuid_bytes_to_string_opt(
            payload.action_uuid,
            "autoplay_mode_set.action_uuid",
        )?,
        queue_version: queue_version_opt(payload.queue_version)?,
        payload: json!({
            "autoplay_mode": payload.autoplay_mode.unwrap_or(false),
            "autoplay_reset": payload.autoplay_reset.unwrap_or(false),
            "autoplay_loading": payload.autoplay_loading.unwrap_or(false)
        }),
    })
}

fn map_autoplay_tracks_loaded(
    payload: AutoplayTracksLoadedMessage,
) -> Result<QueueServerEvent, ProtocolError> {
    let context_uuid =
        uuid_bytes_to_string_opt(payload.context_uuid, "autoplay_tracks_loaded.context_uuid")?;
    let tracks = payload
        .tracks
        .into_iter()
        .map(|track| queue_track_to_json(track, context_uuid.as_deref()))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(QueueServerEvent {
        event_type: QueueEventType::SrvrCtrlAutoplayTracksLoaded,
        action_uuid: uuid_bytes_to_string_opt(
            payload.action_uuid,
            "autoplay_tracks_loaded.action_uuid",
        )?,
        queue_version: queue_version_opt(payload.queue_version)?,
        payload: json!({
            "tracks": tracks
        }),
    })
}

fn map_autoplay_tracks_removed(
    payload: AutoplayTracksRemovedMessage,
) -> Result<QueueServerEvent, ProtocolError> {
    let queue_item_ids = payload
        .queue_item_ids
        .into_iter()
        .map(i32_to_u64)
        .collect::<Result<Vec<_>, _>>()?;

    Ok(QueueServerEvent {
        event_type: QueueEventType::SrvrCtrlAutoplayTracksRemoved,
        action_uuid: uuid_bytes_to_string_opt(
            payload.action_uuid,
            "autoplay_tracks_removed.action_uuid",
        )?,
        queue_version: queue_version_opt(payload.queue_version)?,
        payload: json!({
            "queue_item_ids": queue_item_ids
        }),
    })
}

fn map_tracks_added_from_autoplay(
    payload: QueueTracksAddedFromAutoplayMessage,
) -> Result<QueueServerEvent, ProtocolError> {
    let queue_item_ids = payload
        .queue_item_ids
        .into_iter()
        .map(i32_to_u64)
        .collect::<Result<Vec<_>, _>>()?;

    Ok(QueueServerEvent {
        event_type: QueueEventType::SrvrCtrlQueueTracksAddedFromAutoplay,
        action_uuid: None,
        queue_version: queue_version_opt(payload.queue_version)?,
        payload: json!({
            "queue_item_ids": queue_item_ids
        }),
    })
}

fn queue_track_to_json(
    track: QueueTrack,
    context_uuid: Option<&str>,
) -> Result<Value, ProtocolError> {
    let track_id = required_i32_to_u64(track.track_id, "queue_track.track_id")?;
    let queue_item_id =
        optional_i32_to_u64_named(track.queue_item_id, "queue_track.queue_item_id")?
            .unwrap_or(track_id);

    Ok(json!({
        "track_context_uuid": context_uuid.unwrap_or_default(),
        "track_id": track_id,
        "queue_item_id": queue_item_id
    }))
}

fn queue_track_with_context_to_json(track: QueueTrackWithContext) -> Result<Value, ProtocolError> {
    let track_id = required_i32_to_u64(track.track_id, "queue_track_with_context.track_id")?;
    let queue_item_id = optional_i32_to_u64_named(
        track.queue_item_id,
        "queue_track_with_context.queue_item_id",
    )?
    .unwrap_or(track_id);
    let context_uuid =
        uuid_bytes_to_string_opt(track.context_uuid, "queue_track_with_context.context_uuid")?
            .unwrap_or_default();

    Ok(json!({
        "track_context_uuid": context_uuid,
        "track_id": track_id,
        "queue_item_id": queue_item_id
    }))
}

fn queue_version_opt(
    value: Option<QueueVersionRef>,
) -> Result<Option<QueueVersion>, ProtocolError> {
    let Some(version) = value else {
        return Ok(None);
    };

    let major = required_i32_to_u64(version.major, "queue_version.major")?;
    let minor = required_i32_to_u64(version.minor, "queue_version.minor")?;
    Ok(Some(QueueVersion::new(major, minor)))
}

fn required_i32_to_u64(value: Option<i32>, field_name: &str) -> Result<u64, ProtocolError> {
    let raw = value.ok_or_else(|| {
        ProtocolError::InvalidPayload(format!("missing required numeric field '{field_name}'"))
    })?;
    i32_to_u64(raw)
}

fn optional_i32_to_u64_named(
    value: Option<i32>,
    field_name: &str,
) -> Result<Option<u64>, ProtocolError> {
    value
        .map(|raw| {
            if raw < 0 {
                return Err(ProtocolError::InvalidPayload(format!(
                    "negative value where unsigned expected in '{field_name}': {raw}"
                )));
            }
            Ok(raw as u64)
        })
        .transpose()
}

fn optional_i32_to_u64(value: Option<i32>) -> Result<Option<u64>, ProtocolError> {
    value.map(i32_to_u64).transpose()
}

fn i32_to_u64(value: i32) -> Result<u64, ProtocolError> {
    if value < 0 {
        return Err(ProtocolError::InvalidPayload(format!(
            "negative value where unsigned expected: {value}"
        )));
    }
    Ok(value as u64)
}

fn uuid_bytes_to_string_opt(
    value: Option<Vec<u8>>,
    field_name: &str,
) -> Result<Option<String>, ProtocolError> {
    let Some(bytes) = value else {
        return Ok(None);
    };
    let uuid = Uuid::from_slice(&bytes).map_err(|err| {
        ProtocolError::InvalidUuid(format!("{field_name} invalid UUID bytes: {err}"))
    })?;
    Ok(Some(uuid.to_string()))
}

#[cfg(test)]
mod tests {
    use prost::Message;

    use crate::queue_command_proto::{
        QConnectMessage, QConnectMessageType, QConnectMessages, QueueTrack,
        QueueTracksAddedMessage, QueueVersionRef,
    };

    use super::decode_queue_server_events;

    #[test]
    fn decodes_tracks_added_server_event_batch() {
        let message = QConnectMessage {
            message_type: Some(QConnectMessageType::MessageTypeSrvrCtrlQueueTracksAdded as i32),
            srvr_ctrl_queue_tracks_added: Some(QueueTracksAddedMessage {
                queue_version: Some(QueueVersionRef {
                    major: Some(3),
                    minor: Some(4),
                }),
                action_uuid: Some(
                    uuid::Uuid::parse_str("85fa0dd6-7bd6-4b3c-8f43-b8ee22e65d5e")
                        .expect("uuid")
                        .as_bytes()
                        .to_vec(),
                ),
                tracks: vec![QueueTrack {
                    queue_item_id: Some(44),
                    track_id: Some(555),
                }],
                shuffle_seed: Some(99),
                context_uuid: Some(
                    uuid::Uuid::parse_str("0f892e1a-a2f4-4d18-82c6-31e8daf2ea0f")
                        .expect("context uuid")
                        .as_bytes()
                        .to_vec(),
                ),
                autoplay_reset: Some(false),
                autoplay_loading: Some(false),
                queue_hash: None,
            }),
            ..Default::default()
        };

        let batch = QConnectMessages {
            messages_time: Some(1),
            messages_id: Some(1),
            messages: vec![message],
        };
        let encoded = batch.encode_to_vec();

        let events = decode_queue_server_events(&encoded).expect("decode events");
        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0].message_type(),
            "MESSAGE_TYPE_SRVR_CTRL_QUEUE_TRACKS_ADDED"
        );
    }
}
