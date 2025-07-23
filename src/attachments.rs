use std::borrow::Cow;
use std::path::Path;

use bytes::Bytes;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

use crate::error::{Error, Result, UrlError};
#[cfg(feature = "http")]
use crate::http::Http;

#[derive(Clone, Debug)]
pub enum AttachmentData<'a> {
    Bytes(Bytes),
    File(&'a File),
    Path(&'a Path),
}

/// A builder for creating a new attachment from a file path, file data, or URL.
///
/// [Discord docs](https://discord.com/developers/docs/resources/channel#attachment-object-attachment-structure).
#[derive(Clone, Debug)]
#[non_exhaustive]
#[must_use]
pub struct CreateAttachment<'a> {
    pub filename: Cow<'static, str>,
    pub description: Option<Cow<'a, str>>,
    pub data: AttachmentData<'a>,
}

impl<'a> CreateAttachment<'a> {
    /// Builds an [`CreateAttachment`] from the raw attachment data.
    pub fn bytes(data: impl Into<Bytes>, filename: impl Into<Cow<'static, str>>) -> Self {
        CreateAttachment {
            filename: filename.into(),
            description: None,
            data: AttachmentData::Bytes(data.into()),
        }
    }

    /// Builds an [`CreateAttachment`] by reading a local file.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Io`] if the path is not a valid file path.
    pub fn path(path: &'a Path) -> Result<Self> {
        let filename = path
            .file_name()
            .ok_or_else(|| std::io::Error::other("attachment path must not be a directory"))?
            .to_string_lossy()
            .into_owned();
        Ok(CreateAttachment {
            filename: filename.into(),
            description: None,
            data: AttachmentData::Path(path),
        })
    }

    /// Builds an [`CreateAttachment`] by reading from a file handler.
    pub fn file(file: &'a File, filename: impl Into<Cow<'static, str>>) -> Self {
        CreateAttachment {
            filename: filename.into(),
            description: None,
            data: AttachmentData::File(file),
        }
    }

    /// Builds an [`CreateAttachment`] by downloading attachment data from a URL.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if downloading the data fails.
    #[cfg(feature = "http")]
    pub async fn url(
        http: &Http,
        url: impl reqwest::IntoUrl,
        filename: impl Into<Cow<'static, str>>,
    ) -> Result<Self> {
        let response = http.client.get(url).send().await?;
        let data = response.bytes().await?;

        Ok(CreateAttachment::bytes(data, filename))
    }

    /// Returns the underlying data for the attachment.
    ///
    /// # Errors
    ///
    /// Returns an error if fetching the data failed in some way. If the attachment is a
    /// [`CreateAttachment::path`], then the file at the specified path was unable to be read. If
    /// instead it's [`CreateAttachment::file`], then cloning the handle to the file failed, likely
    /// due to hitting the system's limit on number of open file handles.
    pub async fn get_data(&self) -> Result<Bytes> {
        match &self.data {
            AttachmentData::Bytes(bytes) => Ok(bytes.clone()),
            AttachmentData::Path(path) => {
                let mut file = File::open(path).await?;
                let mut data = Vec::new();
                file.read_to_end(&mut data).await?;
                Ok(data.into())
            },
            AttachmentData::File(file) => {
                let mut data = Vec::new();
                file.try_clone().await?.read_to_end(&mut data).await?;
                Ok(data.into())
            },
        }
    }

    /// Converts the attachment data to a base64-encoded data URI.
    ///
    /// # Errors
    ///
    /// See [`CreateAttachment::get_data`] for details.
    pub async fn encode(&self) -> Result<ImageData<'_>> {
        use base64::engine::{Config, Engine};

        const PREFIX: &str = "data:image/png;base64,";
        let data = self.get_data().await?;

        let engine = base64::prelude::BASE64_STANDARD;
        let encoded_size = base64::encoded_len(data.len(), engine.config().encode_padding())
            .and_then(|len| len.checked_add(PREFIX.len()))
            .expect("buffer capacity overflow");

        let mut encoded = String::with_capacity(encoded_size);
        encoded.push_str(PREFIX);
        engine.encode_string(&data, &mut encoded);
        Ok(ImageData(encoded.into()))
    }

    /// Sets a description for the file (max 1024 characters).
    pub fn description(mut self, description: impl Into<Cow<'a, str>>) -> Self {
        self.description = Some(description.into());
        self
    }
}

/// A wrapper around some base64-encoded image data. Used when an endpoint expects the image
/// payload directly as part of the JSON body, instead of as a multipart upload.
#[derive(Clone, Debug, Serialize)]
#[serde(transparent)]
pub struct ImageData<'a>(Cow<'a, str>);

impl<'a> ImageData<'a> {
    /// Accesses the stored base64-encoded image data.
    #[must_use]
    pub fn as_base64(&self) -> &str {
        &self.0
    }

    /// Constructs image data from a base64-encoded blob of data. The string must be a valid data
    /// URI, for example:
    ///
    /// ```
    /// use serenity::builder::ImageData;
    ///
    /// let s = "data:image/png;base64,R0lGODlhAQABAIAAAP///wAAACwAAAAAAQABAAACAkQBADs=";
    /// assert!(ImageData::from_base64(s).is_ok());
    /// ```
    ///
    /// # Errors
    ///
    /// Returns a [`Error::Url`] if the string is not a valid data URI. See the [Discord
    /// docs](https://discord.com/developers/docs/reference#image-data).
    pub fn from_base64(s: impl Into<Cow<'a, str>>) -> Result<Self> {
        let s = s.into();
        if let Some(("data", tail)) = s.split_once(':')
            && let Some((mimetype, encoding)) = tail.split_once(';')
            && mimetype.split_once('/').is_some()
            && encoding.starts_with("base64,")
        {
            Ok(Self(s))
        } else {
            Err(Error::Url(UrlError::InvalidDataURI))
        }
    }
}
