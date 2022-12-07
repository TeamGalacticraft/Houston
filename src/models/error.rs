use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ApiError<'a> {
    pub err: &'a str,
    pub msg: &'a str,
}