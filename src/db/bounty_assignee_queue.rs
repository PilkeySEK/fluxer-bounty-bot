use chrono::Utc;
use fluxer_neptunium::model::id::{Id, marker::UserMarker};
use sqlx::{postgres::PgQueryResult, prelude::FromRow};

use crate::db::DbManager;

impl DbManager {
    pub async fn list_assignee_queue_for_bounty(
        &self,
        bounty_id: i64,
    ) -> anyhow::Result<Vec<QueuedBountyAssignee>> {
        let raw = sqlx::query_as!(
            QueuedBountyAssigneeSchema,
            "SELECT * FROM bounty_assignee_queue
            WHERE bounty_id = $1",
            bounty_id,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(raw.into_iter().map(Into::into).collect())
    }

    pub async fn count_assignees_for_bounty(&self, bounty_id: i64) -> anyhow::Result<i64> {
        Ok(sqlx::query_scalar!(
            "SELECT COUNT(*) FROM bounty_assignee_queue
            WHERE bounty_id = $1",
            bounty_id
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0))
    }

    pub async fn add_user_to_assignee_queue(
        &self,
        bounty_id: i64,
        user_id: Id<UserMarker>,
        queued_at: chrono::DateTime<Utc>,
    ) -> anyhow::Result<PgQueryResult> {
        Ok(sqlx::query!(
            "INSERT INTO bounty_assignee_queue (bounty_id, user_id, queued_at)
            VALUES ($1, $2, $3)
            ON CONFLICT (bounty_id, user_id) DO NOTHING",
            bounty_id,
            user_id.into_inner().cast_signed(),
            queued_at,
        )
        .execute(&self.pool)
        .await?)
    }

    pub async fn remove_user_from_assignee_queue(
        &self,
        bounty_id: i64,
        user_id: Id<UserMarker>,
    ) -> anyhow::Result<PgQueryResult> {
        Ok(sqlx::query!(
            "DELETE FROM bounty_assignee_queue
            WHERE bounty_id = $1 AND user_id = $2",
            bounty_id,
            user_id.into_inner().cast_signed(),
        )
        .execute(&self.pool)
        .await?)
    }
}

pub struct QueuedBountyAssignee {
    #[expect(unused)]
    pub bounty_id: i64,
    pub user_id: Id<UserMarker>,
    pub queued_at: chrono::DateTime<Utc>,
}

impl From<QueuedBountyAssigneeSchema> for QueuedBountyAssignee {
    fn from(value: QueuedBountyAssigneeSchema) -> Self {
        Self {
            bounty_id: value.bounty_id,
            user_id: value.user_id.cast_unsigned().into(),
            queued_at: value.queued_at,
        }
    }
}

#[derive(FromRow)]
struct QueuedBountyAssigneeSchema {
    bounty_id: i64,
    user_id: i64,
    queued_at: chrono::DateTime<Utc>,
}
