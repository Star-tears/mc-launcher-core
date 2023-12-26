use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AuthorizationTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i32,
    pub scope: String,
    pub refresh_token: String,
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
    pub IssueInstant: String,
    pub NotAfter: String,
    pub Token: String,
    pub DisplayClaims: DisplayClaims,
}

#[derive(Debug, Deserialize)]
pub struct XSTSResponse {
    pub IssueInstant: String,
    pub NotAfter: String,
    pub Token: String,
    pub DisplayClaimns: DisplayClaims,
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
    pub keyId: String,
}

#[derive(Debug, Deserialize)]
pub struct MinecraftAuthenticateResponse {
    pub username: String,
    pub roles: Vec<String>,
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
    pub errorMessage: String,
}

#[derive(Debug, Deserialize)]
pub struct CompleteLoginResponse {
    pub id: String,
    pub name: String,
    pub skins: Vec<MinecraftProfileSkin>,
    pub capes: Vec<MinecraftProfileCape>,
    pub error: String,
    pub errorMessage: String,
    pub access_token: String,
    pub refresh_token: String,
}
