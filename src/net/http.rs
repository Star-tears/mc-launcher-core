use reqwest::blocking::Client;

use crate::Result;

pub fn user_agent() -> String {
    format!("mc-launcher-core/{}", env!("CARGO_PKG_VERSION"))
}

pub fn client() -> Result<Client> {
    Ok(Client::builder().user_agent(user_agent()).build()?)
}

pub fn get_text(url: &str) -> Result<String> {
    Ok(client()?.get(url).send()?.error_for_status()?.text()?)
}

pub fn get_json<T>(url: &str) -> Result<T>
where
    T: serde::de::DeserializeOwned,
{
    Ok(client()?.get(url).send()?.error_for_status()?.json()?)
}
