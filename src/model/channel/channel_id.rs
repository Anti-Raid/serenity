use crate::model::prelude::*;

impl ChannelId {
    /// Converts the type of this Id to [`GenericChannelId`].
    ///
    /// This allows you to call methods which are shared between channels and threads, and does not
    /// change the inner value at all.
    #[must_use]
    pub fn widen(self) -> GenericChannelId {
        self.into()
    }
}

#[cfg(feature = "model")]
impl From<Channel> for GenericChannelId {
    /// Gets the Id of a [`Channel`].
    fn from(channel: Channel) -> Self {
        channel.id()
    }
}

#[cfg(feature = "model")]
impl From<&Channel> for GenericChannelId {
    /// Gets the Id of a [`Channel`].
    fn from(channel: &Channel) -> Self {
        channel.id()
    }
}

impl From<PrivateChannel> for ChannelId {
    /// Gets the Id of a private channel.
    fn from(private_channel: PrivateChannel) -> ChannelId {
        private_channel.id
    }
}

impl From<&PrivateChannel> for ChannelId {
    /// Gets the Id of a private channel.
    fn from(private_channel: &PrivateChannel) -> ChannelId {
        private_channel.id
    }
}

impl From<GuildChannel> for ChannelId {
    /// Gets the Id of a guild channel.
    fn from(public_channel: GuildChannel) -> ChannelId {
        public_channel.id
    }
}

impl From<&GuildChannel> for ChannelId {
    /// Gets the Id of a guild channel.
    fn from(public_channel: &GuildChannel) -> ChannelId {
        public_channel.id
    }
}
