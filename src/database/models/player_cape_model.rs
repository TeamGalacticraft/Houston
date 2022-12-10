use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::database::models::cape_model::CapeModel;
use crate::models::cape::Category;

pub struct PlayerCapeModel {
    pub player: Uuid,
    pub cape: Option<i64>
}

#[derive(Serialize, Deserialize)]
pub struct LegacyPlayerCapeModel {
    pub uuid: Uuid,
    pub name: String,
    pub cape: String
}

impl PlayerCapeModel {
    pub async fn insert(
        &self,
        trans: &mut sqlx::Transaction<'_, sqlx::Postgres>
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "
            insert into player_capes (player, cape)
            values ($1, $2)
            ",
            self.player,
            self.cape
        )
            .execute(&mut *trans)
            .await?;
        Ok(())
    }

    pub async fn get_cape_for<'a, 'b, E>(
        player: Uuid,
        exec: E
    ) -> Result<Option<CapeModel>, sqlx::Error>
    where E: sqlx::Executor<'a, Database = sqlx::Postgres> {
        let result = sqlx::query!(
            "
            select c.id, c.name, c.category, c.texture_url, c.legacy_name
            from capes c, player_capes pc
            where c.id = pc.cape and pc.player = $1
            ",
            player
        )
            .fetch_optional(exec)
            .await?;

        if let Some(row) = result {
            Ok(Some(CapeModel {
                id: row.id,
                name: row.name,
                category: Category::from_string(&row.category),
                texture_url: row.texture_url,
                legacy_name: row.legacy_name
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_list<'a, E>(
        exec: E
    ) -> Result<HashMap<Uuid, CapeModel>, sqlx::Error>
    where E: sqlx::Executor<'a, Database = sqlx::Postgres>
    {
        let result = sqlx::query!(
            "
select pc.player, c.id, c.name, c.category, c.texture_url, c.legacy_name
from player_capes pc, capes c
where pc.cape = c.id
"
        )
            .fetch_all(exec)
            .await?;

        let mut out: HashMap<Uuid, CapeModel> = HashMap::new();

        for row in result {
            out.insert(row.player, CapeModel {
                id: row.id,
                name: row.name,
                category: Category::from_string(&*row.category),
                texture_url: row.texture_url,
                legacy_name: row.legacy_name
            });
        }

        Ok(out)
    }

    pub async fn get_legacy_list<'a, E>(
        exec: E
    ) ->  Result<Vec<LegacyPlayerCapeModel>, sqlx::Error>
    where E: sqlx::Executor<'a, Database = sqlx::Postgres>
    {
        let result = sqlx::query!(
            "
            select pc.player, u.username, c.legacy_name
            from capes c, users u, player_capes pc
            where pc.cape = c.id and c.legacy_name is not null
            "
        )
            .fetch_all(exec)
            .await?;

        let mut capes: Vec<LegacyPlayerCapeModel> = Vec::new();
        for row in result {
            capes.push(LegacyPlayerCapeModel {
                uuid: row.player,
                name: row.username,
                cape: row.legacy_name.expect("selected to make sure this is not null"),
            })
        }

        Ok(capes)
    }

    pub async fn remove(
        player: Uuid,
        trans: &mut sqlx::Transaction<'_, sqlx::Postgres>
    ) -> Result<Option<()>, sqlx::Error> {
        sqlx::query!(
            "
            delete from player_capes
            where player = $1
            ",
            player
        )
            .execute(&mut *trans)
            .await?;

        Ok(Some(()))
    }
}