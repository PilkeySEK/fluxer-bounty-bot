use std::str::FromStr;

use anyhow::Context;
use chrono::{DateTime, Utc};
use fluxer_neptunium::model::id::{
    Id,
    marker::{ChannelMarker, GuildMarker, MessageMarker, UserMarker},
};

use crate::db::DbManager;

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
    pub async fn set_bounty_state(
        &self,
        guild_id: Id<GuildMarker>,
        bounty_number: BountyNum,
        state: BountyState,
    ) -> anyhow::Result<()> {
        sqlx::query!(
            "UPDATE bounties
            SET state = $1
            WHERE guild_id = $2 AND bounty_number = $3",
            state.to_string(),
            guild_id.into_inner().cast_signed(),
            bounty_number.0,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

#[derive(strum::Display, strum::EnumString, PartialEq, Eq)]
pub enum BountyState {
    /// The bounty has been implemented (implies Approved).
    Finished,
    /// The bounty has been approved but is not implemented yet.
    Approved,
    /// The bounty is pending approval.
    Pending,
    /// The bounty has been rejected.
    Rejected,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct BountyContent {}

pub struct BountyRelatedMessage {
    pub channel_id: Id<ChannelMarker>,
    pub message_id: Id<MessageMarker>,
}

#[expect(clippy::struct_field_names)]
pub struct Bounty {
    pub bounty_id: i64,
    pub bounty_number: BountyNum,
    pub guild_id: Id<GuildMarker>,
    pub created_by: Id<UserMarker>,
    pub content: BountyContent,
    pub state: BountyState,
    pub created_at: DateTime<Utc>,
    pub claimed_by: Option<Id<UserMarker>>,
    pub related_message: Option<BountyRelatedMessage>,
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
            claimed_by: value.claimed_by.map(|id| id.cast_unsigned().into()),
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
        })
    }
}

pub struct BountySchema {
    pub bounty_id: i64,
    pub bounty_number: i64,
    pub guild_id: i64,
    pub created_by: i64,
    pub content: serde_json::Value,
    pub state: String,
    pub created_at: DateTime<Utc>,
    pub claimed_by: Option<i64>,
    pub related_message_id: Option<i64>,
    pub related_channel_id: Option<i64>,
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
