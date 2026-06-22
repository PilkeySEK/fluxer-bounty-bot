use std::{collections::HashMap, pin::Pin, sync::Arc};

use chrono::Utc;
use fluxer_neptunium::{
    cached_payload::CachedMessageReactionAdd,
    events::EventError,
    model::id::{Id, marker::MessageMarker},
};
use tokio::sync::{
    Mutex,
    mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel},
};

pub trait ReactionHandler: Send + Sync {
    fn call(
        &mut self,
        // ctx: Context,
        event: Arc<CachedMessageReactionAdd>,
    ) -> (bool, Result<(), EventError>);
}

impl<F> ReactionHandler for F
where
    F: FnMut(/* Context, */ Arc<CachedMessageReactionAdd>) -> (bool, Result<(), EventError>)
        + Send
        + Sync,
{
    fn call(
        &mut self,
        // ctx: Context,
        event: Arc<CachedMessageReactionAdd>,
    ) -> (bool, Result<(), EventError>) {
        self(event)
    }
}

pub type ReactionExpiryHandlerFn =
    Box<dyn FnOnce() -> Pin<Box<dyn Future<Output = Result<(), EventError>> + Send>> + Send + Sync>;

/// The last element may be `None` if the reaction handler does not expire, otherwise it is the time when it expires.
pub type ReactionsEventHandlerMessage = (
    Id<MessageMarker>,
    Box<dyn ReactionHandler>,
    Option<(ReactionExpiryHandlerFn, chrono::DateTime<chrono::Utc>)>,
);

pub(super) struct ReactionsEventHandler {
    rx: tokio::sync::Mutex<UnboundedReceiver<ReactionsEventHandlerMessage>>,
    pub tx: UnboundedSender<ReactionsEventHandlerMessage>,
    #[expect(clippy::type_complexity)]
    reaction_handlers: Mutex<
        HashMap<
            Id<MessageMarker>,
            (
                Box<dyn ReactionHandler>,
                Option<(ReactionExpiryHandlerFn, chrono::DateTime<chrono::Utc>)>,
            ),
        >,
    >,
}

impl ReactionsEventHandler {
    pub(super) fn new() -> Self {
        let (tx, rx) = unbounded_channel();
        Self {
            tx,
            rx: Mutex::new(rx),
            reaction_handlers: Mutex::new(HashMap::new()),
        }
    }

    pub(super) async fn handle_reaction_add(
        &self,
        // ctx: Context,
        event: Arc<CachedMessageReactionAdd>,
    ) -> Result<(), EventError> {
        let message_id = event.message_id;
        self.recv_and_expire_all().await;
        let mut reaction_handlers = self.reaction_handlers.lock().await;
        if let Some((handler, _)) = reaction_handlers.get_mut(&message_id) {
            let (remove, result) = handler.call(event);
            if remove {
                reaction_handlers.remove(&message_id);
            }
            result
        } else {
            Ok(())
        }
    }

    /// Receive and process all pending messages and removes any expired entries.
    /// Basically, this syncs the handler.
    async fn recv_and_expire_all(&self) {
        let mut rx = self.rx.lock().await;
        let mut reaction_handlers = self.reaction_handlers.lock().await;
        while let Ok((message_id, handler, expiry)) = rx.try_recv() {
            if reaction_handlers
                .insert(message_id, (handler, expiry))
                .is_some()
            {
                tracing::warn!(%message_id, "Multiple reaction handlers for message");
            }
        }
        let now = Utc::now();
        let removed_handlers = reaction_handlers.extract_if(|_, (_, expiry)| {
            if let Some(expiry) = expiry
                && expiry.1 < now
            {
                true
            } else {
                false
            }
        });

        for (_, (_, expiry)) in removed_handlers {
            if let Some(expiry) = expiry
                && let Err(e) = expiry.0().await
            {
                tracing::error!("Expiry handler returned error: {e}");
            }
        }
    }
}
