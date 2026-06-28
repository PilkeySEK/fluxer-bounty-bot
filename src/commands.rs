use std::{collections::HashMap, pin::Pin, sync::Arc, time::Duration};

use anyhow::Context as _;
use chrono::{TimeDelta, Utc};
use fluxer_neptunium::{
    cache::CachedGuildMember,
    cached_payload::CachedMessageCreate,
    create_embed,
    events::context::Context,
    exts::{GuildMemberExt, MessageExt},
    http::endpoints::channel::CreateMessageBody,
    model::{
        guild::permissions::Permissions,
        id::{
            Id,
            marker::{GuildMarker, MessageMarker},
        },
    },
};
use rand::distr::{Alphabetic, SampleString};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    colors::FAILURE,
    db::{
        DbManager,
        guild_permissions::{BotPermissions, GuildPermissionEntity},
        guilds::GuildConfig,
    },
    event_handler::reactions::{
        ReactionExpiryHandlerFn, ReactionHandler, ReactionsEventHandlerMessage,
    },
};

pub mod bounty_management;
pub mod bounty_stakeholders;
pub mod guild_config;
pub mod misc;
pub mod permission_management;

pub struct CommandContext<'a> {
    pub ctx: &'a Context,
    pub db: &'a DbManager,
    pub message: &'a CachedMessageCreate,
    pub guild_member: &'a CachedGuildMember,
    pub guild_id: Id<GuildMarker>,
    pub reaction_handler_tx: &'a UnboundedSender<ReactionsEventHandlerMessage>,
    pub bounty_workflow_image_url: &'a str,
    pub guild_config: &'a GuildConfig,
}

impl CommandContext<'_> {
    /// Helper for registering a reaction handler. For more control, use `reaction_handler_tx` on this struct.
    ///
    /// If an error occurs, it is logged but no panics will happen.
    pub fn register_reaction_handler(
        &self,
        message_id: Id<MessageMarker>,
        handler: impl ReactionHandler + 'static,
        expiry: Option<(ReactionExpiryHandlerFn, std::time::Duration)>,
    ) {
        let expiry = match expiry {
            Some((f, expires_in)) => {
                let expires_in: TimeDelta = match TimeDelta::from_std(expires_in) {
                    Ok(expires_in) => expires_in,
                    Err(e) => {
                        tracing::error!("Failed to convert Duration to TimeDelta: {e}");
                        return;
                    }
                };
                let now = Utc::now();
                let Some(expires_at) = now.checked_add_signed(expires_in) else {
                    tracing::error!("Overflow calculating expires_at.");
                    return;
                };
                Some((f, expires_at))
            }
            None => None,
        };
        if self
            .reaction_handler_tx
            .send((message_id, Box::new(handler), expiry))
            .is_err()
        {
            tracing::error!("The reaction handler is gone.");
        }
    }

    /// Does not take Fluxer permissions into account in any way.
    pub async fn my_permissions(&self) -> anyhow::Result<BotPermissions> {
        let guild_permissions = self.db.list_guild_permissions(self.guild_id).await?;
        let my_roles = &self.guild_member.roles;
        let mut my_permissions = BotPermissions::empty();
        for permission in &*guild_permissions {
            match permission.entity {
                GuildPermissionEntity::User(id) => {
                    if id == self.guild_member.id {
                        my_permissions |= permission.allow;
                    }
                }
                GuildPermissionEntity::Role(id) => {
                    if id == self.guild_id.cast() || my_roles.contains(&id) {
                        my_permissions |= permission.allow;
                    }
                }
            }
        }
        Ok(my_permissions)
    }

    /// If the user has Fluxer Administrator permissions, will also return true.
    pub async fn has_permissions(&self, permissions: BotPermissions) -> anyhow::Result<bool> {
        if self
            .guild_member
            .has_permissions(self.ctx, Permissions::ADMINISTRATOR)
            .await?
        {
            return Ok(true);
        }
        Ok(self.my_permissions().await?.contains(permissions))
    }

    /// Reply to the original message, deleting the message and reply after 5 seconds if the guild config has this configured.
    pub async fn reply(&self, body: impl Into<CreateMessageBody> + Send) -> anyhow::Result<()> {
        self.reply_with_deletion_duration(body, Duration::from_secs(5))
            .await
    }

    pub async fn reply_with_deletion_duration(
        &self,
        body: impl Into<CreateMessageBody> + Send,
        duration: Duration,
    ) -> anyhow::Result<()> {
        let reply = self.message.reply(self.ctx, body).await?;
        if self.guild_config.delete_commands {
            let message = self.message.load();
            let ctx = self.ctx.clone();
            tokio::spawn(async move {
                tokio::time::sleep(duration).await;
                if let Err(e) = message.delete(&ctx).await {
                    tracing::error!("Error deleting message {}: {e}", message.id);
                }
                if let Err(e) = reply.delete(&ctx).await {
                    tracing::error!("Error deleting reply {}: {e}", reply.id);
                }
            });
        }
        Ok(())
    }
}

pub trait CommandExecuteFn<'a>: Send + Sync + 'static {
    fn call(
        &self,
        ctx: CommandContext<'a>,
        args: &'a str,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'a>>;
}

impl<'a, F, Fut> CommandExecuteFn<'a> for F
where
    F: Fn(CommandContext<'a>, &'a str) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = anyhow::Result<()>> + Send + 'a,
{
    fn call(
        &self,
        ctx: CommandContext<'a>,
        args: &'a str,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'a>> {
        Box::pin(self(ctx, args))
    }
}

pub struct CommandDispatcher {
    #[expect(clippy::type_complexity)]
    commands: HashMap<&'static str, (BotPermissions, Arc<dyn for<'a> CommandExecuteFn<'a>>)>,
}

impl CommandDispatcher {
    #[expect(clippy::type_complexity)]
    pub fn new<const N: usize>(
        commands: [(
            &[&'static str],
            BotPermissions,
            Arc<dyn for<'a> CommandExecuteFn<'a>>,
        ); N],
    ) -> Self {
        let mut commands_map = HashMap::new();
        for (names, permissions, f) in commands {
            for &name in names {
                if commands_map
                    .insert(name, (permissions, Arc::clone(&f)))
                    .is_some()
                {
                    tracing::warn!("Duplicate command name: {name}");
                }
            }
        }
        Self {
            commands: commands_map,
        }
    }

    /// The prefix must already be stripped.
    pub async fn execute(&self, command: &str, ctx: CommandContext<'_>) -> anyhow::Result<()> {
        let (command, args) = command.trim().split_once(' ').unwrap_or((command, ""));
        let Some((required_permissions, f)) = self.commands.get(command) else {
            return Ok(());
        };
        if !ctx.has_permissions(*required_permissions).await? {
            ctx.reply(create_embed!(
                description: "You do not have the required permissions to perform this command.",
                color: FAILURE,
            ))
            .await?;
            return Ok(());
        }

        let message_backup = ctx.message;
        let ctx_ctx_backup = ctx.ctx;
        if let Err(e) = f.call(ctx, args).await {
            let random_error_id = Alphabetic.sample_string(&mut rand::rng(), 16);
            tracing::error!(
                "[{random_error_id}] Error executing command `{command}` with args `{args}`: {e}"
            );
            message_backup.reply(ctx_ctx_backup, create_embed!(
                description: format!("There was an error executing the command. `{random_error_id}`"),
                color: FAILURE,
            )).await.context("While replying with error code")?;
        }

        Ok(())
    }
}

#[expect(clippy::too_many_lines)]
pub fn new_dispatcher_with_commands() -> CommandDispatcher {
    CommandDispatcher::new([
        (&["ping"], BotPermissions::empty(), Arc::new(misc::ping)),
        (
            &["complete", "complete-bounty", "bounty-complete"],
            BotPermissions::MANAGE_BOUNTIES,
            Arc::new(bounty_management::complete_bounty),
        ),
        (
            &["approve", "approve-bounty", "bounty-approve"],
            BotPermissions::MANAGE_BOUNTIES,
            Arc::new(bounty_management::approve_bounty),
        ),
        (
            &[
                "reject",
                "deny",
                "reject-bounty",
                "bounty-reject",
                "deny-bounty",
                "bounty-deny",
            ],
            BotPermissions::MANAGE_BOUNTIES,
            Arc::new(bounty_management::reject_bounty),
        ),
        (
            &[
                "delete",
                "delete-bounty",
                "deletebounty",
                "removebounty",
                "bounty-delete",
                "bountydelete",
                "bountydel",
                "bountyrm",
                "rmbounty",
                "delbounty",
            ],
            BotPermissions::MANAGE_BOUNTIES,
            Arc::new(bounty_management::delete_bounty),
        ),
        (
            &["assign", "assign-to", "assign-to-bounty", "bounty-assign"],
            BotPermissions::MANAGE_BOUNTIES,
            Arc::new(bounty_management::assign_to_bounty),
        ),
        (
            &["self-assign", "selfassign"],
            BotPermissions::BOUNTY_HUNTER,
            Arc::new(bounty_management::self_assign_to_bounty),
        ),
        (
            &[
                "config",
                "communityconfig",
                "community-config",
                "guildconfig",
                "guild-config",
                "serverconfig",
                "server-config",
                "cfg",
            ],
            BotPermissions::MANAGE_GUILD_CONFIG,
            Arc::new(guild_config::guild_config),
        ),
        (
            &[
                "bounty-workflow",
                "bountyworkflow",
                "workflow",
                "bounty-workflow-image",
            ],
            BotPermissions::empty(),
            Arc::new(misc::bounty_workflow),
        ),
        (
            &["stakeholder"],
            BotPermissions::MANAGE_BOUNTIES,
            Arc::new(bounty_stakeholders::bounty_stakeholder),
        ),
        (
            &["self-unassign", "selfunassign"],
            BotPermissions::BOUNTY_HUNTER,
            Arc::new(bounty_management::self_unassign_from_bounty),
        ),
        (
            &[
                "unassign",
                "unassign-from",
                "unassign-from-bounty",
                "bounty-unassign",
            ],
            BotPermissions::MANAGE_BOUNTIES,
            Arc::new(bounty_management::unassign_from_bounty),
        ),
        (
            &["edit", "edit-bounty"],
            BotPermissions::MANAGE_BOUNTIES,
            Arc::new(bounty_management::edit_bounty),
        ),
        (
            &["add-permission"],
            BotPermissions::MANAGE_GUILD_CONFIG,
            Arc::new(permission_management::add_permission_to),
        ),
        (
            &["remove-permission", "rm-permission"],
            BotPermissions::MANAGE_GUILD_CONFIG,
            Arc::new(permission_management::remove_permission_from),
        ),
        (
            &["permissions", "perms", "list-perms", "list-permissions"],
            BotPermissions::MANAGE_GUILD_CONFIG,
            Arc::new(permission_management::list_permissions),
        ),
        (&["help"], BotPermissions::empty(), Arc::new(misc::help)),
    ])
}
