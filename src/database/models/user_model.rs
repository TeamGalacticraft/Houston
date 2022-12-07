use chrono::{DateTime, Utc};
use uuid::Uuid;
use crate::models::user::Role;

pub struct UserModel {
    pub id: Uuid,
    pub username: String,
    pub avatar_url: String,
    pub roles: Vec<Role>,
    pub created: DateTime<Utc>,
}

impl UserModel {
    pub async fn insert(
        &self,
        trans: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), sqlx::error::Error> {
        let role_strings: Vec<String>
            = self.roles.iter().map(|x| x.to_string()).collect::<Vec<String>>();
        sqlx::query!(
            "
            insert into users (
                id, username, avatar_url,
                roles, created
            ) values ($1, $2, $3, $4, $5)
            ",
            self.id,
            self.username,
            self.avatar_url,
            &role_strings,
            self.created
        )
            .execute(&mut *trans)
            .await?;

        Ok(())
    }

    pub async fn get<'a, 'b, E>(
        id: Uuid,
        exec: E,
    ) -> Result<Option<Self>, sqlx::error::Error>
        where E: sqlx::Executor<'a, Database=sqlx::Postgres>,
    {
        let result = sqlx::query!(
            "
            select u.id, u.username, u.avatar_url, u.roles, u.created
            from users u
            where u.id = $1
            ",
            id
        )
            .fetch_optional(exec)
            .await?;

        if let Some(row) = result {
            Ok(Some(UserModel {
                id,
                username: row.username,
                avatar_url: row.avatar_url,
                roles: row.roles.iter().map(|x| Role::from_string(&x)).collect(),
                created: row.created,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn remove(
        id: Uuid,
        trans: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<Option<()>, sqlx::error::Error> {
        sqlx::query!(
            "
            delete from users
            where id = $1
            ",
            id
        )
            .execute(&mut *trans)
            .await?;

        Ok(Some(()))
    }
}