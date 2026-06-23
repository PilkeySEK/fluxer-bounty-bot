use fluxer_neptunium::{
    create_embed,
    exts::{ChannelExt, MessageExt, UserExt},
    http::endpoints::channel::DeleteMessage,
    model::id::{Id, marker::ChannelMarker},
};

use crate::{
    colors::{FAILURE, SUCCESS},
    commands::CommandContext,
    db::bounties::{BountyRelatedMessage, BountyState},
    util::bounty_content_to_message,
};

// TODO: When replying to a message and not providing the bounty number, try to get the bounty from the replied-to message.

macro_rules! get_bounty_num_from_args {
    ($ctx:expr, $args:expr, $operation:expr) => {{
        let args = $args.trim();
        if args.is_empty() {
            fluxer_neptunium::exts::MessageExt::reply(
                    &*$ctx.message.message,
                    $ctx.ctx,
                    fluxer_neptunium::create_embed!(
                        description: format!("Provide a bounty ID to {} that bounty.", $operation),
                        color: $crate::colors::FAILURE,
                    ),
                )
                .await?;
            return Ok(());
        }
        let Ok(bounty_num): Result<$crate::db::bounties::BountyNum, ()> = std::str::FromStr::from_str(args) else {
            fluxer_neptunium::exts::MessageExt::reply(
                    &*$ctx.message.message,
                    $ctx.ctx,
                    fluxer_neptunium::create_embed!(
                        description: "Could not parse the bounty ID.",
                        color: $crate::colors::FAILURE,
                    ),
                )
                .await?;
            return Ok(());
        };
        bounty_num
    }};
}

pub async fn complete_bounty(ctx: CommandContext<'_>, args: &str) -> anyhow::Result<()> {
    let new_channel = ctx.guild_config.completed_bounties_channel;
    set_bounty_state_common(ctx, args, "complete", BountyState::Completed, new_channel).await
}

pub async fn approve_bounty(ctx: CommandContext<'_>, args: &str) -> anyhow::Result<()> {
    let new_channel = ctx.guild_config.approved_bounties_channel;
    set_bounty_state_common(ctx, args, "approve", BountyState::Approved, new_channel).await
}

pub async fn reject_bounty(ctx: CommandContext<'_>, args: &str) -> anyhow::Result<()> {
    let new_channel = ctx.guild_config.rejected_bounties_channel;
    set_bounty_state_common(ctx, args, "reject", BountyState::Rejected, new_channel).await
}

async fn set_bounty_state_common(
    ctx: CommandContext<'_>,
    args: &str,
    operation: &str,
    new_state: BountyState,
    new_channel: Option<Id<ChannelMarker>>,
) -> anyhow::Result<()> {
    let bounty_num = get_bounty_num_from_args!(ctx, args, operation);

    let Some(bounty) = ctx.db.get_bounty(ctx.guild_id, bounty_num).await? else {
        ctx.message
            .reply(
                ctx.ctx,
                create_embed!(
                    description: "A bounty with that ID does not exist.",
                    color: FAILURE,
                ),
            )
            .await?;
        return Ok(());
    };

    if bounty.state == new_state {
        ctx.message
            .reply(
                ctx.ctx,
                create_embed!(
                    description: format!("The bounty is already {}.", new_state.to_string().to_lowercase()),
                    color: FAILURE,
                ),
            )
            .await?;
        return Ok(());
    }

    let new_related_message = if let Some(new_channel) = new_channel {
        let created_by = match bounty.created_by.get_user(ctx.ctx).await {
            Ok(created_by) => either::Either::Left(created_by.clone_inner()),
            Err(e) => {
                tracing::warn!("Error fetching user {}: {e}", bounty.created_by);
                either::Either::Right(bounty.created_by)
            }
        };
        Some(
            new_channel
                .send_message(
                    ctx.ctx,
                    bounty_content_to_message(
                        &bounty.content,
                        created_by,
                        &ctx.guild_config.bounty_submission_format,
                        bounty.bounty_number,
                        bounty.created_at,
                        new_state,
                    ),
                )
                .await?,
        )
    } else {
        None
    };
    if let Some(related_message) = bounty.related_message
        && let Err(e) = ctx
            .ctx
            .get_http_client()
            .execute(DeleteMessage {
                channel_id: related_message.channel_id,
                message_id: related_message.message_id,
            })
            .await
    {
        tracing::error!("Error deleting related message: {e}");
    }

    ctx.db
        .set_bounty_state_and_related_message(
            ctx.guild_id,
            bounty_num,
            new_state,
            new_related_message.map(|msg| BountyRelatedMessage {
                message_id: msg.id,
                channel_id: msg.channel_id,
            }),
        )
        .await?;

    ctx.message
        .reply(
            ctx.ctx,
            create_embed!(
                description: format!("Updated `{bounty_num}`, it is now {}", new_state.to_string().to_lowercase()),
                color: SUCCESS,
            ),
        )
        .await?;

    Ok(())
}
