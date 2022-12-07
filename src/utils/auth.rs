use std::collections::HashMap;
use actix_web::http::header::HeaderMap;
use serde::{Deserialize, Serialize};
use serde_json::json;
use thiserror::Error;
use uuid::Uuid;
use crate::database::models;
use crate::database::models::user_model::UserModel;
use crate::models::user::User;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Env error")]
    Env(#[from] dotenvy::Error),
    #[error("Unknown DB error: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("DB error: {0}")]
    DB(#[from] models::DBError),
    #[error("JSON error: {0}")]
    SerDe(#[from] serde_json::Error),
    #[error("Error communicating with Microsoft: {0}")]
    MSA(#[from] reqwest::Error),
    #[error("Invalid credentials")]
    InvalidCreds,
    #[error("Uuid parse error: {0}")]
    Uuid(#[from] uuid::Error)
}

// microsoft oauth2
#[derive(Deserialize, Serialize, Debug)]
struct AuthorizationTokenResponse {
    token_type: String,
    scope: String,
    expires_in: u32,
    ext_expires_in: u32,
    access_token: String,
    refresh_token: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct XboxLiveAuthenticationResponse {
    issue_instant: String,
    not_after: String,
    token: String,
    // { "xui": [{"uhs": "xbl_token"}] }
    display_claims: HashMap<String, Vec<HashMap<String, String>>>,
}

#[derive(Deserialize, Serialize, Debug)]
struct MinecraftAuthenticationResponse {
    username: String,
    access_token: String,
    token_type: String,
    expires_in: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MinecraftProfileSkin {
    pub id: String,
    pub state: String,
    pub url: String,
    pub variant: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MinecraftProfileCape {
    pub id: String,
    pub state: String,
    pub url: String,
    pub alias: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MinecraftProfile {
    // player UUID
    pub id: String,
    // player username
    pub name: String,
    pub skins: Vec<MinecraftProfileSkin>,
    pub capes: Vec<MinecraftProfileCape>
}

fn get_reqwest_client() -> reqwest::ClientBuilder {
    reqwest::ClientBuilder::new()
        .https_only(true)
        .user_agent(format!("TeamGalacticraft/Houston@{}", env!("CARGO_PKG_VERSION")))
        .use_native_tls()
}

pub async fn get_token_from_msa_code(
    code: String
) -> Result<String, AuthError> {
    let client = get_reqwest_client().build()?;
    let client_id = dotenvy::var("MICROSOFT_CLIENT_ID")?;
    let client_secret = dotenvy::var("MICROSOFT_CLIENT_SECRET")?;

    let code_auth = client
        .post("https://login.microsoftonline.com/consumers/oauth2/v2.0/token")
        .form(&vec![
            ("client_id", client_id),
            ("client_secret", client_secret),
            ("code", code),
            ("grant_type", "authorization_code".to_string()),
            ("redirect_uri", "http://localhost:8090/v1/auth/microsoft".to_string()),
        ])
        .send()
        .await?
        .json::<AuthorizationTokenResponse>()
        .await?;

    let xbl_auth = client
        .post("https://user.auth.xboxlive.com/user/authenticate")
        .json(&json!({
            "Properties": {
                "AuthMethod": "RPS",
                "SiteName": "user.auth.xboxlive.com",
                "RpsTicket": format!("d={}", code_auth.access_token)
            },
            "RelyingParty": "http://auth.xboxlive.com",
            "TokenType": "JWT"
        }))
        .send()
        .await?
        .json::<XboxLiveAuthenticationResponse>()
        .await?;

    let xbl_security = client
        .post("https://xsts.auth.xboxlive.com/xsts/authorize")
        .json(&json!({
            "Properties": {
                "SandboxId": "RETAIL",
                "UserTokens": [&xbl_auth.token]
            },
            "RelyingParty": "rp://api.minecraftservices.com/",
            "TokenType": "JWT"
        }))
        .send()
        .await?
        .json::<XboxLiveAuthenticationResponse>()
        .await?;

    let mc_auth: MinecraftAuthenticationResponse = client
        .post("https://api.minecraftservices.com/authentication/login_with_xbox")
        .json(&json!({
            "identityToken":
                format!(
                "XBL3.0 x={user_hash};{xsts_token}",
                user_hash = &xbl_auth.display_claims["xui"][0]["uhs"],
                xsts_token = &xbl_security.token
            )
        }))
        .send()
        .await?
        .json()
        .await?;

    Ok(mc_auth.access_token)
}

pub async fn get_profile_from_token(
    token: &str
) -> Result<MinecraftProfile, AuthError> {
    let client = get_reqwest_client().build()?;
    let mc_profile: MinecraftProfile = client
        .get("https://api.minecraftservices.com/minecraft/profile")
        .bearer_auth(&token)
        .send()
        .await?
        .json()
        .await?;

    Ok(mc_profile)
}

pub async fn get_user_from_token<'a, 'b, E>(
    token: &str,
    exec: E
) -> Result<User, AuthError>
where E: sqlx::Executor<'a, Database = sqlx::Postgres>
{
    let profile = get_profile_from_token(token).await?;
    let user_model = UserModel::get(Uuid::parse_str(profile.id.as_str())?, exec).await?.unwrap();

    Ok(User::from(user_model))
}

pub async fn get_user_from_headers<'a, 'b, E>(
    headers: &HeaderMap,
    exec: E
) -> Result<User, AuthError>
where E: sqlx::Executor<'a, Database = sqlx::Postgres>
{
    let token = headers
        .get("Authorization")
        .ok_or(AuthError::InvalidCreds)?
        .to_str()
        .map_err(|_| AuthError::InvalidCreds)?;

    get_user_from_token(token, exec).await
}