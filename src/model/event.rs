//! All the events this library handles.
//!
//! Every event includes the gateway intent required to receive it, as well as a link to the
//! Discord documentation for the event.

use serde::Serialize;
use serde::de::Error as DeError;
use serde_json::value::RawValue;
use crate::constants::Opcode;
use crate::model::prelude::*;

/// The "Ready" event, containing initial ready cache
///
/// Requires no gateway intents.
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#ready).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct ReadyEvent {
    pub ready: Ready,
}

/// Requires no gateway intents.
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#resumed).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ResumedEvent {}

/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#payload-structure).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Debug, Clone, Serialize)]
#[non_exhaustive]
#[serde(untagged)]
pub enum GatewayEvent {
    Dispatch {
        seq: u64,
        event: Box<IEvent>,
    },
    Heartbeat,
    Reconnect,
    /// Whether the session can be resumed.
    InvalidateSession(bool),
    Hello(u64),
    HeartbeatAck,
}

#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct IEvent {
    #[serde(rename = "t")]
    pub ty: String,
    #[serde(rename = "d")]
    #[cfg_attr(feature = "typesize", typesize(with = raw_value_len))]
    pub data: Box<RawValue>,
}

#[cfg(feature = "typesize")]
fn raw_value_len(val: &RawValue) -> usize {
    val.get().len()
}

// Manual impl needed to emulate integer enum tags
impl<'de> Deserialize<'de> for GatewayEvent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        #[derive(Deserialize)]
        struct GatewayEventRaw {
            op: Opcode,
            #[serde(rename = "s")]
            seq: Option<u64>,
            #[serde(rename = "d")]
            data: Box<RawValue>,
            #[serde(rename = "t")]
            ty: Option<String>,
        }

        let raw_data = <&RawValue>::deserialize(deserializer)?;

        let raw = GatewayEventRaw::deserialize(raw_data).map_err(DeError::custom)?;

        Ok(match raw.op {
            Opcode::Dispatch => {
                if raw.ty.is_none() {
                    return Err(DeError::missing_field("t"));
                }

                Self::Dispatch {
                    seq: raw.seq.ok_or_else(|| DeError::missing_field("s"))?,
                    event: Box::new(IEvent {
                        ty: raw.ty.ok_or_else(|| DeError::missing_field("t"))?.to_string(),
                        data: raw.data,
                    }),
                }
            },
            Opcode::Heartbeat => Self::Heartbeat,
            Opcode::InvalidSession => {
                Self::InvalidateSession(bool::deserialize(&*raw.data).map_err(DeError::custom)?)
            },
            Opcode::Hello => {
                #[derive(Deserialize)]
                struct HelloPayload {
                    heartbeat_interval: u64,
                }

                let inner = HelloPayload::deserialize(&*raw.data).map_err(DeError::custom)?;

                Self::Hello(inner.heartbeat_interval)
            },
            Opcode::Reconnect => Self::Reconnect,
            Opcode::HeartbeatAck => Self::HeartbeatAck,
            _ => return Err(DeError::custom("invalid opcode")),
        })
    }
}
