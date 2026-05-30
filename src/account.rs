//! Account identities used when constructing launch arguments.
//!
//! The launcher command builder only needs the fields that Minecraft expects in
//! the version argument templates: player name, UUID, and access token. Real
//! Microsoft authentication is handled in [`crate::auth::microsoft_account`],
//! while offline launches can use [`Account::offline`].

use uuid::Uuid;

/// A Minecraft account identity used by launch argument substitution.
///
/// Offline accounts are convenient for local testing and single-player
/// development. Microsoft accounts should be created from the authenticated
/// profile and access token returned by the Microsoft login flow.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Account {
    /// An offline account with a generated UUID and no access token.
    Offline {
        /// Display name passed to the game as `${auth_player_name}`.
        username: String,
        /// UUID passed to the game as `${auth_uuid}`.
        uuid: String,
    },
    /// A Microsoft-authenticated account.
    Microsoft {
        /// Display name from the Minecraft profile endpoint.
        username: String,
        /// UUID from the Minecraft profile endpoint.
        uuid: String,
        /// Minecraft access token used for authenticated services.
        access_token: String,
    },
}

impl Account {
    /// Creates an offline account with a random UUID.
    ///
    /// Offline accounts return an empty string from [`Account::access_token`].
    ///
    /// # Examples
    ///
    /// ```
    /// use mc_launcher_core::account::Account;
    ///
    /// let account = Account::offline("Steve");
    /// assert_eq!(account.username(), "Steve");
    /// assert_eq!(account.access_token(), "");
    /// ```
    pub fn offline(username: impl Into<String>) -> Self {
        Self::Offline {
            username: username.into(),
            uuid: Uuid::new_v4().to_string(),
        }
    }

    /// Returns the account display name used by Minecraft launch arguments.
    pub fn username(&self) -> &str {
        match self {
            Self::Offline { username, .. } | Self::Microsoft { username, .. } => username,
        }
    }

    /// Returns the account UUID used by Minecraft launch arguments.
    pub fn uuid(&self) -> &str {
        match self {
            Self::Offline { uuid, .. } | Self::Microsoft { uuid, .. } => uuid,
        }
    }

    /// Returns the Minecraft access token, or an empty string for offline accounts.
    pub fn access_token(&self) -> &str {
        match self {
            Self::Offline { .. } => "",
            Self::Microsoft { access_token, .. } => access_token,
        }
    }
}
