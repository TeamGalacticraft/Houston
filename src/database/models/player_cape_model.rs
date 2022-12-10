use uuid::Uuid;
use crate::database::models::cape_model::CapeModel;
use crate::models::cape::Category;

pub struct PlayerCapeModel {
    pub player: Uuid,
    pub cape: Option<i64>
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
            select c.id, c.name, c.category, c.texture_url
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
            }))
        } else {
            Ok(None)
        }
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