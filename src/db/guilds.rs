use std::sync::Arc;

use anyhow::Context;
use enum_map::{Enum, EnumMap, enum_map};
use enumset::{EnumSet, EnumSetType};
use fluxer_neptunium::model::id::{
    Id,
    marker::{ChannelMarker, GuildMarker},
};
use serde::{Deserialize, Serialize};

use crate::db::DbManager;

impl DbManager {
    pub async fn get_guild_config_upsert(
        &self,
        guild_id: Id<GuildMarker>,
    ) -> anyhow::Result<Arc<GuildConfig>> {
        if let Some(config) = self.cached_guild_config.get(&guild_id) {
            return Ok(config);
        }
        let raw = sqlx::query_as!(
            GuildConfigSchema,
            "INSERT INTO guilds (guild_id, bounty_submission_format)
            VALUES ($1, $2)
            ON CONFLICT (guild_id) DO UPDATE
            SET guild_id = EXCLUDED.guild_id
            RETURNING *",
            guild_id.into_inner().cast_signed(),
            serde_json::to_value(&BountySubmissionFormat::default())?,
        )
        .fetch_one(&self.pool)
        .await?;
        let guild_config = Arc::new(raw.try_into().context("Failed to convert")?);
        self.cached_guild_config
            .insert(guild_id, Arc::clone(&guild_config));
        Ok(guild_config)
    }

    pub async fn set_bounty_submission_channel(
        &self,
        guild_id: Id<GuildMarker>,
        channel_id: Option<Id<ChannelMarker>>,
    ) -> anyhow::Result<()> {
        sqlx::query!(
            "UPDATE guilds
            SET bounty_submission_channel = $1
            WHERE guild_id = $2",
            channel_id.map(|id| id.into_inner().cast_signed()),
            guild_id.into_inner().cast_signed(),
        )
        .execute(&self.pool)
        .await?;
        self.cached_guild_config.invalidate(&guild_id);
        Ok(())
    }
    pub async fn set_approval_queue_channel(
        &self,
        guild_id: Id<GuildMarker>,
        channel_id: Option<Id<ChannelMarker>>,
    ) -> anyhow::Result<()> {
        sqlx::query!(
            "UPDATE guilds
            SET approval_queue_channel = $1
            WHERE guild_id = $2",
            channel_id.map(|id| id.into_inner().cast_signed()),
            guild_id.into_inner().cast_signed(),
        )
        .execute(&self.pool)
        .await?;
        self.cached_guild_config.invalidate(&guild_id);
        Ok(())
    }
    pub async fn set_claimed_bounties_channel(
        &self,
        guild_id: Id<GuildMarker>,
        channel_id: Option<Id<ChannelMarker>>,
    ) -> anyhow::Result<()> {
        sqlx::query!(
            "UPDATE guilds
            SET claimed_bounties_channel = $1
            WHERE guild_id = $2",
            channel_id.map(|id| id.into_inner().cast_signed()),
            guild_id.into_inner().cast_signed(),
        )
        .execute(&self.pool)
        .await?;
        self.cached_guild_config.invalidate(&guild_id);
        Ok(())
    }
    pub async fn set_completed_bounties_channel(
        &self,
        guild_id: Id<GuildMarker>,
        channel_id: Option<Id<ChannelMarker>>,
    ) -> anyhow::Result<()> {
        sqlx::query!(
            "UPDATE guilds
            SET completed_bounties_channel = $1
            WHERE guild_id = $2",
            channel_id.map(|id| id.into_inner().cast_signed()),
            guild_id.into_inner().cast_signed(),
        )
        .execute(&self.pool)
        .await?;
        self.cached_guild_config.invalidate(&guild_id);
        Ok(())
    }
    pub async fn set_denied_bounties_channel(
        &self,
        guild_id: Id<GuildMarker>,
        channel_id: Option<Id<ChannelMarker>>,
    ) -> anyhow::Result<()> {
        sqlx::query!(
            "UPDATE guilds
            SET denied_bounties_channel = $1
            WHERE guild_id = $2",
            channel_id.map(|id| id.into_inner().cast_signed()),
            guild_id.into_inner().cast_signed(),
        )
        .execute(&self.pool)
        .await?;
        self.cached_guild_config.invalidate(&guild_id);
        Ok(())
    }
}

#[derive(Enum, Serialize, Deserialize, EnumSetType)]
pub enum BountyInfoKey {
    Title,
    Deadline,
    IssueUrl,
    AdditionalInfo,
    JudgingCriteria,
    BountyAmount,
}

#[derive(Serialize, Deserialize)]
pub struct BountySubmissionFormat {
    pub titles: EnumMap<BountyInfoKey, Vec<String>>,
    pub required: EnumSet<BountyInfoKey>,
}

impl Default for BountySubmissionFormat {
    fn default() -> Self {
        let titles = enum_map! {
            BountyInfoKey::Title => vec!["Title".to_owned(), "Bounty Title".to_owned()],
            BountyInfoKey::Deadline => vec!["Due Date".to_owned(), "Due Date (Timeline)".to_owned(), "Deadline".to_owned()],
            BountyInfoKey::IssueUrl => vec!["Issue URL (Optional)".to_owned(), "Issue URL".to_owned()],
            BountyInfoKey::AdditionalInfo => vec!["Additional Information".to_owned(), "Additional Info".to_owned()],
            BountyInfoKey::JudgingCriteria => vec!["Judging Criteria".to_owned()],
            BountyInfoKey::BountyAmount => vec!["Bounty Amount".to_owned(), "Amount".to_owned()],
        };
        let required = BountyInfoKey::Title
            | BountyInfoKey::AdditionalInfo
            | BountyInfoKey::JudgingCriteria
            | BountyInfoKey::BountyAmount;

        Self { titles, required }
    }
}

pub struct GuildConfig {
    pub guild_id: Id<GuildMarker>,
    pub bounty_submission_channel: Option<Id<ChannelMarker>>,
    pub approval_queue_channel: Option<Id<ChannelMarker>>,
    pub claimed_bounties_channel: Option<Id<ChannelMarker>>,
    pub completed_bounties_channel: Option<Id<ChannelMarker>>,
    pub denied_bounties_channel: Option<Id<ChannelMarker>>,
    pub command_prefixes: Vec<String>,
    pub bounty_submission_format: BountySubmissionFormat,
    pub command_channels: Option<Vec<Id<ChannelMarker>>>,
    pub current_bounty_number: i64,
}

impl TryFrom<GuildConfigSchema> for GuildConfig {
    type Error = anyhow::Error;
    fn try_from(value: GuildConfigSchema) -> Result<Self, Self::Error> {
        Ok(Self {
            guild_id: value.guild_id.cast_unsigned().into(),
            bounty_submission_channel: value
                .bounty_submission_channel
                .map(|id| id.cast_unsigned().into()),
            approval_queue_channel: value
                .approval_queue_channel
                .map(|id| id.cast_unsigned().into()),
            claimed_bounties_channel: value
                .claimed_bounties_channel
                .map(|id| id.cast_unsigned().into()),
            completed_bounties_channel: value
                .completed_bounties_channel
                .map(|id| id.cast_unsigned().into()),
            denied_bounties_channel: value
                .denied_bounties_channel
                .map(|id| id.cast_unsigned().into()),
            command_prefixes: value.command_prefixes,
            bounty_submission_format: serde_json::from_value(value.bounty_submission_format)?,
            command_channels: value.command_channels.map(|command_channels| {
                command_channels
                    .into_iter()
                    .map(|id| id.cast_unsigned().into())
                    .collect()
            }),
            current_bounty_number: value.current_bounty_number,
        })
    }
}

#[derive(sqlx::FromRow)]
pub(super) struct GuildConfigSchema {
    pub guild_id: i64,
    pub bounty_submission_channel: Option<i64>,
    pub approval_queue_channel: Option<i64>,
    pub claimed_bounties_channel: Option<i64>,
    pub completed_bounties_channel: Option<i64>,
    pub denied_bounties_channel: Option<i64>,
    pub command_prefixes: Vec<String>,
    pub bounty_submission_format: serde_json::Value,
    pub command_channels: Option<Vec<i64>>,
    pub current_bounty_number: i64,
}
