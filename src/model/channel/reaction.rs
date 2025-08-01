use std::cmp::Ordering;
use std::fmt::{self, Write as _};

#[cfg(feature = "http")]
use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};
use serde::de::Error as DeError;
use serde::ser::{Serialize, SerializeMap, Serializer};
use crate::model::prelude::*;

enum_number! {
    /// A list of types a reaction can be.
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    
    #[non_exhaustive]
    pub enum ReactionTypes {
        Normal = 0,
        Burst = 1,
        _ => Unknown(u8),
    }
}

/// The type of a [`Reaction`] sent.

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub enum ReactionType {
    /// A reaction with a [`Guild`]s custom [`Emoji`], which is unique to the guild.
    Custom {
        /// Whether the emoji is animated.
        animated: bool,
        /// The Id of the custom [`Emoji`].
        id: EmojiId,
        /// The name of the custom emoji. This is primarily used for decoration and distinguishing
        /// the emoji client-side.
        name: Option<FixedString>,
    },
    /// A reaction with a twemoji.
    Unicode(FixedString),
}

// Manual impl needed to decide enum variant by presence of `id`
impl<'de> Deserialize<'de> for ReactionType {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        #[derive(Deserialize)]
        struct PartialEmoji {
            #[serde(default)]
            animated: bool,
            id: Option<EmojiId>,
            name: Option<FixedString>,
        }
        let emoji = PartialEmoji::deserialize(deserializer)?;
        Ok(match (emoji.id, emoji.name) {
            (Some(id), name) => ReactionType::Custom {
                animated: emoji.animated,
                id,
                name,
            },
            (None, Some(name)) => ReactionType::Unicode(name),
            (None, None) => return Err(DeError::custom("invalid reaction type data")),
        })
    }
}

impl Serialize for ReactionType {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            ReactionType::Custom {
                animated,
                id,
                name,
            } => {
                let mut map = serializer.serialize_map(Some(3))?;

                map.serialize_entry("animated", animated)?;
                map.serialize_entry("id", id)?;
                map.serialize_entry("name", name)?;

                map.end()
            },
            ReactionType::Unicode(name) => {
                let mut map = serializer.serialize_map(Some(1))?;

                map.serialize_entry("name", name)?;

                map.end()
            },
        }
    }
}

impl ReactionType {
    /// Creates a data-esque display of the type. This is not very useful for displaying, as the
    /// primary client can not render it, but can be useful for debugging.
    ///
    /// **Note**: This is mainly for use internally. There is otherwise most likely little use for
    /// it.
    #[must_use]
    #[cfg(feature = "http")]
    pub fn as_data(&self) -> String {
        match self {
            ReactionType::Custom {
                id,
                name,
                ..
            } => {
                format!("{}:{id}", name.as_deref().unwrap_or_default())
            },
            ReactionType::Unicode(unicode) => {
                utf8_percent_encode(unicode, NON_ALPHANUMERIC).to_string()
            },
        }
    }

    /// Helper function to allow testing equality of unicode emojis without having to perform any
    /// allocation. Will always return false if the reaction was not a unicode reaction.
    #[must_use]
    pub fn unicode_eq(&self, other: &str) -> bool {
        if let ReactionType::Unicode(unicode) = &self {
            &**unicode == other
        } else {
            // Always return false if not a unicode reaction
            false
        }
    }

    /// Helper function to allow comparing unicode emojis without having to perform any allocation.
    /// Will return None if the reaction was not a unicode reaction.
    #[must_use]
    pub fn unicode_partial_cmp(&self, other: &str) -> Option<Ordering> {
        if let ReactionType::Unicode(unicode) = &self {
            Some(unicode.as_str().cmp(other))
        } else {
            // Always return None if not a unicode reaction
            None
        }
    }
}

impl From<char> for ReactionType {
    /// Creates a [`ReactionType`] from a `char`.
    ///
    /// # Examples
    ///
    /// Reacting to a message with an apple:
    ///
    /// ```rust,no_run
    /// # #[cfg(feature = "http")]
    /// # use serenity::http::Http;
    /// # use serenity::model::channel::Message;
    /// # use serenity::model::id::ChannelId;
    /// #
    /// # #[cfg(feature = "http")]
    /// # async fn example(http: &Http, message: Message) -> Result<(), Box<dyn std::error::Error>> {
    /// message.react(http, '🍎').await?;
    /// # Ok(())
    /// # }
    /// #
    /// # fn main() {}
    /// ```
    fn from(ch: char) -> ReactionType {
        ReactionType::Unicode(ch.to_string().trunc_into())
    }
}

impl From<EmojiId> for ReactionType {
    fn from(emoji_id: EmojiId) -> ReactionType {
        ReactionType::Custom {
            animated: false,
            id: emoji_id,
            name: Some(FixedString::from_static_trunc("emoji")),
        }
    }
}

#[derive(Debug)]
pub struct ReactionConversionError;

impl fmt::Display for ReactionConversionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("failed to convert from a string to ReactionType")
    }
}

impl std::error::Error for ReactionConversionError {}

impl fmt::Display for ReactionType {
    /// Formats the reaction type, displaying the associated emoji in a way that clients can
    /// understand.
    ///
    /// If the type is a [custom][`ReactionType::Custom`] emoji, then refer to the documentation
    /// for [emoji's formatter][`Emoji::fmt`] on how this is displayed. Otherwise, if the type is a
    /// [unicode][`ReactionType::Unicode`], then the inner unicode is displayed.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReactionType::Custom {
                animated,
                id,
                name,
            } => {
                if *animated {
                    f.write_str("<a:")?;
                } else {
                    f.write_str("<:")?;
                }

                if let Some(name) = name {
                    f.write_str(name)?;
                }

                f.write_char(':')?;
                fmt::Display::fmt(id, f)?;
                f.write_char('>')
            },
            ReactionType::Unicode(unicode) => f.write_str(unicode),
        }
    }
}
