use fluxer_neptunium::{
    create_embed,
    exts::MessageExt,
    model::{
        gateway::payload::outgoing::{RequestGuildMembers, RequestGuildMembersQuery},
        id::{Id, marker::UserMarker},
    },
};

use crate::{
    colors::DEFAULT,
    commands::CommandContext,
    util::confirmation::{MaybeExpired, confirmation},
};

pub fn parse_mention_or_id(input: &str) -> Option<Id<UserMarker>> {
    Id::try_from(
        input
            .trim_start()
            .strip_prefix("<@")
            .and_then(|input| input.strip_suffix('>'))
            .unwrap_or(input),
    )
    .ok()
}

/// Expect the input to already be split from the rest of the args, so have no spaces.
///
/// May also return expired when the user rejected the confirmation dialog that may happen.
pub async fn parse_user_arg(
    ctx: &CommandContext<'_>,
    input: &str,
) -> anyhow::Result<MaybeExpired<Option<Id<UserMarker>>>> {
    // Result<Option<Id<UserMarker>>, CommandError> {
    if let Some(user_id) = parse_mention_or_id(input) {
        Ok(MaybeExpired::NotExpired(Some(user_id)))
    } else {
        let users = ctx
            .ctx
            .request_guild_members(RequestGuildMembers {
                guild_ids: vec![ctx.guild_id],
                query: RequestGuildMembersQuery::Text(input.to_string()),
                limit: Some(1),
                nonce: None,
                presences: None,
            })
            .await?;

        let Some(member) = users.first() else {
            return Ok(MaybeExpired::NotExpired(None));
        };

        let confirmation_reply = ctx
            .message
            .reply(
                ctx.ctx,
                create_embed!(
                    description: format!(
                        "Is <@{}> (`{}#{}`) the user you're looking for?",
                        member.id,
                        member.user.username,
                        member.user.discriminator,
                    ),
                    color: DEFAULT,
                ),
            )
            .await?;
        let confirmation_result =
            confirmation(ctx, confirmation_reply, ctx.message.author.id).await?;
        match confirmation_result {
            MaybeExpired::NotExpired(true) => Ok(MaybeExpired::NotExpired(Some(member.id))),
            MaybeExpired::Expired | MaybeExpired::NotExpired(false) => Ok(MaybeExpired::Expired),
        }
    }
}
