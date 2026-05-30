//! Blocking HTTP helpers with the crate user agent.

use reqwest::blocking::Client;

use crate::Result;

/// Returns the user agent used by crate-managed HTTP requests.
pub fn user_agent() -> String {
    format!("mc-launcher-core/{}", env!("CARGO_PKG_VERSION"))
}

/// Builds a blocking reqwest client.
///
/// # Errors
///
/// Returns [`crate::LauncherError`] if the client cannot be constructed.
pub fn client() -> Result<Client> {
    Ok(Client::builder().user_agent(user_agent()).build()?)
}

/// Fetches a URL as text.
///
/// # Errors
///
/// Returns [`crate::LauncherError`] if the request fails, returns an error
/// status, or the body cannot be decoded as text.
pub fn get_text(url: &str) -> Result<String> {
    Ok(client()?.get(url).send()?.error_for_status()?.text()?)
}

/// Fetches a URL and decodes the JSON body.
///
/// # Errors
///
/// Returns [`crate::LauncherError`] if the request fails, returns an error
/// status, or the body cannot be decoded as `T`.
pub fn get_json<T>(url: &str) -> Result<T>
where
    T: serde::de::DeserializeOwned,
{
    Ok(client()?.get(url).send()?.error_for_status()?.json()?)
}
