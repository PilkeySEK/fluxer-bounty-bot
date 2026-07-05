use fluxer_neptunium::create_embed;

use crate::{
    colors::{FAILURE, SUCCESS},
    commands::CommandContext,
    db::bounty_stakeholders::BountyStakeholder,
    util::{
        confirmation::MaybeExpired, get_bounty_num_from_args, update_bounty_message,
        user_arg::parse_user_arg,
    },
};

pub async fn bounty_stakeholder(ctx: CommandContext<'_>, args: &str) -> anyhow::Result<()> {
    let args = args.trim();
    let (subcommand, args) = args.split_once(' ').unwrap_or((args, ""));
    match subcommand {
        "add" => add_bounty_stakeholder(ctx, args).await,
        "remove" | "rm" => remove_bounty_stakeholder(ctx, args).await,
        _ => {
            ctx.reply(create_embed!(
                description: "Unknown subcommand.",
                color: FAILURE,
            ))
            .await?;
            Ok(())
        }
    }
}

async fn add_bounty_stakeholder(ctx: CommandContext<'_>, args: &str) -> anyhow::Result<()> {
    let (bounty_num, rest) = get_bounty_num_from_args!(ctx, args, "add a stakeholder to");
    let Some((amount, rest)) = parse_amount(rest) else {
        ctx.reply(create_embed!(
            description: "Could not parse the amount.\nSyntax: `b!stakeholder add <bounty id> <amount> <user> [note]`\nExample: `b!stakeholder add UkLWZ $10 someone`",
            color: FAILURE,
        )).await?;
        return Ok(());
    };
    let (user_arg, note) = rest.split_once(' ').unwrap_or((rest, ""));
    let MaybeExpired::NotExpired(user_id) = parse_user_arg(&ctx, user_arg).await? else {
        return Ok(());
    };
    let Some(user_id) = user_id else {
        ctx.reply(create_embed!(
            description: "Could not find a user matching your query.",
            color: FAILURE,
        ))
        .await?;
        return Ok(());
    };
    let note = note.trim();
    let note = if note.is_empty() { None } else { Some(note) };

    let Some(bounty) = ctx.db.get_bounty(ctx.guild_id, bounty_num).await? else {
        ctx.reply(create_embed!(
            description: "A bounty with that ID doesn't exist.",
            color: FAILURE,
        ))
        .await?;
        return Ok(());
    };
    #[expect(clippy::cast_possible_truncation)]
    ctx.db
        .add_bounty_stakeholder(BountyStakeholder {
            bounty_id: bounty.bounty_id,
            user_id,
            amount: (amount * 100.0) as i32,
            note: note.map(str::to_owned),
        })
        .await?;

    let bounty_number = bounty.bounty_number;

    update_bounty_message(ctx.ctx, ctx.db, ctx.guild_config, bounty);

    ctx.reply(create_embed!(
        description: format!("Added bounty stakeholder <@{user_id}> on bounty `{bounty_number}` with `${amount:.2}`."),
        color: SUCCESS,
    )).await?;

    Ok(())
}

async fn remove_bounty_stakeholder(ctx: CommandContext<'_>, args: &str) -> anyhow::Result<()> {
    let (bounty_num, user_arg) = get_bounty_num_from_args!(ctx, args, "remove a stakeholder from");
    let MaybeExpired::NotExpired(user_id) = parse_user_arg(&ctx, user_arg.trim()).await? else {
        return Ok(());
    };
    let Some(user_id) = user_id else {
        ctx.reply(create_embed!(
            description: "Could not find a user matching your query.",
            color: FAILURE,
        ))
        .await?;
        return Ok(());
    };

    let Some(bounty) = ctx.db.get_bounty(ctx.guild_id, bounty_num).await? else {
        ctx.reply(create_embed!(
            description: "A bounty with that ID doesn't exist.",
            color: FAILURE,
        ))
        .await?;
        return Ok(());
    };
    ctx.db
        .remove_bounty_stakeholder(bounty.bounty_id, user_id)
        .await?;

    let bounty_number = bounty.bounty_number;

    update_bounty_message(ctx.ctx, ctx.db, ctx.guild_config, bounty);

    ctx.reply(create_embed!(
        description: format!("Removed bounty stakeholder <@{user_id}> from bounty `{bounty_number}`."),
        color: SUCCESS,
    )).await?;

    Ok(())
}

fn parse_amount(args: &str) -> Option<(f64, &str)> {
    enum CurrencyType {
        Dollars,
        Cents,
    }
    let (amount, rest) = args.split_once(' ').unwrap_or((args, ""));
    let (amount, currency_type) = if let Some(amount) = amount.strip_prefix('$') {
        (amount, CurrencyType::Dollars)
    } else if let Some(amount) = amount.strip_suffix("ct") {
        (amount, CurrencyType::Cents)
    } else {
        (amount, CurrencyType::Dollars)
    };
    let amount = amount.parse::<f64>().ok()?;
    Some((
        match currency_type {
            CurrencyType::Dollars => amount,
            CurrencyType::Cents => amount / 100.0,
        },
        rest,
    ))
}
