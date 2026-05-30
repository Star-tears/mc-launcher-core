//! Account authentication helpers.
//!
//! [`offline`] creates local testing accounts. [`microsoft_account`] contains
//! the Microsoft OAuth, Xbox Live, XSTS, and Minecraft profile calls needed to
//! create an authenticated [`crate::account::Account::Microsoft`] value.

pub mod microsoft_account;
pub mod offline;
