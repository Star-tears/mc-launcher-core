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
