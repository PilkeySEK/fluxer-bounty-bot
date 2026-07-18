use std::{
    env::consts::{FAMILY, OS},
    time::{Duration, Instant},
};

use chrono::{DateTime, TimeDelta, Utc};
use fluxer_neptunium::{create_embed, exts::MessageExt};
use pretty_duration::pretty_duration;
use tokio::runtime::{Handle, RuntimeFlavor};

use crate::{colors::DEFAULT, commands::CommandContext};

pub async fn ping(ctx: CommandContext<'_>, _args: &str) -> anyhow::Result<()> {
    #[expect(clippy::needless_pass_by_value)]
    fn make_description(
        runtime: Handle,
        latency: TimeDelta,
        send_latency: Option<Duration>,
        gateway_latency: Option<Duration>,
    ) -> String {
        let metrics = runtime.metrics();

        format!(
            "**Latency:** {} ms{}
            **Gateway latency:** {}
            **Running on:** {OS} ({FAMILY})
            
            __tokio metrics__
            **ID:** {}
            **Flavor:** {}
            **Number of workers:** {}
            **Number of alive tasks:** {}
            ```
            {}
            ```",
            latency.num_milliseconds(),
            if let Some(send_latency) = send_latency {
                format!(
                    "\n**Message send latency:** {} ms",
                    send_latency.as_millis()
                )
            } else {
                String::new()
            },
            if let Some(gateway_latency) = gateway_latency {
                format!("{} ms", gateway_latency.as_millis())
            } else {
                "*No response*".to_owned()
            },
            runtime.id(),
            match runtime.runtime_flavor() {
                RuntimeFlavor::CurrentThread => "Current Thread",
                RuntimeFlavor::MultiThread => "Multi Thread",
                _ => "Unknown",
            },
            metrics.num_workers(),
            metrics.num_alive_tasks(),
            (0..metrics.num_workers())
                .fold(Vec::new(), |mut vec, worker_index| {
                    vec.push(format!(
                        "Worker #{worker_index}: {} times parked, {} times unparked, total busy for {}",
                        metrics.worker_park_count(worker_index),
                        metrics.worker_park_unpark_count(worker_index),
                        pretty_duration(&metrics.worker_total_busy_duration(worker_index), None)
                    ));
                    vec
                })
                .join("\n"),
        )
        .lines()
        .map(str::trim)
        .collect::<Vec<_>>()
        .join("\n")
    }

    let latency = {
        let now = Utc::now();
        let created_at: DateTime<Utc> = ctx.message.timestamp.into();
        now.signed_duration_since(created_at)
    };

    let gateway_latency = ctx
        .ctx
        .measure_gateway_latency(Duration::from_secs(3))
        .await;

    let start = Instant::now();
    // Using the normal reply for this one to not delete it after 5 seconds.
    let reply = ctx
        .message
        .reply(
            ctx.ctx,
            create_embed!(
                title: "Pong!",
                description: make_description(Handle::current(), latency, None, gateway_latency),
                color: DEFAULT,
            ),
        )
        .await?;
    let end = Instant::now();

    reply.edit(ctx.ctx, create_embed!(
        title: "Pong!",
        description: make_description(Handle::current(), latency, Some(end - start), gateway_latency),
        color: DEFAULT,
    )).await?;

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
