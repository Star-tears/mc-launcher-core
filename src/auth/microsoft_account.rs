use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::{distributions::Alphanumeric, Rng};
use ring::digest;
use std::collections::HashMap;
use url::Url;

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
}
