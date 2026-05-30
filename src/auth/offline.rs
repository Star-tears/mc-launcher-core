//! Offline account compatibility helpers.

use crate::{account::Account, types::MinecraftOptions};

/// Creates an [`Account::Offline`] value for local launches.
pub fn get_offline_account(user_name: &str) -> Account {
    Account::offline(user_name)
}

/// Creates legacy [`MinecraftOptions`] with offline account fields populated.
///
/// Prefer [`Account::offline`] for new code.
#[deprecated(note = "use Account::offline")]
pub fn get_offline_options(user_name: &str) -> MinecraftOptions {
    let account = Account::offline(user_name);
    MinecraftOptions::new(
        account.username().to_string(),
        account.uuid().to_string(),
        account.access_token().to_string(),
    )
}
