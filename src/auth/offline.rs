use crate::{account::Account, types::MinecraftOptions};

pub fn get_offline_account(user_name: &str) -> Account {
    Account::offline(user_name)
}

#[deprecated(note = "use Account::offline")]
pub fn get_offline_options(user_name: &str) -> MinecraftOptions {
    let account = Account::offline(user_name);
    MinecraftOptions::new(
        account.username().to_string(),
        account.uuid().to_string(),
        account.access_token().to_string(),
    )
}
