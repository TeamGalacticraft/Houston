use crate::models::cape::Category;

#[derive(Copy)]
pub struct CapeModel {
    pub id: i64,
    pub name: String,
    pub category: Category,
    pub texture_url: String,
    pub legacy_name: Option<String>
}

impl CapeModel {
    pub async fn insert(
        &self,
        trans: &mut sqlx::Transaction<'_, sqlx::Postgres>
    ) -> Result<(), sqlx::error::Error> {
        sqlx::query!(
            "
            insert into capes (name, category, texture_url, legacy_name)
            values ($1, $2, $3, $4)
            ",
            self.name,
            self.category.to_string(),
            self.texture_url,
            self.legacy_name
        )
            .execute(&mut *trans)
            .await?;
        Ok(())
    }

    pub async fn get<'a, 'b, E>(
        id: i64,
        exec: E
    ) -> Result<Option<Self>, sqlx::error::Error>
    where E: sqlx::Executor<'a, Database=sqlx::Postgres>
    {
        let result = sqlx::query!(
            "
            select c.id, c.name, c.category, c.texture_url, c.legacy_name
            from capes c
            where c.id = $1
            ",
            id
        )
            .fetch_optional(exec)
            .await?;

        if let Some(row) = result {
            Ok(Some(CapeModel {
                id,
                name: row.name,
                category: Category::from_string(&row.category),
                texture_url: row.texture_url,
                legacy_name: row.legacy_name
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn remove(
        id: i64,
        trans: &mut sqlx::Transaction<'_, sqlx::Postgres>
    ) -> Result<Option<()>, sqlx::error::Error> {
        sqlx::query!(
            "
            delete from capes
            where id = $1
            ",
            id
        )
            .execute(&mut *trans)
            .await?;

        Ok(Some(()))
    }
}