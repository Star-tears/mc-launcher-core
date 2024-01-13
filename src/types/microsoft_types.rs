use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct AuthorizationTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i32,
    pub scope: String,
    pub refresh_token: String,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Xui {
    pub uhs: String,
}

#[derive(Debug, Deserialize)]
pub struct DisplayClaims {
    pub xui: Vec<Xui>,
}

#[derive(Debug, Deserialize)]
pub struct XBLResponse {
    #[serde(rename = "IssueInstant")]
    pub issue_instant: String,
    #[serde(rename = "NotAfter")]
    pub not_after: String,
    #[serde(rename = "Token")]
    pub token: String,
    #[serde(rename = "DisplayClaims")]
    pub display_claims: DisplayClaims,
}

#[derive(Debug, Deserialize)]
pub struct XSTSResponse {
    #[serde(rename = "IssueInstant")]
    pub issue_instant: String,
    #[serde(rename = "NotAfter")]
    pub not_after: String,
    #[serde(rename = "Token")]
    pub token: String,
    #[serde(rename = "DisplayClaimns")]
    pub display_claimns: DisplayClaims,
}

#[derive(Debug, Deserialize)]
pub struct MinecraftStoreItem {
    pub name: String,
    pub signature: String,
}

#[derive(Debug, Deserialize)]
pub struct MinecraftStoreResponse {
    pub items: Vec<MinecraftStoreItem>,
    pub signature: String,
    #[serde(rename = "keyId")]
    pub key_id: String,
}

#[derive(Debug, Deserialize)]
pub struct MinecraftAuthenticateResponse {
    pub username: String,
    pub roles: Vec<Value>,
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i32,
}

#[derive(Debug, Deserialize)]
pub struct MinecraftProfileInfo {
    pub id: String,
    pub state: String,
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct MinecraftProfileSkin {
    pub id: String,
    pub state: String,
    pub url: String,
    pub variant: String,
}

#[derive(Debug, Deserialize)]
pub struct MinecraftProfileCape {
    pub id: String,
    pub state: String,
    pub url: String,
    pub alias: String,
}

#[derive(Debug, Deserialize)]
pub struct MinecraftProfileResponse {
    pub id: String,
    pub name: String,
    pub skins: Vec<MinecraftProfileSkin>,
    pub capes: Vec<MinecraftProfileCape>,
    pub error: String,
    pub error_message: String,
}

#[derive(Debug, Deserialize)]
pub struct CompleteLoginResponse {
    pub id: String,
    pub name: String,
    pub skins: Vec<MinecraftProfileSkin>,
    pub capes: Vec<MinecraftProfileCape>,
    pub error: String,
    pub error_message: String,
    pub access_token: String,
    pub refresh_token: String,
}
