use fluxer_neptunium::model::id::{Id, marker::UserMarker};

use crate::db::DbManager;

impl DbManager {
    pub async fn list_bounty_stakeholders(
        &self,
        bounty_id: i64,
    ) -> anyhow::Result<Vec<BountyStakeholder>> {
        let raw = sqlx::query_as!(
            BountyStakeholderSchema,
            "SELECT * FROM bounty_stakeholders
            WHERE bounty_id = $1",
            bounty_id,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(raw.into_iter().map(From::from).collect())
    }

    /// Note that duplicates are allowed and supported.
    pub async fn add_bounty_stakeholder(
        &self,
        stakeholder: BountyStakeholder,
    ) -> anyhow::Result<()> {
        sqlx::query!(
            "INSERT INTO bounty_stakeholders (bounty_id, user_id, amount, note)
            VALUES ($1, $2, $3, $4)",
            stakeholder.bounty_id,
            stakeholder.user_id.into_inner().cast_signed(),
            stakeholder.amount,
            stakeholder.note,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn remove_bounty_stakeholder(
        &self,
        bounty_id: i64,
        user_id: Id<UserMarker>,
    ) -> anyhow::Result<()> {
        sqlx::query!(
            "DELETE FROM bounty_stakeholders
            WHERE bounty_id = $1 AND user_id = $2",
            bounty_id,
            user_id.into_inner().cast_signed(),
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

pub struct BountyStakeholder {
    pub bounty_id: i64,
    pub user_id: Id<UserMarker>,
    pub amount: i32,
    pub note: Option<String>,
}

impl From<BountyStakeholderSchema> for BountyStakeholder {
    fn from(value: BountyStakeholderSchema) -> Self {
        Self {
            bounty_id: value.bounty_id,
            user_id: value.user_id.cast_unsigned().into(),
            amount: value.amount,
            note: value.note,
        }
    }
}

struct BountyStakeholderSchema {
    bounty_id: i64,
    user_id: i64,
    amount: i32,
    note: Option<String>,
}
