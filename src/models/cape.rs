use std::fmt::Formatter;
use serde::{Deserialize, Serialize};
use crate::database::models::cape_model::CapeModel;

#[derive(Serialize, Deserialize, Clone)]
pub struct Cape {
    pub id: i64,
    pub name: String,
    pub category: Category,
    pub texture_url: String,
    pub legacy_name: Option<String>
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Category {
    Modern,
    Legacy,
    Developer
}

impl std::fmt::Display for Category {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Category {
    pub fn from_string(string: &str) -> Category {
        match string {
            "modern" => Category::Modern,
            "legacy" => Category::Legacy,
            "developer" => Category::Developer,
            _ => Category::Modern
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Category::Modern => "modern",
            Category::Legacy => "legacy",
            Category::Developer => "developer"
        }
    }
}

impl From<CapeModel> for Cape {
    fn from(model: CapeModel) -> Self {
        Cape {
            id: model.id,
            name: model.name,
            category: model.category,
            texture_url: model.texture_url,
            legacy_name: model.legacy_name
        }
    }
}