use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::{distributions::Alphanumeric, Rng};
use reqwest::blocking::Client;
use ring::digest;
use serde_json::json;
use std::collections::HashMap;
use url::Url;

use crate::{
    types::microsoft_types::{
        AuthorizationTokenResponse, MinecraftAuthenticateResponse, MinecraftProfileResponse,
        MinecraftStoreResponse, XBLResponse, XSTSResponse,
    },
    utils::helper::get_user_agent,
};

const AUTH_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/authorize";
const TOKEN_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/token";
const SCOPE: &str = "XboxLive.signin offline_access";

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
    let mut rng = rand::thread_rng();
    let chars: Vec<char> = (0..128)
        .map(|_| match rng.gen_range(0..64) {
            0 => '-',
            1 => '_',
            _ => rng.sample(Alphanumeric) as char,
        })
        .collect();
    let code_verifier: String = chars.iter().collect();

    let digest = digest::digest(&digest::SHA256, code_verifier.as_bytes());
    let code_challenge = URL_SAFE_NO_PAD.encode(digest.as_ref());
    code_challenge.trim_end_matches('=').to_string();
    let code_challenge_method = "S256".to_string();

    (code_verifier, code_challenge, code_challenge_method)
}

pub fn generate_state() -> String {
    let mut rng = rand::thread_rng();
    let chars: Vec<char> = (0..16)
        .map(|_| match rng.gen_range(0..64) {
            0 => '-',
            1 => '_',
            _ => rng.sample(Alphanumeric) as char,
        })
        .collect();
    let state: String = chars.iter().collect();
    state
}

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
    let url_with_query = url
        .join(&("?".to_owned() + &serde_urlencoded::to_string(parameters).unwrap()))
        .expect("Failed to build URL");
    (url_with_query.to_string(), state, code_verifier)
}

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

pub fn authenticate_with_xbl(access_token: &str) -> Result<XBLResponse, reqwest::Error> {
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

pub fn get_profile(access_token: &str) -> Result<MinecraftProfileResponse, reqwest::Error> {
    let client = Client::new();
    let res = client
        .get("https://api.minecraftservices.com/minecraft/profile")
        .header("Authorization", format!("Bearer {}", access_token))
        .header("user-agent", get_user_agent())
        .send()?;

    let profile_response: MinecraftProfileResponse = res.json()?;
    Ok(profile_response)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_code_challenge() {
        let code_verifier: String = "7BSNrJnbWnVrx9Y3uoBEJmrd0eii9ZBEQ5AVw_j4lzIlnsxwTDLJdtaiuCdrkJZ4fVH-E3v_hP7ynwS4zIwrSVCzG7vr5MTXahwESJnsb3SFM5zpdNjj525JbjrUwctt".to_string();
        let digest = digest::digest(&digest::SHA256, code_verifier.as_bytes());
        let code_challenge = URL_SAFE_NO_PAD.encode(digest.as_ref());
        code_challenge.trim_end_matches('=').to_string();
        assert_eq!(
            code_challenge,
            "bOQuaNvcR9utb6HhxpkDuvJr4Wh83ugr_FnH4dvTg9I".to_string()
        );
        let code_verifier: String = "sL0L64E7Qk_TANBue-ejOajO7LP3dcVI64ZgsjMsfV5dMhuDoFgb0Ldb4b7U3EXqBldbZJEAMJoxE8NfFmvm2oimm2FDQhy2qPDEoWUsY60mXF1poaw5cwvnpK-dXSFB".to_string();
        let digest = digest::digest(&digest::SHA256, code_verifier.as_bytes());
        let code_challenge = URL_SAFE_NO_PAD.encode(digest.as_ref());
        code_challenge.trim_end_matches('=').to_string();
        assert_eq!(
            code_challenge,
            "Nju8uPgZTErU1OxovBkfsGwykuhtCVCE-dGGhooiD8E".to_string()
        );
    }

    #[test]
    fn debug_generate_pkce_data() {
        println!("{:#?}", generate_pkce_data());
    }

    #[test]
    fn test_get_auth_code_from_url() {
        let url = "https://test.example.com/test?code1=2&code=13&t=sd";
        assert_eq!(get_auth_code_from_url(url), Some("13".to_string()));
    }
}
