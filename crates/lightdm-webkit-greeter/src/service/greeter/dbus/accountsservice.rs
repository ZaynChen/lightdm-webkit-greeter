// SPDX-FileCopyrightText: 2025 ZaynChen <zaynchen@qq.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later AND LGPL-3.0-or-later

use std::sync::OnceLock;

use super::system_conn;

pub struct AccountsService;
impl AccountsService {
    pub fn accounts_proxy() -> &'static Option<AccountsProxy<'static>> {
        static ACCOUNTS_PROXY: OnceLock<Option<AccountsProxy>> = OnceLock::new();
        ACCOUNTS_PROXY.get_or_init(|| zbus::block_on(AccountsProxy::new(system_conn())).ok())
    }

    pub fn user_proxy(o: zbus::OwnedObjectPath) -> UserProxy<'static> {
        zbus::block_on(UserProxy::builder(system_conn()).path(o).build()).unwrap()
    }
}

#[zbus::proxy(
    interface = "org.freedesktop.Accounts",
    default_service = "org.freedesktop.Accounts",
    default_path = "/org/freedesktop/Accounts"
)]
pub trait Accounts {
    /// FindUserById method
    fn find_user_by_id(&self, id: i64) -> zbus::Result<zbus::OwnedObjectPath>;

    /// FindUserByName method
    fn find_user_by_name(&self, name: &str) -> zbus::Result<zbus::OwnedObjectPath>;

    /// ListCachedUsers method
    fn list_cached_users(&self) -> zbus::Result<Vec<zbus::OwnedObjectPath>>;
}

#[zbus::proxy(
    interface = "org.freedesktop.Accounts.User",
    default_service = "org.freedesktop.Accounts"
)]
pub trait User {
    /// SetHomeDirectory method
    fn set_home_directory(&self, homedir: &str) -> zbus::Result<()>;

    /// SetIconFile method
    fn set_icon_file(&self, filename: &str) -> zbus::Result<()>;

    /// SetLanguage method
    fn set_language(&self, language: &str) -> zbus::Result<()>;

    /// SetLanguages method
    fn set_languages(&self, languages: &[&str]) -> zbus::Result<()>;

    /// SetLocked method
    fn set_locked(&self, locked: bool) -> zbus::Result<()>;

    /// SetRealName method
    fn set_real_name(&self, name: &str) -> zbus::Result<()>;

    /// SetSession method
    fn set_session(&self, session: &str) -> zbus::Result<()>;

    /// SetUserName method
    fn set_user_name(&self, name: &str) -> zbus::Result<()>;

    /// HomeDirectory property
    #[zbus(property)]
    fn home_directory(&self) -> zbus::Result<String>;

    /// IconFile property
    #[zbus(property)]
    fn icon_file(&self) -> zbus::Result<String>;

    /// Language property
    #[zbus(property)]
    fn language(&self) -> zbus::Result<String>;

    /// Languages property
    #[zbus(property)]
    fn languages(&self) -> zbus::Result<Vec<String>>;

    /// Locked property
    #[zbus(property)]
    fn locked(&self) -> zbus::Result<bool>;

    /// RealName property
    #[zbus(property)]
    fn real_name(&self) -> zbus::Result<String>;

    /// Session property
    #[zbus(property)]
    fn session(&self) -> zbus::Result<String>;

    /// Uid property
    #[zbus(property)]
    fn uid(&self) -> zbus::Result<u64>;

    /// UserName property
    #[zbus(property)]
    fn user_name(&self) -> zbus::Result<String>;
}
