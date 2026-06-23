use std::str::FromStr;

use anyhow::Context;
use fluxer_neptunium::{
    create_embed,
    exts::{GuildMemberExt, MessageExt},
    http::endpoints::channel::DeleteMessage,
    model::guild::permissions::Permissions,
};

use crate::{
    colors::{DEFAULT, FAILURE, SUCCESS},
    commands::CommandContext,
    db::bounties::{BountyNum, BountyRelatedMessage, BountyState},
    util::confirmation::{MaybeExpired, confirmation},
};

macro_rules! get_bounty_num_from_args {
    ($ctx:expr, $args:expr, $operation:expr) => {{
        let args = $args.trim();
        if args.is_empty() {
            $ctx.message
                .reply(
                    $ctx.ctx,
                    create_embed!(
                        description: format!("Provide a bounty ID to {} that bounty.", $operation),
                        color: FAILURE,
                    ),
                )
                .await?;
            return Ok(());
        }
        let Ok(bounty_num) = BountyNum::from_str(args) else {
            $ctx.message
                .reply(
                    $ctx.ctx,
                    create_embed!(
                        description: "Could not parse the bounty ID.",
                        color: FAILURE,
                    ),
                )
                .await?;
            return Ok(());
        };
        bounty_num
    }};
}

pub async fn bounty_management(ctx: CommandContext<'_>, args: &'_ str) -> anyhow::Result<()> {
    let (subcommand, args) = args.split_once(' ').unwrap_or((args, ""));

    match subcommand {
        "delete" | "remove" | "rm" | "del" => delete_bounty(ctx, args).await,
        "approve" | "accept" => approve_bounty(ctx, args).await,
        _ => {
            ctx.message
                .reply(
                    ctx.ctx,
                    create_embed!(
                        description: "Unknown subcommand",
                        color: FAILURE,
                    ),
                )
                .await?;
            Ok(())
        }
    }
}

async fn approve_bounty(ctx: CommandContext<'_>, args: &str) -> anyhow::Result<()> {
    let bounty_num = get_bounty_num_from_args!(ctx, args, "approve");
    let bounty_data = ctx.db.get_bounty(ctx.guild_id, bounty_num).await?;
    let Some(bounty_data) = bounty_data else {
        ctx.message
            .reply(
                ctx.ctx,
                create_embed!(
                    description: "Could not find that bounty.",
                    color: FAILURE,
                ),
            )
            .await?;
        return Ok(());
    };

    match bounty_data.state {
        BountyState::Approved => {
            ctx.message
                .reply(
                    ctx.ctx,
                    create_embed!(
                        description: "That bounty is already approved.",
                        color: FAILURE,
                    ),
                )
                .await?;
            return Ok(());
        }
        BountyState::Rejected => {
            let message = ctx.message.reply(ctx.ctx, create_embed!(
                description: "That bounty has been rejected. Do you want to accept it instead?",
                color: DEFAULT,
            )).await?;
            let confirmation_result = confirmation(&ctx, message, ctx.guild_member.id).await?;
            let MaybeExpired::NotExpired(true) = confirmation_result else {
                return Ok(());
            };
        }
        BountyState::Finished => {
            let message = ctx.message.reply(ctx.ctx, create_embed!(
                description: "That bounty has been marked as finished. Do you want to move it back to accepted state?",
                color: DEFAULT,
            )).await?;
            let confirmation_result = confirmation(&ctx, message, ctx.guild_member.id).await?;
            let MaybeExpired::NotExpired(true) = confirmation_result else {
                return Ok(());
            };
        }
        BountyState::Pending => {}
    }

    ctx.db
        .set_bounty_state(ctx.guild_id, bounty_num, BountyState::Approved)
        .await?;

    ctx.message
        .reply(
            ctx.ctx,
            create_embed!(
                description: format!("Marked bounty `{bounty_num}` as accepted."),
                color: SUCCESS,
            ),
        )
        .await?;

    Ok(())
}

async fn delete_bounty(ctx: CommandContext<'_>, args: &str) -> anyhow::Result<()> {
    let bounty_num = get_bounty_num_from_args!(ctx, args, "delete");
    let bounty_data = ctx
        .db
        .delete_and_return_bounty(ctx.guild_id, bounty_num)
        .await
        .with_context(|| format!("Failed to delete bounty with number {bounty_num}"))?;
    let Some(bounty_data) = bounty_data else {
        ctx.message
            .reply(
                ctx.ctx,
                create_embed!(
                    description: "Bounty not found.",
                    color: FAILURE,
                ),
            )
            .await?;
        return Ok(());
    };
    if let Some(BountyRelatedMessage {
        channel_id,
        message_id,
    }) = bounty_data.related_message
    {
        ctx.ctx.get_http_client()
            .execute(DeleteMessage {
                channel_id,
                message_id,
            })
            .await
            .with_context(|| format!("Failed to delete related message of bounty {} message_id {message_id} and channel_id {channel_id}", bounty_data.bounty_id))?;
    }
    ctx.message
        .reply(
            ctx.ctx,
            create_embed!(
                description: format!("Deleted bounty `{bounty_num}`."),
                color: SUCCESS,
            ),
        )
        .await?;
    Ok(())
}
