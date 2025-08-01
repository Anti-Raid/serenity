//! Auto moderation types
//!
//! [Discord docs](https://discord.com/developers/docs/resources/auto-moderation)

use serde::{Deserialize, Serialize};

use crate::model::prelude::*;

/// Configured auto moderation rule.
///
/// [Discord docs](https://discord.com/developers/docs/resources/auto-moderation#auto-moderation-rule-object).

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub struct AutoModRule {
    /// ID of the rule.
    pub id: RuleId,
    /// ID of the guild this rule belongs to.
    pub guild_id: GuildId,
    /// Name of the rule.
    pub name: FixedString,
    /// ID of the user which created the rule.
    pub creator_id: UserId,
    /// Event context in which the rule should be checked.
    pub event_type: EventType,
    /// Characterizes the type of content which can trigger the rule.
    #[serde(flatten)]
    pub trigger: Trigger,
    /// Actions which will execute when the rule is triggered.
    pub actions: FixedArray<Action>,
    /// Whether the rule is enabled.
    pub enabled: bool,
    /// Roles that should not be affected by the rule.
    ///
    /// Maximum of 20.
    pub exempt_roles: FixedArray<RoleId, u8>,
    /// Channels that should not be affected by the rule.
    ///
    /// Maximum of 50.
    pub exempt_channels: FixedArray<ChannelId, u8>,
}

/// Indicates in what event context a rule should be checked.
///
/// [Discord docs](https://discord.com/developers/docs/resources/auto-moderation#auto-moderation-rule-object-event-types).
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]

#[serde(from = "u8", into = "u8")]
#[non_exhaustive]
pub enum EventType {
    MessageSend,
    Unknown(u8),
}

impl From<u8> for EventType {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::MessageSend,
            _ => Self::Unknown(value),
        }
    }
}

impl From<EventType> for u8 {
    fn from(value: EventType) -> Self {
        match value {
            EventType::MessageSend => 1,
            EventType::Unknown(unknown) => unknown,
        }
    }
}

/// Characterizes the type of content which can trigger the rule.
///
/// Discord docs:
/// [type](https://discord.com/developers/docs/resources/auto-moderation#auto-moderation-rule-object-trigger-types),
/// [metadata](https://discord.com/developers/docs/resources/auto-moderation#auto-moderation-rule-object-trigger-metadata)
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub struct Trigger {
    #[serde(rename = "trigger_type")]
    kind: TriggerType,
    #[serde(rename = "trigger_metadata")]
    metadata: TriggerMetadata,
}

/// Type of [`Trigger`].
///
/// [Discord docs](https://discord.com/developers/docs/resources/auto-moderation#auto-moderation-rule-object-trigger-types).
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]

#[serde(from = "u8", into = "u8")]
#[non_exhaustive]
pub enum TriggerType {
    Keyword,
    Spam,
    KeywordPreset,
    MentionSpam,
    Unknown(u8),
}

impl From<u8> for TriggerType {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::Keyword,
            3 => Self::Spam,
            4 => Self::KeywordPreset,
            5 => Self::MentionSpam,
            _ => Self::Unknown(value),
        }
    }
}

impl From<TriggerType> for u8 {
    fn from(value: TriggerType) -> Self {
        match value {
            TriggerType::Keyword => 1,
            TriggerType::Spam => 3,
            TriggerType::KeywordPreset => 4,
            TriggerType::MentionSpam => 5,
            TriggerType::Unknown(unknown) => unknown,
        }
    }
}

/// Individual change for trigger metadata within an audit log entry.
///
/// Different fields are relevant based on the value of trigger_type. See
/// [`Change::TriggerMetadata`].
///
/// [`Change::TriggerMetadata`]: crate::model::guild::audit_log::Change::TriggerMetadata
///
/// [Discord docs](https://discord.com/developers/docs/resources/auto-moderation#auto-moderation-rule-object-trigger-metadata).

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub struct TriggerMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keyword_filter: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regex_patterns: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presets: Option<Vec<KeywordPresetType>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_list: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mention_total_limit: Option<u8>,
}

/// Internally pre-defined wordsets which will be searched for in content.
///
/// [Discord docs](https://discord.com/developers/docs/resources/auto-moderation#auto-moderation-rule-object-keyword-preset-types).
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]

#[serde(from = "u8", into = "u8")]
#[non_exhaustive]
pub enum KeywordPresetType {
    /// Words that may be considered forms of swearing or cursing
    Profanity,
    /// Words that refer to sexually explicit behavior or activity
    SexualContent,
    /// Personal insults or words that may be considered hate speech
    Slurs,
    Unknown(u8),
}

impl From<u8> for KeywordPresetType {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::Profanity,
            2 => Self::SexualContent,
            3 => Self::Slurs,
            _ => Self::Unknown(value),
        }
    }
}

impl From<KeywordPresetType> for u8 {
    fn from(value: KeywordPresetType) -> Self {
        match value {
            KeywordPresetType::Profanity => 1,
            KeywordPresetType::SexualContent => 2,
            KeywordPresetType::Slurs => 3,
            KeywordPresetType::Unknown(unknown) => unknown,
        }
    }
}

/// Metadata for an action.
#[derive(Default, Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub struct ActionMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    channel_id: Option<GenericChannelId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    duration_seconds: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    custom_message: Option<FixedString<u16>>,
}

/// An action which will execute whenever a rule is triggered.
///
/// [Discord docs](https://discord.com/developers/docs/resources/auto-moderation#auto-moderation-action-object).
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub struct Action {
    #[serde(rename = "type")]
    pub kind: ActionType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<ActionMetadata>,
}

enum_number! {
    /// See [`Action`]
    ///
    /// [Discord docs](https://discord.com/developers/docs/resources/auto-moderation#auto-moderation-action-object-action-types).
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[non_exhaustive]
    pub enum ActionType {
        BlockMessage = 1,
        Alert = 2,
        Timeout = 3,
        _ => Unknown(u8),
    }
}
