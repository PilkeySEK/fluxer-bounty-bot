use fluxer_neptunium::{
    create_embed,
    exts::{GuildExt, GuildMemberExt, MessageExt},
    model::{
        guild::permissions::Permissions,
        id::{
            Id,
            marker::{ChannelMarker, GuildMarker},
        },
    },
};

use crate::{
    colors::{DEFAULT, FAILURE, SUCCESS},
    commands::CommandContext,
    db::DbManager,
    util::parse_channel_mention_or_id_or_link,
};

pub async fn guild_config(ctx: CommandContext<'_>, args: &str) -> anyhow::Result<()> {
    // TODO: bounty managers and/or role stored in database to bypass this
    if !ctx
        .guild_member
        .has_permissions(ctx.ctx, Permissions::MANAGE_GUILD)
        .await?
    {
        ctx.message.reply(ctx.ctx, create_embed!(
            description: "You need \"Manage Community\" permissions to execute this command.",
            color: FAILURE,
        )).await?;
        return Ok(());
    }

    let (subcommand, args) = args.split_once(' ').unwrap_or((args, ""));

    match subcommand.trim() {
        "bounty-submission-channel" => {
            set_channel_common(
                ctx,
                args,
                DbManager::set_bounty_submission_channel,
                "bounty submission",
            )
            .await
        }
        "approval-queue-channel" => {
            set_channel_common(
                ctx,
                args,
                DbManager::set_approval_queue_channel,
                "approval queue",
            )
            .await
        }
        "approved-bounties-channel" => {
            set_channel_common(
                ctx,
                args,
                DbManager::set_approved_bounties_channel,
                "approved bounties",
            )
            .await
        }
        "claimed-bounties-channel" => {
            set_channel_common(
                ctx,
                args,
                DbManager::set_claimed_bounties_channel,
                "claimed bounties",
            )
            .await
        }
        "completed-bounties-channel" => {
            set_channel_common(
                ctx,
                args,
                DbManager::set_completed_bounties_channel,
                "completed bounties",
            )
            .await
        }
        "rejected-bounties-channel" | "denied-bounties-channel" => {
            set_channel_common(
                ctx,
                args,
                DbManager::set_rejected_bounties_channel,
                "rejected bounties",
            )
            .await
        }
        "" => reply_with_guild_config(ctx).await,
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

async fn set_channel_common<'a, F, Fut>(
    ctx: CommandContext<'a>,
    args: &str,
    f: F,
    channel_name: &str,
) -> anyhow::Result<()>
where
    F: Fn(&'a DbManager, Id<GuildMarker>, Option<Id<ChannelMarker>>) -> Fut,
    Fut: Future<Output = anyhow::Result<()>>,
{
    let args = args.trim();
    let channel_id = {
        if args.is_empty() || args == "reset" {
            None
        } else if let Some((_, channel_id)) = parse_channel_mention_or_id_or_link(args) {
            Some(channel_id)
        } else {
            ctx.message
                .reply(
                    ctx.ctx,
                    fluxer_neptunium::create_embed!(
                        description: "Could not parse the channel.",
                        color: FAILURE,
                    ),
                )
                .await?;
            return Ok(());
        }
    };
    if let Some(channel_id) = channel_id {
        let guild_channels = ctx.guild_id.list_channels(ctx.ctx).await?;
        if guild_channels
            .iter()
            .find(|channel| channel.id == channel_id)
            .is_none()
        {
            ctx.message.reply(ctx.ctx, create_embed!(
                description: "That channel does not exist in this community or I don't have access to it.",
                color: FAILURE,
            )).await?;
            return Ok(());
        }
    }

    f(ctx.db, ctx.guild_id, channel_id).await?;

    ctx.message
        .reply(
            ctx.ctx,
            create_embed!(
                description: if let Some(channel_id) = channel_id {
                    format!("Set the {channel_name} channel to <#{channel_id}>.")
                } else {
                    format!("Unset the {channel_name} channel.")
                },
                color: SUCCESS,
            ),
        )
        .await?;
    Ok(())
}

async fn reply_with_guild_config(ctx: CommandContext<'_>) -> anyhow::Result<()> {
    let config_string = format!(
        "**Total bounties ever created:** `{}`\n__Channels__\n**Bounty Submissions:** {}\n**Approval Queue:** {}\n**Claimed Bounties:** {}\n**Completed Bounties:** {}\n**Denied Bounties:** {}",
        ctx.guild_config.current_bounty_number,
        ctx.guild_config
            .bounty_submission_channel
            .map_or_else(|| "*none*".to_owned(), |id| format!("<#{id}>")),
        ctx.guild_config
            .approval_queue_channel
            .map_or_else(|| "*none*".to_owned(), |id| format!("<#{id}>")),
        ctx.guild_config
            .claimed_bounties_channel
            .map_or_else(|| "*none*".to_owned(), |id| format!("<#{id}>")),
        ctx.guild_config
            .completed_bounties_channel
            .map_or_else(|| "*none*".to_owned(), |id| format!("<#{id}>")),
        ctx.guild_config
            .rejected_bounties_channel
            .map_or_else(|| "*none*".to_owned(), |id| format!("<#{id}>")),
    );
    ctx.message
        .reply(
            ctx.ctx,
            create_embed!(
                title: "Bot configuration",
                description: config_string,
                color: DEFAULT,
            ),
        )
        .await?;
    Ok(())
}
