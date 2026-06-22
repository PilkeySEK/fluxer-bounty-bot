use std::time::Duration;

use chrono::{DateTime, Utc};
use fluxer_neptunium::{
    cached_payload::CachedMessageCreate,
    create_embed,
    events::context::Context,
    exts::{ChannelExt, MessageExt, UserExt},
    http::endpoints::channel::CreateMessageBody,
    model::{
        channel::message::embed::EmbedFooter,
        id::{Id, marker::GuildMarker},
        user::PartialUser,
    },
};

use crate::{
    AVATAR_URL_BASE, STATIC_BASE,
    colors::{FAILURE, SUBMISSION_PENDING, SUCCESS},
    db::{
        DbManager,
        bounties::{
            BountyCreateData, BountyNum, BountyRelatedMessage, BountyState, BountySubmissionContent,
        },
        guilds::{BountyInfoKey, BountySubmissionFormat, GuildConfig},
    },
    util::parse_message_content_as_submission,
};

pub async fn handle_submission_create(
    ctx: &Context,
    message: &CachedMessageCreate,
    user: &PartialUser,
    guild_config: &GuildConfig,
    db: &DbManager,
    guild_id: Id<GuildMarker>,
) -> anyhow::Result<()> {
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
    let bounty_number = db.get_next_bounty_number_upsert(guild_id).await?;
    let now = Utc::now();
    let related_message = if let Some(approval_queue_channel) = guild_config.approval_queue_channel
    {
        let related_message = approval_queue_channel
            .send_message(
                ctx,
                bounty_content_to_message(
                    &parsed,
                    user,
                    &guild_config.bounty_submission_format,
                    bounty_number,
                    now,
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
        claimed_by: None,
        content: parsed,
        guild_id,
        state: BountyState::Pending,
        created_by: user.id,
        created_at: now,
        related_message: related_message.map(|message| BountyRelatedMessage {
            message_id: message.id,
            channel_id: message.channel_id,
        }),
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

fn bounty_content_to_message(
    content: &BountySubmissionContent,
    created_by: &PartialUser,
    format: &BountySubmissionFormat,
    bounty_number: BountyNum,
    created_at: DateTime<Utc>,
) -> impl Into<CreateMessageBody> {
    let mut content = content.iter().collect::<Vec<_>>();
    content.sort();
    let mut description = Vec::new();
    let mut title = None;
    for (key, value) in content {
        if *key == BountyInfoKey::Title {
            title = Some(value);
            continue;
        }
        let key_title = format.titles[*key]
            .first()
            .map_or("*no titles for key*", String::as_str);
        description.push(format!("## {key_title}\n{value}"));
    }
    let description = description.join("\n");

    let avatar_url = if let Some(avatar) = &created_by.avatar {
        format!("{AVATAR_URL_BASE}/{}/{avatar}.webp?size=128", created_by.id)
    } else {
        format!(
            "{STATIC_BASE}/avatars/{}.png",
            created_by.get_default_avatar_id()
        )
    };

    let mut embed = create_embed!(
        title: if let Some(title) = title {
            title.as_str()
        } else {
            "*No title*"
        },
        description: description,
        color: SUBMISSION_PENDING,
        author: {
            name: format!("{}#{} ({})", created_by.username, created_by.discriminator, created_by.id),
            icon_url: avatar_url,
        }
    );
    embed.footer = Some(EmbedFooter {
        icon_url: None,
        proxy_icon_url: None,
        text: bounty_number.to_string(),
    });
    embed.timestamp = Some(created_at.into());
    embed
}
