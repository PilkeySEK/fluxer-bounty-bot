use std::sync::Arc;

use anyhow::bail;
use bitflags::bitflags;
use fluxer_neptunium::model::id::{
    Id,
    marker::{GuildMarker, RoleMarker, UserMarker},
};

use crate::db::DbManager;

impl DbManager {
    pub async fn list_guild_permissions(
        &self,
        guild_id: Id<GuildMarker>,
    ) -> anyhow::Result<Arc<Vec<GuildPermissionsEntry>>> {
        if let Some(permissions) = self.cached_guild_permissions.get(&guild_id) {
            return Ok(permissions);
        }
        let raw = sqlx::query_as!(
            GuildPermissionsEntrySchema,
            "SELECT * FROM guild_permissions
            WHERE guild_id = $1",
            guild_id.into_inner().cast_signed()
        )
        .fetch_all(&self.pool)
        .await?;
        let permissions = Arc::new(
            raw.into_iter()
                .map(TryInto::try_into)
                .collect::<anyhow::Result<_>>()?,
        );
        self.cached_guild_permissions
            .insert(guild_id, Arc::clone(&permissions));
        Ok(permissions)
    }
}

bitflags! {
    #[derive(Copy, Clone)]
    pub struct BotPermissions: u64 {
        const CREATE_BOUNTIES = 1 << 0;
        const MANAGE_BOUNTIES = 1 << 1;
        const MANAGE_GUILD_CONFIG = 1 << 2;
    }
}

pub enum GuildPermissionEntity {
    User(Id<UserMarker>),
    Role(Id<RoleMarker>),
}

pub struct GuildPermissionsEntry {
    pub guild_id: Id<GuildMarker>,
    pub entity: GuildPermissionEntity,
    pub allow: BotPermissions,
}

pub struct GuildPermissionsEntrySchema {
    pub guild_id: i64,
    pub kind: String,
    pub entity_id: i64,
    pub allow: i64,
}

impl TryFrom<GuildPermissionsEntrySchema> for GuildPermissionsEntry {
    type Error = anyhow::Error;
    fn try_from(value: GuildPermissionsEntrySchema) -> Result<Self, Self::Error> {
        Ok(Self {
            guild_id: value.guild_id.cast_unsigned().into(),
            entity: match value.kind.to_lowercase().as_str() {
                "user" => GuildPermissionEntity::User(value.entity_id.cast_unsigned().into()),
                "role" => GuildPermissionEntity::Role(value.entity_id.cast_unsigned().into()),
                _ => bail!("Failed to parse value.kind which is \"{}\"", value.kind),
            },
            allow: BotPermissions::from_bits_truncate(value.allow.cast_unsigned()),
        })
    }
}
