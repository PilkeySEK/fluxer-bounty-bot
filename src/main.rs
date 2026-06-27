use std::{collections::HashSet, sync::LazyLock};

use anyhow::Context;
use fluxer_neptunium::{
    client::{Client, ClientConfig},
    http::endpoints::channel::AllowedMentions,
};
use sqids::{Sqids, SqidsBuilder};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use crate::{db::DbManager, event_handler::Handler};

mod colors;
mod commands;
mod db;
mod event_handler;
mod util;

const AVATAR_URL_BASE: &str = "https://fluxerusercontent.com/avatars";
const STATIC_BASE: &str = "https://fluxerstatic.com";
const SQIDS_MIN_LENGTH: u8 = 5;
static SQIDS: LazyLock<Sqids> = LazyLock::new(|| {
    #[expect(
        clippy::unwrap_used,
        reason = "There is a test for the initialization being successful."
    )]
    SqidsBuilder::new()
        .min_length(SQIDS_MIN_LENGTH)
        .build()
        .unwrap()
});
static SQIDS_NO_BLOCKLIST: LazyLock<Sqids> = LazyLock::new(|| {
    #[expect(
        clippy::unwrap_used,
        reason = "There is a test for the initialization being successful."
    )]
    SqidsBuilder::new()
        .blocklist(HashSet::new())
        .min_length(SQIDS_MIN_LENGTH)
        .build()
        .unwrap()
});

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::DEBUG.into())
                .from_env_lossy(),
        )
        .init();

    if let Err(e) = dotenvy::dotenv() {
        tracing::warn!(".env file not found: {e}");
    }

    let database_url =
        dotenvy::var("DATABASE_URL").context("Failed to load DATABASE_URL from env")?;
    let token = dotenvy::var("TOKEN").context("Failed to load TOKEN from env")?;
    let client_id = serde_json::from_str(
        &dotenvy::var("CLIENT_ID").context("Failed to load CLIENT_ID from env")?,
    )
    .context("Failed to parse CLIENT_ID")?;
    let bounty_workflow_image_url = dotenvy::var("BOUNTY_WORKFLOW_IMAGE_URL")
        .context("Failed to load BOUNTY_WORKFLOW_IMAGE_URL from env")?;

    let db = DbManager::new(&database_url)
        .await
        .context("Failed to create database manager")?;

    let mut client = Client::new_with_config(
        token,
        ClientConfig::builder()
            .default_allowed_mentions(AllowedMentions {
                parse: Some(Vec::new()),
                users: Some(Vec::new()),
                roles: Some(Vec::new()),
                replied_user: false,
            })
            .build(),
    );
    client.register_event_handler(Handler::new(db, client_id, bounty_workflow_image_url));

    client.start().await.context("Fatal client error")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sqids_initialization() {
        let id_to_encode = 123;
        let encoded_id = SQIDS.encode(&[id_to_encode]);
        let encoded_id_no_blocklist = SQIDS.encode(&[id_to_encode]);
        assert!(encoded_id.is_ok());
        assert!(encoded_id_no_blocklist.is_ok());
    }
}
