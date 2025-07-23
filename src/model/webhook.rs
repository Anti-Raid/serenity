//! Webhook model and implementations.

use crate::model::prelude::*;

enum_number! {
    /// A representation of a type of webhook.
    ///
    /// [Discord docs](https://discord.com/developers/docs/resources/webhook#webhook-object-webhook-types).
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[non_exhaustive]
    pub enum WebhookType {
        /// An indicator that the webhook can post messages to channels with a token.
        Incoming = 1,
        /// An indicator that the webhook is managed by Discord for posting new messages to
        /// channels without a token.
        ChannelFollower = 2,
        /// Application webhooks are webhooks used with Interactions.
        Application = 3,
        _ => Unknown(u8),
    }
}

impl WebhookType {
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Incoming => "incoming",
            Self::ChannelFollower => "channel follower",
            Self::Application => "application",
            _ => "unknown",
        }
    }
}

/// A representation of a webhook, which is a low-effort way to post messages to channels. They do
/// not necessarily require a bot user or authentication to use.
///
/// [Discord docs](https://discord.com/developers/docs/resources/webhook#webhook-object).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Webhook {
    /// The unique Id.
    ///
    /// Can be used to calculate the creation date of the webhook.
    pub id: WebhookId,
    /// The type of the webhook.
    #[serde(rename = "type")]
    pub kind: WebhookType,
    /// The Id of the guild that owns the webhook.
    pub guild_id: Option<GuildId>,
    /// The Id of the channel that owns the webhook.
    pub channel_id: Option<ChannelId>,
    /// The user that created the webhook.
    ///
    /// **Note**: This is not received when getting a webhook by its token.
    pub user: Option<User>,
    /// The default name of the webhook.
    ///
    /// This can be temporarily overridden via [`ExecuteWebhook::username`].
    pub name: Option<FixedString<u8>>,
    /// The default avatar.
    ///
    /// This can be temporarily overridden via [`ExecuteWebhook::avatar_url`].
    pub avatar: Option<ImageHash>,
    /// The webhook's secure token.
    pub token: Option<SecretString>,
    /// The bot/OAuth2 application that created this webhook.
    pub application_id: Option<ApplicationId>,
    /// The guild of the channel that this webhook is following (returned for
    /// [`WebhookType::ChannelFollower`])
    pub source_guild: Option<WebhookGuild>,
    /// The channel that this webhook is following (returned for
    /// [`WebhookType::ChannelFollower`]).
    pub source_channel: Option<WebhookChannel>,
    /// The url used for executing the webhook (returned by the webhooks OAuth2 flow).
    pub url: Option<SecretString>,
}

impl ExtractKey<WebhookId> for Webhook {
    fn extract_key(&self) -> &WebhookId {
        &self.id
    }
}

/// The guild object returned by a [`Webhook`], of type [`WebhookType::ChannelFollower`].
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct WebhookGuild {
    /// The unique Id identifying the guild.
    pub id: GuildId,
    /// The name of the guild.
    pub name: FixedString<u16>,
    /// The hash of the icon used by the guild.
    ///
    /// In the client, this appears on the guild list on the left-hand side.
    pub icon: Option<ImageHash>,
}

#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct WebhookChannel {
    /// The unique Id of the channel.
    pub id: ChannelId,
    /// The name of the channel.
    pub name: FixedString<u16>,
}

#[cfg(feature = "model")]
impl Webhook {
    /// Returns the url of the webhook.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Model`] if the [`Self::token`] is [`None`].
    pub fn url(&self) -> Result<String> {
        let token = self.token.as_ref().ok_or(ModelError::NoTokenSet)?.expose_secret();
        Ok(format!("https://discord.com/api/webhooks/{}/{token}", self.id))
    }
}
