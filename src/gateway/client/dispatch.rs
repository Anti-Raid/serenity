use std::sync::Arc;

use super::event_handler::EventHandler;
use super::Context;
use crate::all::IEvent;
use crate::internal::tokio::spawn_named;

/// Calls the user's event handlers.
pub(crate) async fn dispatch_model(
    event: Box<IEvent>,
    context: Context,
    event_handler: Option<Arc<dyn EventHandler>>,
) {
    spawn_named("dispatch::user", async move {
        dispatch_event_handler(&context, event_handler, &event).await;
    });
}

async fn dispatch_event_handler(
    context: &Context,
    event_handler: Option<Arc<dyn EventHandler>>,
    full_event: &IEvent,
) {
    if let Some(handler) = event_handler {
        handler.dispatch(context, full_event).await;
    }
}
