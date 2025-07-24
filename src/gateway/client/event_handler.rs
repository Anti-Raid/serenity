use async_trait::async_trait;

use super::context::Context;
#[cfg(doc)]
use crate::gateway::ShardRunner;
use crate::model::prelude::*;

#[async_trait]
pub trait EventHandler: Send + Sync {
    /// Dispatches an event through this handler, allowing for event matching and handling
    /// based on individual event variants.
    async fn dispatch(&self, _context: &Context, _event: &IEvent) {}
}