use std::{collections::HashMap, fmt::Write, iter::Peekable, str::Lines};

use chrono::{DateTime, Utc};
use either::Either;
use enum_map::EnumMap;
use fluxer_neptunium::{
    create_embed,
    exts::UserExt,
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
        bounties::{BountyNum, BountyState, BountySubmissionContent},
        bounty_stakeholders::BountyStakeholder,
        guilds::{BountyInfoKey, BountySubmissionFormat},
    },
};

pub mod confirmation;
pub mod user_arg;

macro_rules! get_bounty_num_from_args {
    ($ctx:expr, $args:expr, $operation:expr) => {{
        let args = $args.trim();
        let (num, rest) = args.split_once(' ').unwrap_or((args, ""));
        if num.is_empty() {
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
        let Ok(bounty_num): Result<$crate::db::bounties::BountyNum, ()> = std::str::FromStr::from_str(num) else {
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

#[expect(clippy::too_many_arguments, reason = "so what?")]
pub fn bounty_content_to_message(
    content: &BountySubmissionContent,
    created_by: either::Either<PartialUser, Id<UserMarker>>,
    format: &BountySubmissionFormat,
    bounty_number: BountyNum,
    created_at: DateTime<Utc>,
    state: BountyState,
    assigned_to: Option<Id<UserMarker>>,
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
    if let Some(assigned_to) = assigned_to {
        let assigned_to_string = format!("**Assigned to**\n<@{assigned_to}>\n");
        description.push_str(&assigned_to_string);
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
