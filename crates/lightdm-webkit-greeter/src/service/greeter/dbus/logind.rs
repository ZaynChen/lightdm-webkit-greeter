// SPDX-FileCopyrightText: 2025 ZaynChen <zaynchen@qq.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later AND LGPL-3.0-or-later

use std::sync::OnceLock;

use super::system_conn;

pub struct LogindManager;
impl LogindManager {
    pub fn proxy() -> &'static ManagerProxy<'static> {
        static MANAGER: OnceLock<ManagerProxy> = OnceLock::new();
        MANAGER.get_or_init(|| zbus::block_on(ManagerProxy::new(system_conn())).unwrap())
    }
}

#[zbus::proxy(
    interface = "org.freedesktop.login1.Manager",
    default_service = "org.freedesktop.login1",
    default_path = "/org/freedesktop/login1"
)]
pub trait Manager {
    /// CanHibernate method
    fn can_hibernate(&self) -> zbus::Result<String>;

    /// CanPowerOff method
    fn can_power_off(&self) -> zbus::Result<String>;

    /// CanReboot method
    fn can_reboot(&self) -> zbus::Result<String>;

    /// CanSuspend method
    fn can_suspend(&self) -> zbus::Result<String>;

    /// Hibernate method
    fn hibernate(&self, interactive: bool) -> zbus::Result<()>;

    /// PowerOff method
    fn power_off(&self, interactive: bool) -> zbus::Result<()>;

    /// Reboot method
    fn reboot(&self, interactive: bool) -> zbus::Result<()>;

    /// Suspend method
    fn suspend(&self, interactive: bool) -> zbus::Result<()>;
}
