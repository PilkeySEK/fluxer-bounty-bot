use std::time::Duration;

use chrono::{DateTime, Utc};
use fluxer_neptunium::{create_embed, exts::MessageExt};

use crate::{colors::DEFAULT, commands::CommandContext};

pub async fn ping(ctx: CommandContext<'_>, _args: &str) -> anyhow::Result<()> {
    let latency = {
        let now = Utc::now();
        let created_at: DateTime<Utc> = ctx.message.timestamp.into();
        now.signed_duration_since(created_at)
    };
    // Using the normal reply for this one to not delete it after 5 seconds.
    ctx.message
        .reply(
            ctx.ctx,
            create_embed!(
                title: "Pong!",
                description: format!("**Latency:** {} ms", latency.num_milliseconds()),
                color: DEFAULT,
            ),
        )
        .await?;
    Ok(())
}

pub async fn bounty_workflow(ctx: CommandContext<'_>, _args: &str) -> anyhow::Result<()> {
    ctx.message
        .reply(ctx.ctx, ctx.bounty_workflow_image_url)
        .await?;
    Ok(())
}

pub async fn help(ctx: CommandContext<'_>, _args: &str) -> anyhow::Result<()> {
    ctx.reply_with_deletion_duration(create_embed!(
        description: "The documentation for all commands can be found in the GitHub repository, DOCS.md:\nhttps://github.com/PilkeySEK/fluxer-bounty-bot/blob/master/DOCS.md",
        color: DEFAULT,
    ), Duration::from_secs(10)).await?;
    Ok(())
}
