use std::sync::Arc;

use anyhow::Context;
use fluxer_neptunium::model::id::{Id, marker::GuildMarker};
use moka::sync::Cache;
use sqlx::PgPool;

use crate::db::{guild_permissions::GuildPermissionsEntry, guilds::GuildConfig};

pub mod bounties;
pub mod bounty_assignee_queue;
pub mod bounty_stakeholders;
pub mod guild_permissions;
pub mod guilds;

#[derive(Clone)]
pub struct DbManager {
    pool: PgPool,
    cached_guild_config: Arc<moka::sync::Cache<Id<GuildMarker>, Arc<GuildConfig>>>,
    cached_guild_permissions:
        Arc<moka::sync::Cache<Id<GuildMarker>, Arc<Vec<GuildPermissionsEntry>>>>,
}

impl DbManager {
    pub async fn new(url: &str) -> anyhow::Result<Self> {
        let pool = PgPool::connect(url).await.context("Failed to connect")?;
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .context("Failed to run migrations")?;
        Ok(Self {
            pool,
            cached_guild_config: Arc::new(Cache::new(128)),
            cached_guild_permissions: Arc::new(Cache::new(32)),
        })
    }
}
