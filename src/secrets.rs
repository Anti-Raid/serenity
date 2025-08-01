use std::sync::Arc;

/// A cheaply clonable, zeroed on drop, String.
///
/// This is a simple newtype of `Arc<str>` that does not reveal its
/// content in its [`Debug`](std::fmt::Debug) output.
#[derive(Clone, Deserialize, Serialize)]
pub struct SecretString(Arc<str>);

impl SecretString {
    #[must_use]
    pub fn new(inner: Arc<str>) -> Self {
        Self(inner)
    }

    #[must_use]
    pub fn expose_secret(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for SecretString {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.debug_tuple(std::any::type_name::<Self>()).field(&"<secret>").finish()
    }
}
