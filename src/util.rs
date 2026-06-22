use std::{collections::HashMap, iter::Peekable, str::Lines};

use enum_map::EnumMap;
use fluxer_neptunium::model::id::{
    Id,
    marker::{ChannelMarker, GuildMarker},
};

use crate::db::{bounties::BountySubmissionContent, guilds::BountySubmissionFormat};

pub mod confirmation;

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
