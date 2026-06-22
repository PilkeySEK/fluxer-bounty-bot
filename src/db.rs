use std::sync::Arc;

use anyhow::Context;
use fluxer_neptunium::model::id::{Id, marker::GuildMarker};
use moka::sync::Cache;
use sqlx::PgPool;

use crate::db::guilds::GuildConfig;

pub mod bounties;
pub mod guilds;

#[derive(Clone)]
pub struct DbManager {
    pool: PgPool,
    cached_guild_config: Arc<moka::sync::Cache<Id<GuildMarker>, Arc<GuildConfig>>>,
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
        })
    }
}
