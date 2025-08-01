use std::collections::HashMap;
use std::fmt;

use crate::model::prelude::*;
use crate::model::utils::single_recipient;

/// A Direct Message text channel with another user.
///
/// [Discord docs](https://discord.com/developers/docs/resources/channel#channel-object).

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct PrivateChannel {
    /// The unique Id of the private channel.
    ///
    /// Can be used to calculate the first message's creation date.
    pub id: ChannelId,
    /// Indicator of the type of channel this is.
    ///
    /// This should always be [`ChannelType::Private`].
    #[serde(rename = "type")]
    pub kind: ChannelType,
    /// The recipient to the private channel.
    #[serde(with = "single_recipient", rename = "recipients")]
    pub recipient: User,

    #[serde(flatten)]
    pub extra_info: HashMap<String, serde_json::Value>,
}

impl fmt::Display for PrivateChannel {
    /// Formats the private channel, displaying the recipient's username.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.recipient.name)
    }
}
