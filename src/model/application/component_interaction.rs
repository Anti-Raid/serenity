use serde::de::Error as DeError;
use serde::ser::{Serialize, SerializeMap as _};
use serde_json::from_value;

use crate::model::prelude::*;

/// An interaction triggered by a message component.
///
/// [Discord docs](https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-object-interaction-structure).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(remote = "Self")]
#[non_exhaustive]
pub struct ComponentInteraction {
    /// Id of the interaction.
    pub id: InteractionId,
    /// Id of the application this interaction is for.
    pub application_id: ApplicationId,
    /// The data of the interaction which was triggered.
    pub data: ComponentInteractionData,
    /// The guild Id this interaction was sent from, if there is one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guild_id: Option<GuildId>,
    /// Channel that the interaction was sent from.
    pub channel: Option<GenericInteractionChannel>,
    /// The channel Id this interaction was sent from.
    pub channel_id: GenericChannelId,
    /// The `member` data for the invoking user.
    ///
    /// **Note**: It is only present if the interaction is triggered in a guild.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub member: Option<Member>,
    /// The `user` object for the invoking user.
    #[serde(default)]
    pub user: User,
    /// A continuation token for responding to the interaction.
    pub token: FixedString,
    /// Always `1`.
    pub version: u8,
    /// The message this interaction was triggered by, if it is a component.
    pub message: Box<Message>,
    /// Permissions the app or bot has within the channel the interaction was sent from.
    pub app_permissions: Permissions,
    /// The selected language of the invoking user.
    pub locale: FixedString,
    /// The guild's preferred locale.
    pub guild_locale: Option<FixedString>,
    /// For monetized applications, any entitlements of the invoking user.
    pub entitlements: Vec<Entitlement>,
    /// The owners of the applications that authorized the interaction, such as a guild or user.
    #[serde(default)]
    pub authorizing_integration_owners: AuthorizingIntegrationOwners,
    /// The context where the interaction was triggered from.
    pub context: Option<InteractionContext>,
}

// Manual impl needed to insert guild_id into model data
impl<'de> Deserialize<'de> for ComponentInteraction {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        // calls #[serde(remote)]-generated inherent method
        let mut interaction = Self::deserialize(deserializer)?;
        if let (Some(guild_id), Some(member)) = (interaction.guild_id, &mut interaction.member) {
            member.guild_id = guild_id;
            // If `member` is present, `user` wasn't sent and is still filled with default data
            interaction.user = member.user.clone();
        }
        Ok(interaction)
    }
}

impl Serialize for ComponentInteraction {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> StdResult<S::Ok, S::Error> {
        // calls #[serde(remote)]-generated inherent method
        Self::serialize(self, serializer)
    }
}

#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug)]
pub enum ComponentInteractionDataKind {
    Button,
    StringSelect { values: FixedArray<String> },
    UserSelect { values: FixedArray<UserId> },
    RoleSelect { values: FixedArray<RoleId> },
    MentionableSelect { values: FixedArray<GenericId> },
    ChannelSelect { values: FixedArray<ChannelId> },
    Unknown(u8),
}

// Manual impl needed to emulate integer enum tags
impl<'de> Deserialize<'de> for ComponentInteractionDataKind {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        #[derive(Deserialize)]
        struct Json {
            component_type: ComponentType,
            values: Option<Value>,
        }
        let json = Json::deserialize(deserializer)?;

        macro_rules! parse_values {
            () => {
                from_value(json.values.ok_or_else(|| D::Error::missing_field("values"))?)
                    .map_err(D::Error::custom)?
            };
        }

        Ok(match json.component_type {
            ComponentType::Button => Self::Button,
            ComponentType::StringSelect => Self::StringSelect {
                values: parse_values!(),
            },
            ComponentType::UserSelect => Self::UserSelect {
                values: parse_values!(),
            },
            ComponentType::RoleSelect => Self::RoleSelect {
                values: parse_values!(),
            },
            ComponentType::MentionableSelect => Self::MentionableSelect {
                values: parse_values!(),
            },
            ComponentType::ChannelSelect => Self::ChannelSelect {
                values: parse_values!(),
            },
            x @ (ComponentType::ActionRow | ComponentType::InputText) => {
                return Err(D::Error::custom(format_args!(
                    "invalid message component type in this context: {x:?}",
                )));
            },
            ComponentType(x) => Self::Unknown(x),
        })
    }
}

impl Serialize for ComponentInteractionDataKind {
    #[rustfmt::skip] // Remove this for horror.
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> StdResult<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry("component_type", &match self {
            Self::Button { .. } => 2,
            Self::StringSelect { .. } => 3,
            Self::UserSelect { .. } => 5,
            Self::RoleSelect { .. } => 6,
            Self::MentionableSelect { .. } => 7,
            Self::ChannelSelect { .. } => 8,
            Self::Unknown(x) => *x,
        })?;

        match self {
            Self::StringSelect { values } => map.serialize_entry("values", values)?,
            Self::UserSelect { values } => map.serialize_entry("values", values)?,
            Self::RoleSelect { values } => map.serialize_entry("values", values)?,
            Self::MentionableSelect { values } => map.serialize_entry("values", values)?,
            Self::ChannelSelect { values } => map.serialize_entry("values", values)?,
            Self::Button | Self::Unknown(_) => map.serialize_entry("values", &None::<()>)?,
        }

        map.end()
    }
}

/// A message component interaction data, provided by [`ComponentInteraction::data`]
///
/// [Discord docs](https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-object-message-component-data-structure).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ComponentInteractionData {
    /// The custom id of the component.
    pub custom_id: FixedString,
    /// Type and type-specific data of this component interaction.
    #[serde(flatten)]
    pub kind: ComponentInteractionDataKind,
}
