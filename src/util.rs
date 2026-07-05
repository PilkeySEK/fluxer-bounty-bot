use std::{collections::HashMap, fmt::Write, iter::Peekable, str::Lines};

use anyhow::Context as _;
use chrono::{DateTime, Utc};
use either::Either;
use enum_map::EnumMap;
use fluxer_neptunium::{
    create_embed,
    events::context::Context,
    exts::{ChannelExt, UserExt},
    http::endpoints::channel::{DeleteMessage, EditMessage},
    model::{
        channel::message::embed::{EmbedFooter, MessageEmbed},
        id::{
            Id,
            marker::{ChannelMarker, GuildMarker, UserMarker},
        },
        time::timestamp::{Timestamp, TimestampDisplayType, representations::Iso8601},
        user::PartialUser,
    },
};

use crate::{
    AVATAR_URL_BASE, STATIC_BASE,
    colors::SUBMISSION_PENDING,
    db::{
        DbManager,
        bounties::{Bounty, BountyNum, BountyRelatedMessage, BountyState, BountySubmissionContent},
        bounty_assignee_queue::QueuedBountyAssignee,
        bounty_stakeholders::BountyStakeholder,
        guilds::{BountyInfoKey, BountySubmissionFormat, GuildConfig},
    },
};

pub mod confirmation;
pub mod user_arg;

macro_rules! get_bounty_num_from_args {
    ($ctx:expr, $args:expr, $operation:expr) => {{
        let args = $args.trim();
        let (num, rest) = args.split_once(' ').unwrap_or((args, ""));
        if num.is_empty() {
            $ctx.reply(
                    fluxer_neptunium::create_embed!(
                        description: format!("Provide a bounty ID to {} that bounty.", $operation),
                        color: $crate::colors::FAILURE,
                    ),
                )
                .await?;
            return Ok(());
        }
        let Ok(bounty_num): Result<$crate::db::bounties::BountyNum, ()> = std::str::FromStr::from_str(num) else {
            $ctx.reply(
                    fluxer_neptunium::create_embed!(
                        description: "Could not parse the bounty ID.",
                        color: $crate::colors::FAILURE,
                    ),
                )
                .await?;
            return Ok(());
        };
        (bounty_num, rest)
    }};
}

pub(crate) use get_bounty_num_from_args;

pub fn parse_channel_mention_or_id_or_link(
    input: &str,
) -> Option<(Option<Id<GuildMarker>>, Id<ChannelMarker>)> {
    let input = input.trim();
    if let Some(input) = input.strip_prefix("<#") {
        if let Some(input) = input.strip_suffix(">")
            && let Ok(id) = input.try_into()
        {
            Some((None, id))
        } else {
            None
        }
    } else if let Ok(id) = Id::try_from(input) {
        Some((None, id))
    } else {
        let mut parts = input.split('/').filter(|part| !part.is_empty());
        let channel_id_str = parts.next_back()?;
        let guild_id_str = parts.next_back()?;
        Some((
            Some(guild_id_str.try_into().ok()?),
            channel_id_str.try_into().ok()?,
        ))
    }
}

const TITLE_MARKER: &str = "## ";

/// Does not validate whether all required fields are present.
pub fn parse_message_content_as_submission(
    format: &BountySubmissionFormat,
    content: &str,
) -> BountySubmissionContent {
    fn parse_parts(mut lines: Peekable<Lines<'_>>) -> Vec<(&str, String)> {
        let mut parts = Vec::new();
        while let Some(next_line) = lines.next() {
            let next_line = next_line.trim();
            if let Some(title) = next_line.strip_prefix(TITLE_MARKER) {
                let title = title.trim();
                let mut line_content = Vec::new();
                while lines
                    .peek()
                    .is_some_and(|line| !line.trim().starts_with(TITLE_MARKER))
                {
                    let Some(next) = lines.next() else {
                        break;
                    };
                    line_content.push(next);
                }
                parts.push((title, line_content.join("\n").trim().to_owned()));
            }
        }
        parts
    }
    let titles = format
        .titles
        .iter()
        .map(|(k, v)| {
            (
                k,
                v.iter().map(|s| s.to_lowercase()).collect::<Vec<String>>(),
            )
        })
        .collect::<EnumMap<_, _>>();

    let parts = parse_parts(content.lines().peekable());
    let mut content = HashMap::new();
    for part in parts {
        let part_title = part.0.to_lowercase();
        for (key, titles) in &titles {
            if titles.iter().find(|title| *title == &part_title).is_some() {
                content.insert(key, part.1);
                break;
            }
        }
    }
    content
}

#[expect(clippy::too_many_lines)]
#[expect(clippy::too_many_arguments, reason = "so what?")]
pub fn bounty_content_to_message(
    content: &BountySubmissionContent,
    created_by: either::Either<PartialUser, Id<UserMarker>>,
    format: &BountySubmissionFormat,
    bounty_number: BountyNum,
    created_at: DateTime<Utc>,
    state: BountyState,
    mut assignees: Vec<QueuedBountyAssignee>,
    deadline: Option<DateTime<Utc>>,
    stakeholders: Vec<BountyStakeholder>,
) -> MessageEmbed {
    let mut content = content.iter().collect::<Vec<_>>();
    content.sort();
    let mut description = Vec::new();
    let mut title = None;
    for (key, value) in content {
        if *key == BountyInfoKey::Title {
            title = Some(value);
            continue;
        }
        if *key == BountyInfoKey::Deadline {
            continue;
        }
        let key_title = format.titles[*key]
            .first()
            .map_or("*no titles for key*", String::as_str);
        description.push(format!("## {key_title}\n{value}"));
    }
    let mut description = description.join("\n");
    description.push_str("\n===\n");
    description.push_str("**Assigned to**\n");
    if assignees.is_empty() {
        description.push_str("*No one*\n");
    } else {
        assignees.sort_by_key(|elem| elem.queued_at);
        let mut is_first = true;
        let assignees_string = assignees
            .into_iter()
            .map(|assignee| {
                if is_first {
                    is_first = false;
                    format!("[**Assigned**] <@{}>", assignee.user_id)
                } else {
                    format!("[**Queued**] <@{}>", assignee.user_id)
                }
            })
            .collect::<Vec<String>>()
            .join("\n");
        description.push_str(&assignees_string);
        description.push('\n');
    }
    if let Some(deadline) = deadline {
        // Maybe take the description from `content` instead? Seems super unnecessary though since it probably wouldn't change anyway in 99% of cases
        let deadline_string = format!(
            "**Due date**\n{}\n",
            Timestamp::<Iso8601>::from(deadline)
                .time_string(TimestampDisplayType::ShortDateAndTime)
        );
        description.push_str(&deadline_string);
    }
    if !stakeholders.is_empty() {
        description.push_str("**Bounty Amount (USD)**\n");
        let mut total = 0.0;
        for stakeholder in stakeholders {
            let amount = f64::from(stakeholder.amount);
            total += amount;
            if let Err(e) = writeln!(
                description,
                "`${:.2}` by <@{}>{}",
                amount / 100.0,
                stakeholder.user_id,
                if let Some(note) = stakeholder.note {
                    format!(" - {note}")
                } else {
                    String::new()
                }
            ) {
                tracing::warn!("Error calling writeln!(): {e}");
            }
        }
        if let Err(e) = writeln!(
            description,
            "**Total Bounty Amount (USD)**\n`${:.2}`",
            total / 100.0
        ) {
            tracing::warn!("Error calling writeln!(): {e}");
        }
    }

    let avatar_url = if let Either::Left(created_by) = &created_by {
        if let Some(avatar) = &created_by.avatar {
            format!("{AVATAR_URL_BASE}/{}/{avatar}.webp?size=128", created_by.id)
        } else {
            format!(
                "{STATIC_BASE}/avatars/{}.png",
                created_by.get_default_avatar_id(),
            )
        }
    } else {
        format!("{STATIC_BASE}/avatars/0.png")
    };

    let author_name = match created_by {
        Either::Left(created_by) => {
            format!(
                "{}#{} ({})",
                created_by.username, created_by.discriminator, created_by.id
            )
        }
        Either::Right(id) => id.to_string(),
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
            name: author_name,
            icon_url: avatar_url,
        }
    );
    embed.footer = Some(EmbedFooter {
        icon_url: None,
        proxy_icon_url: None,
        text: format!("{bounty_number} - {state}"),
    });
    embed.timestamp = Some(created_at.into());
    embed
}

pub fn update_bounty_message(
    ctx: &Context,
    db: &DbManager,
    guild_config: &GuildConfig,
    bounty: Bounty,
) {
    let ctx = ctx.clone();
    let db = db.clone();
    let guild_config = guild_config.clone();
    tokio::spawn(async move {
        let bounty_id = bounty.bounty_id;
        if let Err(e) = update_bounty_message_inner(&ctx, &db, &guild_config, bounty)
            .await
            .with_context(|| format!("Updating the bounty message for bounty {bounty_id}"))
        {
            tracing::error!("{e:?}");
        }
    });
}

/// Update the bounty message. Might delete the existing message, edit it and create a new one.
/// Will update the related message in the database too.
#[expect(clippy::too_many_lines)]
async fn update_bounty_message_inner(
    ctx: &Context,
    db: &DbManager,
    guild_config: &GuildConfig,
    bounty: Bounty,
) -> anyhow::Result<()> {
    let assignees = db.list_assignee_queue_for_bounty(bounty.bounty_id).await?;
    let assignees_is_empty = assignees.is_empty();
    let embed = bounty_content_to_message(
        &bounty.content,
        match bounty.created_by.get_user(ctx).await {
            Ok(user) => either::Either::Left(user.clone_inner()),
            Err(e) => {
                tracing::warn!("Error fetching user {}: {}", bounty.created_by, e);
                either::Either::Right(bounty.created_by)
            }
        },
        &guild_config.bounty_submission_format,
        bounty.bounty_number,
        bounty.created_at,
        bounty.state,
        assignees,
        bounty.deadline,
        db.list_bounty_stakeholders(bounty.bounty_id).await?,
    );

    let channel_id = if !assignees_is_empty && bounty.state == BountyState::Approved {
        guild_config.claimed_bounties_channel
    } else {
        match bounty.state {
            BountyState::Approved => guild_config.approved_bounties_channel,
            BountyState::Completed => guild_config.completed_bounties_channel,
            BountyState::Pending => guild_config.approval_queue_channel,
            BountyState::Rejected => guild_config.rejected_bounties_channel,
        }
    };

    if let Some(channel_id) = channel_id {
        if let Some(related_message) = bounty.related_message {
            if related_message.channel_id == channel_id {
                ctx.get_http_client()
                    .execute(EditMessage {
                        channel_id: related_message.channel_id,
                        message_id: related_message.message_id,
                        body: embed.into(),
                    })
                    .await
                    .with_context(|| {
                        format!(
                            "Editing existing message related to bounty {}",
                            bounty.bounty_id
                        )
                    })?;
            } else {
                if let Err(e) = ctx
                    .get_http_client()
                    .execute(DeleteMessage {
                        channel_id: related_message.channel_id,
                        message_id: related_message.message_id,
                    })
                    .await
                {
                    tracing::warn!(
                        "Error deleting bounty related message {} in channel {} related to bounty {}: {}",
                        related_message.message_id,
                        related_message.channel_id,
                        bounty.bounty_id,
                        e
                    );
                }
                let message = channel_id.send_message(ctx, embed).await.with_context(|| {
                    format!("Sending message related to bounty {}", bounty.bounty_id)
                })?;
                db.set_bounty_related_message(
                    bounty.bounty_id,
                    Some(BountyRelatedMessage {
                        channel_id: message.channel_id,
                        message_id: message.id,
                    }),
                )
                .await?;
            }
        } else {
            let message = channel_id.send_message(ctx, embed).await.with_context(|| {
                format!("Sending message related to bounty {}", bounty.bounty_id)
            })?;
            db.set_bounty_related_message(
                bounty.bounty_id,
                Some(BountyRelatedMessage {
                    channel_id: message.channel_id,
                    message_id: message.id,
                }),
            )
            .await?;
        }
    } else if let Some(related_message) = bounty.related_message {
        if let Err(e) = ctx
            .get_http_client()
            .execute(DeleteMessage {
                channel_id: related_message.channel_id,
                message_id: related_message.message_id,
            })
            .await
        {
            tracing::warn!(
                "Error deleting bounty related message {} in channel {} related to bounty {}: {}",
                related_message.message_id,
                related_message.channel_id,
                bounty.bounty_id,
                e
            );
        }
        db.set_bounty_related_message(bounty.bounty_id, None)
            .await?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::db::guilds::BountyInfoKey;

    use super::*;

    #[test]
    fn test_parse_message_content_as_submission() {
        let format = BountySubmissionFormat::default();

        {
            let content = "
            ## Title
            Some content
            ";
            assert_eq!(parse_message_content_as_submission(&format, content), {
                let mut map = HashMap::new();
                map.insert(BountyInfoKey::Title, "Some content".to_owned());
                map
            });
        }
        {
            let content = "## Bounty title
## Deadline
never™
or actually- yesterday!

## Amount
one miwwion dollahs";
            assert_eq!(parse_message_content_as_submission(&format, content), {
                let mut map = HashMap::new();
                map.insert(BountyInfoKey::Title, String::new());
                map.insert(
                    BountyInfoKey::Deadline,
                    "never™\nor actually- yesterday!".to_owned(),
                );
                map.insert(
                    BountyInfoKey::BountyAmount,
                    "one miwwion dollahs".to_owned(),
                );
                map
            });
        }
    }
}
