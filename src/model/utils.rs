use std::fmt;

use super::prelude::*;

pub(super) enum StrOrInt<'de> {
    String(String),
    Str(&'de str),
    Int(u64),
}

impl StrOrInt<'_> {
    pub fn parse(&self) -> Result<u64, std::num::ParseIntError> {
        match self {
            StrOrInt::String(val) => val.parse(),
            StrOrInt::Str(val) => val.parse(),
            StrOrInt::Int(val) => Ok(*val),
        }
    }
}

impl<'de> serde::Deserialize<'de> for StrOrInt<'de> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = StrOrInt<'de>;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a string or integer")
            }

            fn visit_borrowed_str<E>(self, val: &'de str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(StrOrInt::Str(val))
            }

            fn visit_str<E: serde::de::Error>(self, val: &str) -> StdResult<Self::Value, E> {
                self.visit_string(val.into())
            }

            fn visit_string<E: serde::de::Error>(self, val: String) -> StdResult<Self::Value, E> {
                Ok(StrOrInt::String(val))
            }

            fn visit_i64<E: serde::de::Error>(self, val: i64) -> StdResult<Self::Value, E> {
                self.visit_u64(val as _)
            }

            fn visit_u64<E: serde::de::Error>(self, val: u64) -> Result<Self::Value, E> {
                Ok(StrOrInt::Int(val))
            }
        }

        deserializer.deserialize_any(Visitor)
    }
}

#[cfg(test)]
#[track_caller]
pub(crate) fn assert_json<T>(data: &T, json: Value)
where
    T: serde::Serialize + for<'de> Deserialize<'de> + PartialEq + std::fmt::Debug,
{
    // test serialization
    let serialized = serde_json::to_value(data).unwrap();
    assert!(
        serialized == json,
        "data->JSON serialization failed\nexpected: {json:?}\n     got: {serialized:?}"
    );

    // test deserialization
    let deserialized = serde_json::from_value::<T>(json).unwrap();
    assert!(
        &deserialized == data,
        "JSON->data deserialization failed\nexpected: {data:?}\n     got: {deserialized:?}"
    );
}

/// Used with `#[serde(with = "single_recipient")]`
pub mod single_recipient {
    use serde::de::Error;
    use serde::ser::SerializeSeq;
    use serde::{Deserialize, Deserializer, Serializer};

    use crate::model::user::User;

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<User, D::Error> {
        let mut users: Vec<User> = Vec::deserialize(deserializer)?;

        let user = if users.is_empty() {
            return Err(Error::custom("Expected a single recipient"));
        } else {
            users.remove(0)
        };

        Ok(user)
    }

    pub fn serialize<S: Serializer>(user: &User, serializer: S) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(1))?;

        seq.serialize_element(user)?;

        seq.end()
    }
}
