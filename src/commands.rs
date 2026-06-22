use chrono::{TimeDelta, Utc};
use fluxer_neptunium::{
    cache::CachedGuildMember,
    cached_payload::CachedMessageCreate,
    events::context::Context,
    model::id::{
        Id,
        marker::{GuildMarker, MessageMarker},
    },
};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    db::DbManager,
    event_handler::reactions::{
        ReactionExpiryHandlerFn, ReactionHandler, ReactionsEventHandlerMessage,
    },
};

pub mod bounty_management;
pub mod misc;

pub struct CommandContext<'a> {
    pub ctx: &'a Context,
    pub db: &'a DbManager,
    pub message: &'a CachedMessageCreate,
    pub guild_member: &'a CachedGuildMember,
    pub guild_id: Id<GuildMarker>,
    pub reaction_handler_tx: &'a UnboundedSender<ReactionsEventHandlerMessage>,
}

impl CommandContext<'_> {
    /// Helper for registering a reaction handler. For more control, use `reaction_handler_tx` on this struct.
    ///
    /// If an error occurs, it is logged but no panics will happen.
    pub fn register_reaction_handler(
        &self,
        message_id: Id<MessageMarker>,
        handler: impl ReactionHandler + 'static,
        expiry: Option<(ReactionExpiryHandlerFn, std::time::Duration)>,
    ) {
        let expiry = match expiry {
            Some((f, expires_in)) => {
                let expires_in: TimeDelta = match TimeDelta::from_std(expires_in) {
                    Ok(expires_in) => expires_in,
                    Err(e) => {
                        tracing::error!("Failed to convert Duration to TimeDelta: {e}");
                        return;
                    }
                };
                let now = Utc::now();
                let Some(expires_at) = now.checked_add_signed(expires_in) else {
                    tracing::error!("Overflow calculating expires_at.");
                    return;
                };
                Some((f, expires_at))
            }
            None => None,
        };
        if self
            .reaction_handler_tx
            .send((message_id, Box::new(handler), expiry))
            .is_err()
        {
            tracing::error!("The reaction handler is gone.");
        }
    }
}
