use thiserror::Error;

pub mod user_model;
pub mod cape_model;

#[derive(Error, Debug)]
pub enum DBError {
    #[error("Error while interacting with database: {0}")]
    Database(#[from] sqlx::error::Error),
    #[error("A DB request failed")]
    Other(String)
}