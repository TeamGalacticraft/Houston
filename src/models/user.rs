use std::fmt::Formatter;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::database::models::user_model::UserModel;

#[derive(Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub avatar_url: String,
    pub roles: Vec<Role>,
    pub created: DateTime<Utc>
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    ModernCapes,
    LegacyCapes,
    Developer,
    Admin
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Role {
    pub fn from_string(string: &str) -> Role {
        match string {
            "modern_capes" => Role::ModernCapes,
            "legacy_capes" => Role::LegacyCapes,
            "developer" => Role::Developer,
            "admin" => Role::Admin,
            _ => Role::ModernCapes
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Role::ModernCapes => "modern_capes",
            Role::LegacyCapes => "legacy_capes",
            Role::Developer => "developer",
            Role::Admin => "admin"
        }
    }
}

impl From<UserModel> for User {
    fn from(model: UserModel) -> Self {
        Self {
            id: model.id,
            username: model.username,
            avatar_url: model.avatar_url,
            roles: model.roles,
            created: model.created,
        }
    }
}