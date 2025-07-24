use serde::Serialize;

use crate::model::prelude::*;

/// An interaction triggered by a modal submit.
///
/// [Discord docs](https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-object).

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(remote = "Self")]
#[non_exhaustive]
pub struct ModalInteraction {
    /// Id of the interaction.
    pub id: InteractionId,
    /// Id of the application this interaction is for.
    pub application_id: ApplicationId,
    /// The data of the interaction which was triggered.
    pub data: ModalInteractionData,
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
    /// The message this interaction was triggered by
    ///
    /// **Note**: Does not exist if the modal interaction originates from an application command
    /// interaction
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<Box<Message>>,
    /// Permissions the app or bot has within the channel the interaction was sent from.
    pub app_permissions: Permissions,
    /// The selected language of the invoking user.
    pub locale: FixedString,
    /// The guild's preferred locale.
    pub guild_locale: Option<FixedString>,
    /// For monetized applications, any entitlements of the invoking user.
    pub entitlements: Vec<Entitlement>,
}

// Manual impl needed to insert guild_id into resolved Role's
impl<'de> Deserialize<'de> for ModalInteraction {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        let mut interaction = Self::deserialize(deserializer)?; // calls #[serde(remote)]-generated inherent method
        if let (Some(guild_id), Some(member)) = (interaction.guild_id, &mut interaction.member) {
            member.guild_id = guild_id;
            // If `member` is present, `user` wasn't sent and is still filled with default data
            interaction.user = member.user.clone();
        }
        Ok(interaction)
    }
}

impl Serialize for ModalInteraction {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> StdResult<S::Ok, S::Error> {
        Self::serialize(self, serializer) // calls #[serde(remote)]-generated inherent method
    }
}

/// A modal submit interaction data, provided by [`ModalInteraction::data`]
///
/// [Discord docs](https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-object-interaction-data-structure).

#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ModalInteractionData {
    /// The custom id of the modal
    pub custom_id: FixedString,
    /// The components.
    pub components: FixedArray<ActionRow>,
}
