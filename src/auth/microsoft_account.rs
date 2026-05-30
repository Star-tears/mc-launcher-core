//! Microsoft account login helpers.
//!
//! The flow is split into small steps so desktop launchers can control browser
//! handling, redirect capture, token storage, and refresh scheduling. Use
//! [`get_secure_login_data`] for PKCE-enabled sign-in, then pass the returned
//! verifier to [`complete_login`] after the redirect URL yields an auth code.

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::{distr::Alphanumeric, Rng};
use reqwest::blocking::Client;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use url::Url;

use crate::{
    types::microsoft_types::{
        AuthorizationTokenResponse, CompleteLoginResponse, MinecraftAuthenticateResponse,
        MinecraftProfileResponse, MinecraftStoreResponse, XBLResponse, XSTSResponse,
    },
    utils::helper::get_user_agent,
};

const AUTH_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/authorize";
const TOKEN_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/token";
const SCOPE: &str = "XboxLive.signin offline_access";

/// Builds a Microsoft OAuth login URL without PKCE.
///
/// New applications should prefer [`get_secure_login_data`].
pub fn get_login_url(client_id: &str, redirect_uri: &str) -> String {
    let mut parameters = HashMap::new();
    parameters.insert("client_id", client_id);
    parameters.insert("response_type", "code");
    parameters.insert("redirect_uri", redirect_uri);
    parameters.insert("response_mode", "query");
    parameters.insert("scope", SCOPE);

    let url = Url::parse(AUTH_URL).expect("Invalid AUTH_URL");
    let url_with_query = url
        .join(&("?".to_owned() + &serde_urlencoded::to_string(parameters).unwrap()))
        .expect("Failed to build URL");

    url_with_query.to_string()
}

fn generate_pkce_data() -> (String, String, String) {
    let mut rng = rand::rng();
    let chars: Vec<char> = (0..128)
        .map(|_| match rng.random_range(0..64) {
            0 => '-',
            1 => '_',
            _ => rng.sample(Alphanumeric) as char,
        })
        .collect();
    let code_verifier: String = chars.iter().collect();

    let digest = Sha256::digest(code_verifier.as_bytes());
    let code_challenge = URL_SAFE_NO_PAD.encode(digest);
    code_challenge.trim_end_matches('=').to_string();
    let code_challenge_method = "S256".to_string();

    (code_verifier, code_challenge, code_challenge_method)
}

/// Generates a random OAuth state token.
pub fn generate_state() -> String {
    let mut rng = rand::rng();
    let chars: Vec<char> = (0..16)
        .map(|_| match rng.random_range(0..64) {
            0 => '-',
            1 => '_',
            _ => rng.sample(Alphanumeric) as char,
        })
        .collect();
    let state: String = chars.iter().collect();
    state
}

/// Builds a PKCE-enabled login URL, state token, and code verifier.
///
/// The returned tuple is `(login_url, state, code_verifier)`. Store the verifier
/// until the redirect is received, then pass it to [`complete_login`].
pub fn get_secure_login_data(
    client_id: &str,
    redirect_uri: &str,
    state: Option<&str>,
) -> (String, String, String) {
    let (code_verifier, code_challenge, code_challenge_method) = generate_pkce_data();

    let state = match state {
        Some(s) => s.to_string(),
        None => generate_state(),
    };

    let mut parameters = HashMap::new();
    parameters.insert("client_id", client_id);
    parameters.insert("response_type", "code");
    parameters.insert("redirect_uri", redirect_uri);
    parameters.insert("response_mode", "query");
    parameters.insert("scope", SCOPE);
    parameters.insert("state", &state);
    parameters.insert("code_challenge", &code_challenge);
    parameters.insert("code_challenge_method", &code_challenge_method);
    let url = Url::parse(AUTH_URL).expect("Invalid AUTH_URL");
    let login_url = url
        .join(&("?".to_owned() + &serde_urlencoded::to_string(parameters).unwrap()))
        .expect("Failed to build URL");
    (login_url.to_string(), state, code_verifier)
}

/// Returns true when a redirect URL contains a `code` query parameter.
pub fn url_contains_auth_code(url: &str) -> bool {
    if let Ok(parsed) = Url::parse(url) {
        if let Some(qs) = parsed.query() {
            let query_pairs: Vec<_> = qs.split('&').collect();
            for pair in query_pairs {
                let parts: Vec<_> = pair.split('=').collect();
                if let [key, _] = parts[..] {
                    if key == "code" {
                        return true;
                    }
                }
            }
        }
    }
    false
}

/// Extracts the raw `code` query parameter from a redirect URL.
pub fn get_auth_code_from_url(url: &str) -> Option<String> {
    if let Ok(parsed) = Url::parse(url) {
        if let Some(qs) = parsed.query() {
            let query_pairs: HashMap<_, _> = qs
                .split('&')
                .filter_map(|s| {
                    let mut split = s.split('=');
                    let key = split.next()?;
                    let value = split.next()?;
                    Some((key, value.to_string()))
                })
                .collect();
            if let Some(code) = query_pairs.get("code") {
                return Some(code.clone());
            }
        }
    }
    None
}

/// Parses a redirect URL and validates the optional state value.
///
/// # Errors
///
/// Returns an error when the URL has no auth code or the state value does not
/// match the expected state.
pub fn parse_auth_code_url(
    url: &str,
    state: Option<String>,
) -> Result<String, Box<dyn std::error::Error>> {
    if let Ok(parsed) = Url::parse(url) {
        if let Some(qs) = parsed.query() {
            let query_pairs: HashMap<_, _> = qs
                .split('&')
                .filter_map(|s| {
                    let mut split = s.split('=');
                    let key = split.next()?;
                    let value = split.next()?;
                    Some((key, value.to_string()))
                })
                .collect();
            if state.is_some() {
                if state != query_pairs.get("state").cloned() {
                    return Err("state not equal.".into());
                }
            }
            if let Some(code) = query_pairs.get("code") {
                return Ok(code.clone());
            }
        }
    }
    Err("parse_auth_code_url error.".into())
}

/// Exchanges an OAuth authorization code for Microsoft access and refresh tokens.
///
/// Pass the PKCE verifier returned by [`get_secure_login_data`] when using the
/// secure login flow.
///
/// # Errors
///
/// Returns a [`reqwest::Error`] if the HTTP request or response decoding fails.
pub fn get_authorization_token(
    client_id: &str,
    client_secret: Option<&str>,
    redirect_uri: &str,
    auth_code: &str,
    code_verifier: Option<&str>,
) -> Result<AuthorizationTokenResponse, reqwest::Error> {
    let mut parameters = HashMap::new();
    parameters.insert("client_id", client_id);
    parameters.insert("scope", SCOPE);
    parameters.insert("code", auth_code);
    parameters.insert("redirect_uri", redirect_uri);
    parameters.insert("grant_type", "authorization_code");

    if let Some(secret) = client_secret {
        parameters.insert("client_secret", secret);
    }

    if let Some(verifier) = code_verifier {
        parameters.insert("code_verifier", verifier);
    }

    let client = Client::new();
    let res = client
        .post(TOKEN_URL)
        .form(&parameters)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("user-agent", get_user_agent())
        .send()?;

    let token_response: AuthorizationTokenResponse = res.json()?;
    Ok(token_response)
}

/// Refreshes Microsoft OAuth tokens using a refresh token.
///
/// # Errors
///
/// Returns a [`reqwest::Error`] if the HTTP request or response decoding fails.
pub fn refresh_authorization_token(
    client_id: &str,
    client_secret: Option<&str>,
    refresh_token: &str,
) -> Result<AuthorizationTokenResponse, reqwest::Error> {
    let mut parameters = HashMap::new();
    parameters.insert("client_id", client_id);
    parameters.insert("scope", SCOPE);
    parameters.insert("refresh_token", refresh_token);
    parameters.insert("grant_type", "refresh_token");

    if let Some(secret) = client_secret {
        parameters.insert("client_secret", secret);
    }

    let client = Client::new();
    let res = client
        .post("https://login.live.com/oauth20_token.srf")
        .form(&parameters)
        .header("user-agent", get_user_agent())
        .send()?;

    let token_response: AuthorizationTokenResponse = res.json()?;
    Ok(token_response)
}

/// Authenticates a Microsoft access token with Xbox Live.
///
/// # Errors
///
/// Returns an error if the Xbox Live request or response decoding fails.
pub fn authenticate_with_xbl(
    access_token: &str,
) -> Result<XBLResponse, Box<dyn std::error::Error>> {
    let mut parameters = HashMap::new();
    parameters.insert(
        "Properties",
        json!({
            "AuthMethod": "RPS",
            "SiteName": "user.auth.xboxlive.com",
            "RpsTicket": format!("d={}", access_token),
        }),
    );
    parameters.insert("RelyingParty", "http://auth.xboxlive.com".into());
    parameters.insert("TokenType", "JWT".into());

    let client = Client::new();
    let res = client
        .post("https://user.auth.xboxlive.com/user/authenticate")
        .json(&parameters)
        .header("Content-Type", "application/json")
        .header("user-agent", get_user_agent())
        .header("Accept", "application/json")
        .send()?;

    let xbl_response: XBLResponse = res.json()?;
    Ok(xbl_response)
}

/// Exchanges an Xbox Live token for an XSTS token.
///
/// # Errors
///
/// Returns a [`reqwest::Error`] if the HTTP request or response decoding fails.
pub fn authenticate_with_xsts(xbl_token: &str) -> Result<XSTSResponse, reqwest::Error> {
    let mut parameters = HashMap::new();
    parameters.insert(
        "Properties",
        json!({
            "SandboxId": "RETAIL",
            "UserTokens": [xbl_token],
        }),
    );
    parameters.insert("RelyingParty", "rp://api.minecraftservices.com/".into());
    parameters.insert("TokenType", "JWT".into());

    let client = Client::new();
    let res = client
        .post("https://xsts.auth.xboxlive.com/xsts/authorize")
        .json(&parameters)
        .header("Content-Type", "application/json")
        .header("user-agent", get_user_agent())
        .header("Accept", "application/json")
        .send()?;

    let xsts_response: XSTSResponse = res.json()?;
    Ok(xsts_response)
}

/// Exchanges XSTS identity data for a Minecraft services access token.
///
/// # Errors
///
/// Returns a [`reqwest::Error`] if the HTTP request or response decoding fails.
pub fn authenticate_with_minecraft(
    userhash: &str,
    xsts_token: &str,
) -> Result<MinecraftAuthenticateResponse, reqwest::Error> {
    let parameters = json!({
        "identityToken": format!("XBL3.0 x={};{}", userhash, xsts_token),
    });

    let client = Client::new();
    let res = client
        .post("https://api.minecraftservices.com/authentication/login_with_xbox")
        .json(&parameters)
        .header("Content-Type", "application/json")
        .header("user-agent", get_user_agent())
        .header("Accept", "application/json")
        .send()?;

    let minecraft_response: MinecraftAuthenticateResponse = res.json()?;
    Ok(minecraft_response)
}

/// Fetches Minecraft store entitlement information for an access token.
///
/// # Errors
///
/// Returns a [`reqwest::Error`] if the HTTP request or response decoding fails.
pub fn get_store_information(access_token: &str) -> Result<MinecraftStoreResponse, reqwest::Error> {
    let client = Client::new();
    let res = client
        .get("https://api.minecraftservices.com/entitlements/mcstore")
        .header("Authorization", format!("Bearer {}", access_token))
        .header("user-agent", get_user_agent())
        .send()?;

    let store_response: MinecraftStoreResponse = res.json()?;
    Ok(store_response)
}

/// Fetches the Minecraft profile for an authenticated account.
///
/// # Errors
///
/// Returns an error if the profile request or response decoding fails.
pub fn get_profile(
    access_token: &str,
) -> Result<MinecraftProfileResponse, Box<dyn std::error::Error>> {
    let client = Client::new();
    let res = client
        .get("https://api.minecraftservices.com/minecraft/profile")
        .header("Authorization", format!("Bearer {}", access_token))
        .header("user-agent", get_user_agent())
        .send()?;

    let profile_response: MinecraftProfileResponse = res.json()?;
    Ok(profile_response)
}

/// Completes the full Microsoft-to-Minecraft login flow.
///
/// This exchanges the OAuth code, authenticates with Xbox Live and XSTS, logs in
/// to Minecraft services, and returns profile plus token data.
///
/// # Errors
///
/// Returns an error if any network step fails, the app is not permitted, or the
/// account does not own Minecraft.
pub fn complete_login(
    client_id: &str,
    client_secret: Option<&str>,
    redirect_uri: &str,
    auth_code: &str,
    code_verifier: Option<&str>,
) -> Result<CompleteLoginResponse, Box<dyn std::error::Error>> {
    let token_request = get_authorization_token(
        client_id,
        client_secret,
        redirect_uri,
        auth_code,
        code_verifier,
    )?;
    let token = token_request.access_token;

    let xbl_request = authenticate_with_xbl(&token)?;
    let xbl_token = xbl_request.token;
    let userhash = xbl_request.display_claims.xui[0].uhs.clone();

    let xsts_request = authenticate_with_xsts(&xbl_token)?;
    let xsts_token = xsts_request.token;

    let account_request = authenticate_with_minecraft(&userhash, &xsts_token)?;

    if account_request.access_token.is_empty() {
        return Err("Azure App not permitted.".into());
    }

    let access_token = account_request.access_token.clone();

    let profile = get_profile(&access_token)?;

    if profile.error == Some("NOT_FOUND".to_string()) {
        return Err("Account not own minecraft".into());
    }

    Ok(CompleteLoginResponse {
        id: profile.id,
        name: profile.name,
        access_token: account_request.access_token,
        refresh_token: token_request.refresh_token,
        skins: profile.skins,
        capes: profile.capes,
        error: profile.error,
        error_message: profile.error_message,
    })
}

/// Completes the full token refresh flow.
///
/// This refreshes Microsoft OAuth tokens, then repeats Xbox Live, XSTS, and
/// Minecraft services authentication to return fresh launch credentials.
///
/// # Errors
///
/// Returns an error if the refresh token is invalid, any network step fails, or
/// the account does not own Minecraft.
pub fn complete_refresh(
    client_id: &str,
    client_secret: Option<&str>,
    refresh_token: &str,
) -> Result<CompleteLoginResponse, Box<dyn std::error::Error>> {
    let token_request = refresh_authorization_token(client_id, client_secret, refresh_token)?;

    if token_request.error.is_some() {
        return Err("Invalid Refresh Token.".into());
    }

    let token = token_request.access_token;

    let xbl_request = authenticate_with_xbl(&token)?;
    let xbl_token = xbl_request.token;
    let userhash = xbl_request.display_claims.xui[0].uhs.clone();

    let xsts_request = authenticate_with_xsts(&xbl_token)?;
    let xsts_token = xsts_request.token;

    let account_request = authenticate_with_minecraft(&userhash, &xsts_token)?;
    let access_token = account_request.access_token.clone();

    let profile = get_profile(&access_token)?;

    if profile.error == Some("NOT_FOUND".to_string()) {
        return Err("Account not own minecraft".into());
    }

    Ok(CompleteLoginResponse {
        id: profile.id,
        name: profile.name,
        access_token: account_request.access_token,
        refresh_token: token_request.refresh_token,
        skins: profile.skins,
        capes: profile.capes,
        error: profile.error,
        error_message: profile.error_message,
    })
}

#[cfg(test)]
mod test {
    use super::*;

    // test with minecraft-console-client public client_id and redirecr_uri
    const CLIENT_ID: &str = "54473e32-df8f-42e9-a649-9419b0dab9d3";
    const REDIRECT_URI: &str = "https://mccteam.github.io/redirect.html";

    #[test]
    fn debug_get_login_url() {
        dbg!(get_login_url(CLIENT_ID, REDIRECT_URI));
    }

    #[test]
    fn debug_generate_pkce_data() {
        dbg!(generate_pkce_data());
    }

    #[test]
    fn debug_get_secure_login_data() {
        dbg!(get_secure_login_data(CLIENT_ID, REDIRECT_URI, None));
    }

    #[test]
    fn test_code_challenge() {
        let code_verifier: String = "7BSNrJnbWnVrx9Y3uoBEJmrd0eii9ZBEQ5AVw_j4lzIlnsxwTDLJdtaiuCdrkJZ4fVH-E3v_hP7ynwS4zIwrSVCzG7vr5MTXahwESJnsb3SFM5zpdNjj525JbjrUwctt".to_string();
        let digest = Sha256::digest(code_verifier.as_bytes());
        let code_challenge = URL_SAFE_NO_PAD.encode(digest);
        code_challenge.trim_end_matches('=').to_string();
        assert_eq!(
            code_challenge,
            "bOQuaNvcR9utb6HhxpkDuvJr4Wh83ugr_FnH4dvTg9I".to_string()
        );
        let code_verifier: String = "sL0L64E7Qk_TANBue-ejOajO7LP3dcVI64ZgsjMsfV5dMhuDoFgb0Ldb4b7U3EXqBldbZJEAMJoxE8NfFmvm2oimm2FDQhy2qPDEoWUsY60mXF1poaw5cwvnpK-dXSFB".to_string();
        let digest = Sha256::digest(code_verifier.as_bytes());
        let code_challenge = URL_SAFE_NO_PAD.encode(digest);
        code_challenge.trim_end_matches('=').to_string();
        assert_eq!(
            code_challenge,
            "Nju8uPgZTErU1OxovBkfsGwykuhtCVCE-dGGhooiD8E".to_string()
        );
    }

    #[test]
    fn test_get_auth_code_from_url() {
        let url = "https://test.example.com/test?code1=2&code=13&t=sd";
        assert_eq!(get_auth_code_from_url(url), Some("13".to_string()));
    }
}
