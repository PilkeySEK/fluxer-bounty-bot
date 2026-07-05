use std::{collections::HashMap, str::FromStr};

use anyhow::Context;
use chrono::{DateTime, Utc};
use fluxer_neptunium::model::id::{
    Id,
    marker::{ChannelMarker, GuildMarker, MessageMarker, UserMarker},
};

use crate::db::{DbManager, guilds::BountyInfoKey};

impl DbManager {
    /// Returns `Ok(None)` if the bounty with that number does not exist in the specified guild.
    pub async fn delete_and_return_bounty(
        &self,
        guild_id: Id<GuildMarker>,
        bounty_number: BountyNum,
    ) -> anyhow::Result<Option<Bounty>> {
        let raw = sqlx::query_as!(
            BountySchema,
            "DELETE FROM bounties
            WHERE guild_id = $1 AND bounty_number = $2
            RETURNING *",
            guild_id.into_inner().cast_signed(),
            bounty_number.0
        )
        .fetch_optional(&self.pool)
        .await?;
        let Some(raw) = raw else {
            return Ok(None);
        };
        Ok(Some(raw.try_into().context("Failed to convert")?))
    }

    /// Returns `Ok(None)` if the bounty with that number does not exist in the specified guild.
    pub async fn get_bounty(
        &self,
        guild_id: Id<GuildMarker>,
        bounty_number: BountyNum,
    ) -> anyhow::Result<Option<Bounty>> {
        let raw = sqlx::query_as!(
            BountySchema,
            "SELECT * FROM bounties
            WHERE guild_id = $1 AND bounty_number = $2",
            guild_id.into_inner().cast_signed(),
            bounty_number.0,
        )
        .fetch_optional(&self.pool)
        .await?;
        let Some(raw) = raw else {
            return Ok(None);
        };
        Ok(Some(raw.try_into().context("Failed to convert")?))
    }

    /// Won't take the previous state of the bounty into account.
    pub async fn set_bounty_state_and_related_message(
        &self,
        guild_id: Id<GuildMarker>,
        bounty_number: BountyNum,
        state: BountyState,
        related_message: Option<BountyRelatedMessage>,
    ) -> anyhow::Result<()> {
        sqlx::query!(
            "UPDATE bounties
            SET state = $1, related_message_id = $2, related_channel_id = $3
            WHERE guild_id = $4 AND bounty_number = $5",
            state.to_string(),
            related_message.map(|related| related.message_id.into_inner().cast_signed()),
            related_message.map(|related| related.channel_id.into_inner().cast_signed()),
            guild_id.into_inner().cast_signed(),
            bounty_number.0,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn create_bounty(&self, bounty: BountyCreateData) -> anyhow::Result<()> {
        sqlx::query!(
            "INSERT INTO bounties (bounty_number, guild_id, created_by, content, state, created_at, related_message_id, related_channel_id, deadline)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
            bounty.bounty_number.0,
            bounty.guild_id.into_inner().cast_signed(),
            bounty.created_by.into_inner().cast_signed(),
            serde_json::to_value(&bounty.content)?,
            bounty.state.to_string(),
            bounty.created_at,
            bounty.related_message.map(|v| v.message_id.into_inner().cast_signed()),
            bounty.related_message.map(|v| v.channel_id.into_inner().cast_signed()),
            bounty.deadline,
        ).execute(&self.pool).await?;
        Ok(())
    }
    /*
        pub async fn assign_user_to_bounty(
            &self,
            guild_id: Id<GuildMarker>,
            bounty_number: BountyNum,
            user_id: Option<Id<UserMarker>>,
        ) -> anyhow::Result<PgQueryResult> {
            Ok(sqlx::query!(
                "UPDATE bounties
            SET assigned_to = $1
            WHERE guild_id = $2 AND bounty_number = $3",
                user_id.map(|id| id.into_inner().cast_signed()),
                guild_id.into_inner().cast_signed(),
                bounty_number.0,
            )
            .execute(&self.pool)
            .await?)
        }
    */
    pub async fn set_bounty_content(
        &self,
        bounty_id: i64,
        content: &BountySubmissionContent,
    ) -> anyhow::Result<()> {
        sqlx::query!(
            "UPDATE bounties
            SET content = $1
            WHERE bounty_id = $2",
            serde_json::to_value(content)?,
            bounty_id,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn set_bounty_deadine(
        &self,
        bounty_id: i64,
        deadline: Option<DateTime<Utc>>,
    ) -> anyhow::Result<()> {
        sqlx::query!(
            "UPDATE bounties
            SET deadline = $1
            WHERE bounty_id = $2",
            deadline,
            bounty_id,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

#[derive(strum::Display, strum::EnumString, PartialEq, Eq, Copy, Clone)]
pub enum BountyState {
    /// The bounty has been implemented (implies Approved).
    Completed,
    /// The bounty has been approved but is not implemented yet.
    Approved,
    /// The bounty is pending approval.
    Pending,
    /// The bounty has been rejected.
    Rejected,
}

pub type BountySubmissionContent = HashMap<BountyInfoKey, String>;

#[derive(Copy, Clone)]
pub struct BountyRelatedMessage {
    pub channel_id: Id<ChannelMarker>,
    pub message_id: Id<MessageMarker>,
}

#[expect(clippy::struct_field_names)]
pub struct Bounty {
    pub bounty_id: i64,
    pub bounty_number: BountyNum,
    #[expect(unused)]
    pub guild_id: Id<GuildMarker>,
    pub created_by: Id<UserMarker>,
    pub content: BountySubmissionContent,
    pub state: BountyState,
    pub created_at: DateTime<Utc>,
    pub related_message: Option<BountyRelatedMessage>,
    pub deadline: Option<DateTime<Utc>>,
}

pub struct BountyCreateData {
    pub bounty_number: BountyNum,
    pub guild_id: Id<GuildMarker>,
    pub created_by: Id<UserMarker>,
    pub content: BountySubmissionContent,
    pub state: BountyState,
    pub created_at: DateTime<Utc>,
    pub related_message: Option<BountyRelatedMessage>,
    pub deadline: Option<DateTime<Utc>>,
}

impl TryFrom<BountySchema> for Bounty {
    type Error = anyhow::Error;
    fn try_from(value: BountySchema) -> Result<Self, Self::Error> {
        Ok(Self {
            bounty_id: value.bounty_id,
            bounty_number: BountyNum(value.bounty_number),
            guild_id: value.guild_id.cast_unsigned().into(),
            created_by: value.created_by.cast_unsigned().into(),
            content: serde_json::from_value(value.content)?,
            state: BountyState::from_str(&value.state)?,
            created_at: value.created_at,
            related_message: if let Some(related_message_id) = value.related_message_id
                && let Some(related_channel_id) = value.related_channel_id
            {
                Some(BountyRelatedMessage {
                    channel_id: related_channel_id.cast_unsigned().into(),
                    message_id: related_message_id.cast_unsigned().into(),
                })
            } else {
                None
            },
            deadline: value.deadline,
        })
    }
}

struct BountySchema {
    bounty_id: i64,
    bounty_number: i64,
    guild_id: i64,
    created_by: i64,
    content: serde_json::Value,
    state: String,
    created_at: DateTime<Utc>,
    related_message_id: Option<i64>,
    related_channel_id: Option<i64>,
    deadline: Option<DateTime<Utc>>,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct BountyNum(pub i64);

impl std::fmt::Display for BountyNum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Try to encode it using SQIDS with the blocklist, which is very unlikely to fail.
        // It only fails when it has reached the maximum number of tries for getting around the
        // blocklist, in which case we use sqids without a blocklist, which may contain a bad
        // word but this is probably fine in practice. Either way, it would be better than
        // panicking if sqids fails.
        let sqids_encoded = match crate::SQIDS.encode(&[self.0.cast_unsigned()]) {
            Ok(encoded) => encoded,
            #[expect(
                clippy::unwrap_used,
                reason = "There is no blocklist so this can't fail."
            )]
            Err(_) => crate::SQIDS_NO_BLOCKLIST
                .encode(&[self.0.cast_unsigned()])
                .unwrap(),
        };
        f.write_str(&sqids_encoded)
    }
}

impl FromStr for BountyNum {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // I think sqids doesn't take any blocklists into account when decoding so this is fine
        // even if the original ID was generated using SQIDS_NO_BLOCKLIST
        let id = crate::SQIDS.decode(s);
        match id.first() {
            Some(id) => {
                let result = Self(id.cast_signed());
                if result.to_string() == s {
                    Ok(result)
                } else {
                    Err(())
                }
            }
            None => Err(()),
        }
    }
}
