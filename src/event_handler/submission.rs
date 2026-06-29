use std::time::Duration;

use chrono::{DateTime, Utc};
use fluxer_neptunium::{
    cached_payload::CachedMessageCreate,
    create_embed,
    events::context::Context,
    exts::{ChannelExt, MessageExt},
    model::{
        id::{Id, marker::GuildMarker},
        time::timestamp::{Timestamp, representations::Iso8601},
        user::PartialUser,
    },
};

use crate::{
    colors::{FAILURE, SUCCESS},
    db::{
        DbManager,
        bounties::{BountyCreateData, BountyRelatedMessage, BountyState},
        guilds::{BountyInfoKey, GuildConfig},
    },
    util::{bounty_content_to_message, parse_message_content_as_submission},
};

#[expect(clippy::too_many_lines)]
pub async fn handle_submission_create(
    ctx: &Context,
    message: &CachedMessageCreate,
    user: PartialUser,
    guild_config: &GuildConfig,
    db: &DbManager,
    guild_id: Id<GuildMarker>,
) -> anyhow::Result<()> {
    let created_by_user_id = user.id;
    let parsed = parse_message_content_as_submission(
        &guild_config.bounty_submission_format,
        &message.content,
    );
    let mut missing_keys = Vec::new();
    for key in guild_config.bounty_submission_format.required {
        if !parsed.contains_key(&key) {
            missing_keys.push(key);
        }
    }
    if !missing_keys.is_empty() {
        let key_descriptors = missing_keys
            .into_iter()
            .map(|key| {
                guild_config.bounty_submission_format.titles[key]
                    .first()
                    .map_or("*no titles for key*", String::as_str)
            })
            .collect::<Vec<_>>()
            .join(", ");
        let reply_result = message.reply(ctx, create_embed!(
            description: format!("Your submission is missing the following: {key_descriptors}"),
            color: FAILURE,
        )).await;
        // Delete the original message after 10 seconds.
        tokio::time::sleep(Duration::from_secs(10)).await;
        message.delete(ctx).await?;
        reply_result?;
        return Ok(());
    }
    let deadline_timestamp = if let Some(timestamp_from_parsed) =
        parsed.get(&BountyInfoKey::Deadline)
    {
        let Some(timestamp) = Timestamp::<Iso8601>::parse(timestamp_from_parsed.trim()) else {
            let reply_result = message.reply(ctx, create_embed!(
                description: format!("The timestamp provided in the due date could not be parsed. Make sure to format it in the Fluxer timestamp format."),
                color: FAILURE,
            )).await;
            tokio::time::sleep(Duration::from_secs(10)).await;
            message.delete(ctx).await?;
            reply_result?.delete(ctx).await?;
            return Ok(());
        };
        Some(DateTime::from(timestamp))
    } else {
        None
    };
    let bounty_number = db.get_next_bounty_number_upsert(guild_id).await?;
    let now = Utc::now();
    let related_message = if let Some(approval_queue_channel) = guild_config.approval_queue_channel
    {
        let related_message = approval_queue_channel
            .send_message(
                ctx,
                bounty_content_to_message(
                    &parsed,
                    either::Either::Left(user),
                    &guild_config.bounty_submission_format,
                    bounty_number,
                    now,
                    BountyState::Pending,
                    None,
                    deadline_timestamp,
                    Vec::new(),
                    Vec::new(),
                ),
            )
            .await;
        match related_message {
            Err(e) => {
                tracing::error!("Error sending message in the approval queue channel: {e}");
                let reply_result = message.reply(ctx, create_embed!(
                    description: "Could not send the submission message in the approval queue. Submission was not created.",
                    color: FAILURE,
                )).await;
                message.delete(ctx).await?;
                tokio::time::sleep(Duration::from_secs(5)).await;
                reply_result?.delete(ctx).await?;
                return Ok(());
            }
            Ok(message) => Some(message),
        }
    } else {
        None
    };
    let bounty = BountyCreateData {
        bounty_number,
        assigned_to: None,
        content: parsed,
        guild_id,
        state: BountyState::Pending,
        created_by: created_by_user_id,
        created_at: now,
        related_message: related_message.map(|message| BountyRelatedMessage {
            message_id: message.id,
            channel_id: message.channel_id,
        }),
        deadline: deadline_timestamp,
    };
    db.create_bounty(bounty).await?;
    let message_send_result = message
        .channel_id
        .send_message(
            ctx,
            create_embed!(
                description: format!("Bounty `{bounty_number}` created (now awaiting approval)."),
                color: SUCCESS,
            ),
        )
        .await;
    tokio::time::sleep(Duration::from_secs(5)).await;
    message.delete(ctx).await?;
    message_send_result?.delete(ctx).await?;
    Ok(())
}
