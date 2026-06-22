use std::{sync::Arc, time::Duration};

use fluxer_neptunium::{
    cache::{Cached, CachedMessage},
    cached_payload::CachedMessageReactionAdd,
    exts::MessageExt,
    model::{
        guild::Emoji,
        id::{Id, marker::UserMarker},
    },
};
use tokio::sync::mpsc::unbounded_channel;

use crate::commands::CommandContext;

pub enum MaybeExpired<T> {
    NotExpired(T),
    Expired,
}

pub async fn confirmation(
    ctx: &CommandContext<'_>,
    confirmation_message: Cached<CachedMessage>,
    allowed_reactor: Id<UserMarker>,
) -> anyhow::Result<MaybeExpired<bool>> {
    const CONFIRM: &str = "✅";
    const CANCEL: &str = "❌";

    enum ConfirmationMessage {
        Ok(bool),
        Expired,
    }

    confirmation_message.add_reaction(ctx.ctx, CONFIRM).await?;
    confirmation_message.add_reaction(ctx.ctx, CANCEL).await?;

    let (handler_tx, mut rx) = unbounded_channel();
    let expiry_handler_tx = handler_tx.clone();
    ctx.register_reaction_handler(
        confirmation_message.id,
        move |event: Arc<CachedMessageReactionAdd>| {
            if event.user_id != allowed_reactor {
                return (false, Ok(()));
            }
            let Emoji::Default(emoji) = &event.emoji else {
                return (false, Ok(()));
            };
            if emoji == CONFIRM {
                let _ = handler_tx.send(ConfirmationMessage::Ok(true));
                (true, Ok(()))
            } else if emoji == CANCEL {
                let _ = handler_tx.send(ConfirmationMessage::Ok(false));
                (true, Ok(()))
            } else {
                (false, Ok(()))
            }
        },
        Some((
            Box::new(move || {
                Box::pin(async move {
                    let _ = expiry_handler_tx.send(ConfirmationMessage::Expired);
                    Ok(())
                })
            }),
            Duration::from_mins(10),
        )),
    );

    if let Some(message) = rx.recv().await {
        match message {
            ConfirmationMessage::Expired => Ok(MaybeExpired::Expired),
            ConfirmationMessage::Ok(confirmed) => {
                confirmation_message.delete(ctx.ctx).await?;
                Ok(MaybeExpired::NotExpired(confirmed))
            }
        }
    } else {
        tracing::warn!("The channel should not close.");
        Ok(MaybeExpired::Expired)
    }
}
