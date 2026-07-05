use std::ops::ControlFlow;

use anyhow::Context as _;
use chrono::{DateTime, Utc};
use enum_map::enum_map;
use fluxer_neptunium::{
    create_embed,
    exts::{MessageExt, UserExt},
    http::endpoints::channel::{DeleteMessage, EditMessage},
    model::{
        id::{Id, marker::UserMarker},
        time::timestamp::{Timestamp, representations::Iso8601},
    },
};

use crate::{
    colors::{DEFAULT, FAILURE, SUCCESS},
    commands::CommandContext,
    db::{
        bounties::{BountyRelatedMessage, BountyState},
        guilds::BountyInfoKey,
    },
    util::{
        bounty_content_to_message,
        confirmation::{MaybeExpired, confirmation},
        get_bounty_num_from_args, update_bounty_message,
        user_arg::parse_user_arg,
    },
};

// TODO: When replying to a message and not providing the bounty number, try to get the bounty from the replied-to message.

pub async fn complete_bounty(ctx: CommandContext<'_>, args: &str) -> anyhow::Result<()> {
    set_bounty_state_common(ctx, args, "complete", BountyState::Completed).await
}

pub async fn approve_bounty(ctx: CommandContext<'_>, args: &str) -> anyhow::Result<()> {
    set_bounty_state_common(ctx, args, "approve", BountyState::Approved).await
}

pub async fn reject_bounty(ctx: CommandContext<'_>, args: &str) -> anyhow::Result<()> {
    set_bounty_state_common(ctx, args, "reject", BountyState::Rejected).await
}

async fn set_bounty_state_common(
    ctx: CommandContext<'_>,
    args: &str,
    operation: &str,
    new_state: BountyState,
) -> anyhow::Result<()> {
    let (bounty_num, _rest) = get_bounty_num_from_args!(ctx, args, operation);

    let Some(mut bounty) = ctx.db.get_bounty(ctx.guild_id, bounty_num).await? else {
        ctx.reply(create_embed!(
            description: "A bounty with that ID does not exist.",
            color: FAILURE,
        ))
        .await?;
        return Ok(());
    };

    if bounty.state == new_state {
        ctx.reply(create_embed!(
            description: format!("The bounty is already {}.", new_state.to_string().to_lowercase()),
            color: FAILURE,
        ))
        .await?;
        return Ok(());
    }

    ctx.db.set_bounty_state(bounty.bounty_id, new_state).await?;
    bounty.state = new_state;
    update_bounty_message(ctx.ctx, ctx.db, ctx.guild_config, bounty);

    ctx.reply(
            create_embed!(
                description: format!("Updated `{bounty_num}`, it is now {}", new_state.to_string().to_lowercase()),
                color: SUCCESS,
            ),
        )
        .await?;

    Ok(())
}

pub async fn delete_bounty(ctx: CommandContext<'_>, args: &str) -> anyhow::Result<()> {
    let (bounty_num, _rest) = get_bounty_num_from_args!(ctx, args, "delete");
    let MaybeExpired::NotExpired(true) = confirmation(
        &ctx,
        ctx.message.reply(ctx.ctx, create_embed!(
            description: format!("This will delete the bounty `{bounty_num}`, which cannot be undone.\nAre you sure?"),
            color: DEFAULT,
        )).await?,
        ctx.guild_member.id)
        .await?
    else {
        return Ok(());
    };
    let bounty_data = ctx
        .db
        .delete_and_return_bounty(ctx.guild_id, bounty_num)
        .await
        .with_context(|| format!("Failed to delete bounty with number {bounty_num}"))?;
    let Some(bounty_data) = bounty_data else {
        ctx.reply(create_embed!(
            description: "Bounty not found.",
            color: FAILURE,
        ))
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
    ctx.reply(create_embed!(
        description: format!("Deleted bounty `{bounty_num}`."),
        color: SUCCESS,
    ))
    .await?;
    Ok(())
}

pub async fn assign_to_bounty(ctx: CommandContext<'_>, args: &str) -> anyhow::Result<()> {
    let (bounty_num, rest) = get_bounty_num_from_args!(ctx, args, "assign");

    let MaybeExpired::NotExpired(user_id) = parse_user_arg(&ctx, rest.trim()).await? else {
        return Ok(());
    };
    let Some(user_id) = user_id else {
        ctx.reply(create_embed!(
            description: "Could not find a user matching the query.",
            color: FAILURE,
        ))
        .await?;
        return Ok(());
    };
    let bounty = ctx.db.get_bounty(ctx.guild_id, bounty_num).await?;
    let Some(bounty) = bounty else {
        ctx.reply(create_embed!(
            description: "A bounty with that ID does not exist.",
            color: FAILURE,
        ))
        .await?;
        return Ok(());
    };

    if try_assign_to_bounty(&ctx, bounty.bounty_id, user_id)
        .await?
        .is_break()
    {
        return Ok(());
    }
    update_bounty_message(ctx.ctx, ctx.db, ctx.guild_config, bounty);

    ctx.reply(create_embed!(
        description: format!("Assigned <@{user_id}> to the bounty `{bounty_num}`."),
        color: SUCCESS,
    ))
    .await?;

    Ok(())
}

pub async fn self_assign_to_bounty(ctx: CommandContext<'_>, args: &str) -> anyhow::Result<()> {
    let (bounty_num, _rest) = get_bounty_num_from_args!(ctx, args, "assign");
    let user_id = ctx.guild_member.id;

    let Some(bounty) = ctx.db.get_bounty(ctx.guild_id, bounty_num).await? else {
        ctx.reply(create_embed!(
            description: "A bounty with that ID does not exist.",
            color: FAILURE,
        ))
        .await?;
        return Ok(());
    };

    if bounty.state != BountyState::Approved {
        ctx.reply(create_embed!(
            description: "You cannot assign yourself to this bounty because it is not in the correct state.",
            color: FAILURE,
        )).await?;
        return Ok(());
    }

    if try_assign_to_bounty(&ctx, bounty.bounty_id, user_id)
        .await?
        .is_break()
    {
        return Ok(());
    }
    update_bounty_message(ctx.ctx, ctx.db, ctx.guild_config, bounty);

    ctx.reply(create_embed!(
        description: format!("Assigned yourself to the bounty `{bounty_num}`."),
        color: SUCCESS,
    ))
    .await?;

    Ok(())
}

pub async fn unassign_from_bounty(ctx: CommandContext<'_>, args: &str) -> anyhow::Result<()> {
    let (bounty_num, rest) = get_bounty_num_from_args!(ctx, args, "unassign");

    let MaybeExpired::NotExpired(user_id) = parse_user_arg(&ctx, rest.trim()).await? else {
        return Ok(());
    };
    let Some(user_id) = user_id else {
        ctx.reply(create_embed!(
            description: "Could not find a user matching the query.",
            color: FAILURE,
        ))
        .await?;
        return Ok(());
    };

    let bounty = ctx.db.get_bounty(ctx.guild_id, bounty_num).await?;
    let Some(bounty) = bounty else {
        ctx.reply(create_embed!(
            description: "A bounty with that ID does not exist.",
            color: FAILURE,
        ))
        .await?;
        return Ok(());
    };
    let query_result = ctx
        .db
        .remove_user_from_assignee_queue(bounty.bounty_id, user_id)
        .await?;
    if query_result.rows_affected() == 0 {
        ctx.reply(create_embed!(
            description: "The user wasn't assigned or queued for that bounty.",
            color: FAILURE,
        ))
        .await?;
        return Ok(());
    }

    update_bounty_message(ctx.ctx, ctx.db, ctx.guild_config, bounty);

    ctx.reply(create_embed!(
        description: format!("Unassigned <@{user_id}> from the bounty `{bounty_num}`."),
        color: SUCCESS,
    ))
    .await?;

    Ok(())
}

pub async fn self_unassign_from_bounty(ctx: CommandContext<'_>, args: &str) -> anyhow::Result<()> {
    let (bounty_num, _rest) = get_bounty_num_from_args!(ctx, args, "unassign");

    let bounty = ctx.db.get_bounty(ctx.guild_id, bounty_num).await?;
    let Some(bounty) = bounty else {
        ctx.reply(create_embed!(
            description: "A bounty with that ID does not exist.",
            color: FAILURE,
        ))
        .await?;
        return Ok(());
    };
    let query_result = ctx
        .db
        .remove_user_from_assignee_queue(bounty.bounty_id, ctx.guild_member.id)
        .await?;
    if query_result.rows_affected() == 0 {
        ctx.reply(create_embed!(
            description: "You are not assigned to that bounty.",
            color: FAILURE,
        ))
        .await?;
        return Ok(());
    }

    update_bounty_message(ctx.ctx, ctx.db, ctx.guild_config, bounty);

    ctx.reply(create_embed!(
        description: format!("Unassigned you from `{bounty_num}`."),
        color: SUCCESS,
    ))
    .await?;
    Ok(())
}

#[expect(clippy::too_many_lines)]
pub async fn edit_bounty(ctx: CommandContext<'_>, args: &str) -> anyhow::Result<()> {
    let field_descriptors = enum_map! {
        BountyInfoKey::Title => "title",
        BountyInfoKey::AdditionalInfo => "additional-info",
        BountyInfoKey::BountyAmount => "proposed-amount",
        BountyInfoKey::Deadline => "due-date",
        BountyInfoKey::IssueUrl => "issue-url",
        BountyInfoKey::JudgingCriteria => "judging-criteria",
    };
    let (bounty_num, rest) = get_bounty_num_from_args!(ctx, args, "edit");
    let rest = rest.trim();
    let (field_descriptor, value) = rest.split_once(' ').unwrap_or((rest, ""));
    let value = value.trim();
    let field_descriptor = field_descriptors.iter().find_map(|(k, v)| {
        if *v == field_descriptor {
            Some(k)
        } else {
            None
        }
    });
    let Some(field_key) = field_descriptor else {
        let descriptor_list = field_descriptors
            .values()
            .map(|v| format!("`{v}`"))
            .collect::<Vec<String>>()
            .join(", ");
        ctx.reply(create_embed!(
            description: format!("Unknown field. Possible values are: {descriptor_list}"),
            color: FAILURE,
        ))
        .await?;
        return Ok(());
    };
    let bounty = ctx.db.get_bounty(ctx.guild_id, bounty_num).await?;
    let Some(mut bounty) = bounty else {
        ctx.reply(create_embed!(
            description: "A bounty with that ID does not exist.",
            color: FAILURE,
        ))
        .await?;
        return Ok(());
    };
    if field_key == BountyInfoKey::Deadline {
        let deadline_timestamp = if value.is_empty() {
            None
        } else {
            let Some(timestamp) = Timestamp::<Iso8601>::parse(value) else {
                ctx.reply(create_embed!(
                    description: format!("The timestamp provided in the due date could not be parsed. Make sure to format it in the Fluxer timestamp format."),
                    color: FAILURE,
                )).await?;
                return Ok(());
            };
            Some(DateTime::from(timestamp))
        };
        bounty.deadline = deadline_timestamp;
        ctx.db
            .set_bounty_deadine(bounty.bounty_id, deadline_timestamp)
            .await?;
    } else {
        if value.is_empty() {
            bounty.content.remove(&field_key);
        } else {
            bounty.content.insert(field_key, value.to_owned());
        }
        ctx.db
            .set_bounty_content(bounty.bounty_id, &bounty.content)
            .await?;
    }

    if let Some(related_message) = bounty.related_message {
        let created_by = match bounty.created_by.get_user(ctx.ctx).await {
            Ok(created_by) => either::Either::Left(created_by.clone_inner()),
            Err(e) => {
                tracing::warn!("Error fetching user {}: {e}", bounty.created_by);
                either::Either::Right(bounty.created_by)
            }
        };
        let embed = bounty_content_to_message(
            &bounty.content,
            created_by,
            &ctx.guild_config.bounty_submission_format,
            bounty_num,
            bounty.created_at,
            bounty.state,
            ctx.db
                .list_assignee_queue_for_bounty(bounty.bounty_id)
                .await?,
            bounty.deadline,
            ctx.db.list_bounty_stakeholders(bounty.bounty_id).await?,
        );
        ctx.ctx
            .get_http_client()
            .execute(EditMessage {
                message_id: related_message.message_id,
                channel_id: related_message.channel_id,
                body: embed.into(),
            })
            .await?;
    }

    ctx.reply(create_embed!(
        description: format!("Updated the bounty content of `{bounty_num}`."),
        color: SUCCESS,
    ))
    .await?;

    Ok(())
}

async fn try_assign_to_bounty(
    ctx: &CommandContext<'_>,
    bounty_id: i64,
    user_id: Id<UserMarker>,
) -> anyhow::Result<ControlFlow<()>> {
    const BOUNTY_ASSIGNEES_MAX_COUNT: i64 = 10;

    let count = ctx.db.count_assignees_for_bounty(bounty_id).await?;
    if count >= BOUNTY_ASSIGNEES_MAX_COUNT {
        ctx.reply(create_embed!(
            description: "The queue is full.",
            color: FAILURE,
        ))
        .await?;
        return Ok(ControlFlow::Break(()));
    }
    let query_result = ctx
        .db
        .add_user_to_assignee_queue(bounty_id, user_id, Utc::now())
        .await?;
    if query_result.rows_affected() == 0 {
        ctx.reply(create_embed!(
            description: "You are already assigned or queued for that bounty.",
            color: FAILURE,
        ))
        .await?;
        return Ok(ControlFlow::Break(()));
    }
    Ok(ControlFlow::Continue(()))
}
