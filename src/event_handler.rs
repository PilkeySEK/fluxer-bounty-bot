use std::sync::Arc;

use anyhow::Context as _;
use fluxer_neptunium::{
    cached_payload::{CachedMessageCreate, CachedMessageReactionAdd, CachedReady},
    model::{
        guild::permissions::Permissions,
        id::{Id, marker::UserMarker},
    },
    prelude::*,
};

use crate::{
    commands::{self, CommandContext},
    db::DbManager,
    event_handler::{reactions::ReactionsEventHandler, submission::handle_submission_create},
};

pub mod reactions;
mod submission;

pub struct Handler {
    db: DbManager,
    client_id: Id<UserMarker>,
    #[expect(clippy::struct_field_names)]
    reactions_event_handler: ReactionsEventHandler,
}

impl Handler {
    pub fn new(db: DbManager, client_id: Id<UserMarker>) -> Self {
        Self {
            db,
            client_id,
            reactions_event_handler: ReactionsEventHandler::new(),
        }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn on_ready(&self, _ctx: Context, ready: Arc<CachedReady>) -> Result<(), EventError> {
        tracing::info!(
            "Logged in as {}#{}",
            ready.user.username,
            ready.user.discriminator
        );
        Ok(())
    }

    async fn on_message_create(
        &self,
        ctx: Context,
        message: Arc<CachedMessageCreate>,
    ) -> Result<(), EventError> {
        if message.author.bot {
            return Ok(());
        }
        let Some(guild_id) = message.channel_id.get(&ctx).await?.guild_id else {
            return Ok(());
        };
        let author_guild_member = guild_id.get_member(&ctx, message.author.id).await?.load();

        let guild_config = match self
            .db
            .get_guild_config_upsert(guild_id)
            .await
            .with_context(|| format!("Failed to get guild config for {guild_id}"))
        {
            Ok(guild_config) => guild_config,
            Err(e) => {
                tracing::error!("{e}");
                return Ok(());
            }
        };

        if let Some(command_channels) = &guild_config.command_channels
            && !command_channels.contains(&message.channel_id)
            && !author_guild_member
                .has_permissions(&ctx, Permissions::MANAGE_GUILD)
                .await?
        {
            return Ok(());
        }

        let content = message.content.trim();

        let mention_prefix = format!("<@{}>", self.client_id);

        let full_command = if let Some(full_command) = content.strip_prefix(&mention_prefix) {
            full_command
        } else {
            let Some(full_command) = guild_config
                .command_prefixes
                .iter()
                .find_map(|prefix| content.strip_prefix(prefix))
            else {
                if let Some(bounty_submission_channel) = guild_config.bounty_submission_channel
                    && message.channel_id == bounty_submission_channel
                {
                    if let Err(e) = handle_submission_create(
                        &ctx,
                        &message,
                        &author_guild_member.user.load(),
                        &guild_config,
                        &self.db,
                        guild_id,
                    )
                    .await
                    {
                        tracing::error!("Error handling submission: {e}");
                    }
                }
                return Ok(());
            };
            full_command
        };

        let (command, args) = full_command.split_once(' ').unwrap_or((full_command, ""));

        let command_context = CommandContext {
            ctx: &ctx,
            db: &self.db,
            message: &message,
            guild_member: &author_guild_member,
            guild_id,
            reaction_handler_tx: &self.reactions_event_handler.tx,
        };

        if let Err(e) = match command {
            "ping" => commands::misc::ping(command_context).await,
            "bounty" => commands::bounty_management::bounty_management(command_context, args).await,
            "config" | "communityconfig" | "community-config" | "guildconfig" | "guild-config"
            | "serverconfig" | "server-config" | "cfg" => {
                commands::guild_config::guild_config(command_context, args).await
            }
            _ => Ok(()),
        } {
            tracing::error!("Error executing command `{command}`: {e}");
        }

        Ok(())
    }

    async fn on_message_reaction_add(
        &self,
        _ctx: Context,
        event: Arc<CachedMessageReactionAdd>,
    ) -> Result<(), EventError> {
        self.reactions_event_handler
            .handle_reaction_add(event)
            .await
    }
}
