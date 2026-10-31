// SPDX-FileCopyrightText: 2025 ZaynChen <zaynchen@qq.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

mod dbus;

mod accounts;
mod language;
mod layout;
mod power;
mod session;

use accounts::UserManager;
use language::LanguageManager;
use layout::LayoutManager;
use power::PowerManager;
use session::SessionManager;

use webkit::{
    WebView,
    glib::{self, Variant, clone, variant::ToVariant},
};

pub struct LightDMGreeter {
    greeter: lightdm_client::Greeter,
}

impl LightDMGreeter {
    pub fn new(webview: &WebView) -> Self {
        let greeter = lightdm_client::Greeter::new();

        greeter.connect_authentication_complete(clone!(
            #[strong]
            webview,
            move |_| signals::authentication_complete(&webview)
        ));
        greeter.connect_show_prompt(clone!(
            #[strong]
            webview,
            move |_, text, ty| signals::show_prompt(&webview, text, ty.try_into().unwrap())
        ));
        greeter.connect_show_message(clone!(
            #[strong]
            webview,
            move |_, text, ty| signals::show_message(&webview, text, ty.try_into().unwrap())
        ));

        if let Err(e) = glib::MainContext::default().block_on(greeter.connect_to_daemon_future()) {
            log::error!("{e}");
        }

        log::debug!("LightDM API connected");
        Self { greeter }
    }

    pub(super) fn handle(&self, method: &str, json_args: &str) -> Variant {
        let val: serde_json::Value = serde_json::from_str(json_args).unwrap();
        let args = val.as_array().expect("json_args should be array");
        let json_result = if args.is_empty() {
            match method {
                "can_hibernate" => self.can_hibernate(),
                "can_restart" => self.can_reboot(),
                "can_shutdown" => self.can_shutdown(),
                "can_suspend" => self.can_suspend(),
                "hibernate" => self.hibernate(),
                "restart" => self.reboot(),
                "shutdown" => self.shutdown(),
                "suspend" => self.suspend(),
                "language" => self.language(),
                "languages" => self.languages(),
                "sessions" => self.sessions(),
                "layout" => self.layout(),
                "layouts" => self.layouts(),
                "users" => self.users(),
                //----
                "authentication_user" => self.authentication_user(),
                "in_authentication" => self.in_authentication(),
                "is_authenticated" => self.is_authenticated(),
                "cancel_authentication" => self.cancel_authentication(),
                // ----
                "autologin_guest" => self.autologin_guest_hint(),
                "autologin_timeout" => self.autologin_timeout_hint(),
                "autologin_user" => self.autologin_user_hint(),
                "default_session" => self.default_session_hint(),
                "has_guest_account" => self.has_guest_account_hint(),
                "hide_users_hint" => self.hide_users_hint(),
                "lock_hint" => self.lock_hint(),
                "select_guest_hint" => self.select_guest_hint(),
                "select_user_hint" => self.select_user_hint(),
                "show_manual_login_hint" => self.show_manual_login_hint(),
                "show_remote_login_hint" => self.show_remote_login_hint(),
                "authenticate_as_guest" => self.authenticate_as_guest(),
                "cancel_autologin" => self.cancel_autologin(),
                s => {
                    log::warn!("{s} does not implemented");
                    "undefined".to_string()
                }
            }
        } else {
            match method {
                "layout" => self.set_layout(args[0].as_str().unwrap()),
                // ----
                "authenticate" => self.authenticate(args[0].as_str().map(str::to_string)),
                "respond" => self.respond(args[0].as_str().unwrap().to_string()),
                "start_session" => self.start_session_sync(args[0].as_str().map(str::to_string)),
                s => {
                    log::warn!("{s} does not implemented");
                    "undefined".to_string()
                }
            }
        };
        json_result.to_variant()
    }

    fn can_hibernate(&self) -> String {
        PowerManager::can_hibernate().to_string()
    }

    fn can_reboot(&self) -> String {
        PowerManager::can_reboot().to_string()
    }

    fn can_shutdown(&self) -> String {
        PowerManager::can_power_off().to_string()
    }

    fn can_suspend(&self) -> String {
        PowerManager::can_suspend().to_string()
    }

    fn hibernate(&self) -> String {
        PowerManager::hibernate()
            .inspect_err(|e| log::error!("{e}"))
            .is_ok()
            .to_string()
    }

    fn reboot(&self) -> String {
        PowerManager::reboot()
            .inspect_err(|e| log::error!("{e}"))
            .is_ok()
            .to_string()
    }

    fn shutdown(&self) -> String {
        PowerManager::power_off()
            .inspect_err(|e| log::error!("{e}"))
            .is_ok()
            .to_string()
    }

    fn suspend(&self) -> String {
        PowerManager::suspend()
            .inspect_err(|e| log::error!("{e}"))
            .is_ok()
            .to_string()
    }

    fn languages(&self) -> String {
        serde_json::to_string(LanguageManager::languages()).unwrap()
    }

    fn language(&self) -> String {
        match LanguageManager::current() {
            Some(language) => serde_json::to_string(&language).unwrap(),
            None => match LanguageManager::languages().first() {
                Some(language) => serde_json::to_string(&language).unwrap(),
                None => "null".to_string(),
            },
        }
    }

    fn layout(&self) -> String {
        serde_json::to_string(LayoutManager::instance().layout()).unwrap()
    }

    fn set_layout(&self, layout: &str) -> String {
        LayoutManager::instance().set_layout(layout).to_string()
    }

    fn layouts(&self) -> String {
        serde_json::to_string(LayoutManager::instance().layouts()).unwrap()
    }

    fn sessions(&self) -> String {
        serde_json::to_string(&SessionManager::sessions()).unwrap()
    }

    fn users(&self) -> String {
        serde_json::to_string(UserManager::instance().list_users()).unwrap()
    }

    fn authentication_user(&self) -> String {
        self.greeter
            .authentication_user()
            .as_deref()
            .unwrap_or("null")
            .to_string()
    }

    fn in_authentication(&self) -> String {
        self.greeter.in_authentication().to_string()
    }

    fn is_authenticated(&self) -> String {
        self.greeter.is_authenticated().to_string()
    }

    fn authenticate(&self, username: Option<String>) -> String {
        self.greeter
            .authenticate(username)
            .inspect_err(|e| log::error!("{e}"))
            .is_ok()
            .to_string()
    }

    fn cancel_authentication(&self) -> String {
        self.greeter
            .cancel_authentication()
            .inspect_err(|e| log::error!("{e}"))
            .is_ok()
            .to_string()
    }

    fn respond(&self, response: String) -> String {
        self.greeter
            .respond(response)
            .inspect_err(|e| log::error!("{e}"))
            .is_ok()
            .to_string()
    }

    fn start_session_sync(&self, session: Option<String>) -> String {
        glib::MainContext::default()
            .block_on(self.greeter.start_session_future(session))
            .inspect_err(|e| log::error!("{e}"))
            .is_ok()
            .to_string()
    }

    fn autologin_guest_hint(&self) -> String {
        self.greeter.autologin_guest_hint().to_string()
    }

    fn autologin_timeout_hint(&self) -> String {
        self.greeter.autologin_timeout_hint().to_string()
    }

    fn autologin_user_hint(&self) -> String {
        self.greeter
            .autologin_user_hint()
            .unwrap_or("null".to_string())
    }

    fn default_session_hint(&self) -> String {
        self.greeter
            .default_session_hint()
            .unwrap_or("null".to_string())
    }

    fn has_guest_account_hint(&self) -> String {
        self.greeter.has_guest_account_hint().to_string()
    }

    fn hide_users_hint(&self) -> String {
        self.greeter.hide_users_hint().to_string()
    }

    fn lock_hint(&self) -> String {
        self.greeter.lock_hint().to_string()
    }

    fn select_guest_hint(&self) -> String {
        self.greeter.select_guest_hint().to_string()
    }

    fn select_user_hint(&self) -> String {
        self.greeter
            .select_user_hint()
            .unwrap_or("null".to_string())
    }

    fn show_manual_login_hint(&self) -> String {
        self.greeter.show_manual_login_hint().to_string()
    }

    fn show_remote_login_hint(&self) -> String {
        self.greeter.show_remote_login_hint().to_string()
    }

    fn authenticate_as_guest(&self) -> String {
        self.greeter
            .authenticate_as_guest()
            .inspect_err(|e| log::error!("{e}"))
            .is_ok()
            .to_string()
    }

    fn cancel_autologin(&self) -> String {
        self.greeter.cancel_autologin();
        true.to_string()
    }
}

mod signals {
    use webkit::{
        UserMessage, WebView,
        gio::Cancellable,
        glib::{self, variant::ToVariant},
        prelude::WebViewExt,
    };

    #[derive(Debug, Clone, Copy)]
    pub enum PromptType {
        Visible = 0,
        Secret = 1,
    }

    impl TryFrom<u32> for PromptType {
        type Error = glib::Error;

        fn try_from(value: u32) -> Result<Self, Self::Error> {
            match value {
                0 => Ok(Self::Visible),
                1 => Ok(Self::Secret),
                _ => Err(glib::Error::new(
                    glib::ConvertError::Failed,
                    "invalid prompt type",
                )),
            }
        }
    }

    #[derive(Debug, Clone, Copy)]
    pub enum MessageType {
        Info = 0,
        Error = 1,
    }

    impl TryFrom<u32> for MessageType {
        type Error = glib::Error;

        fn try_from(value: u32) -> Result<Self, <Self as TryFrom<u32>>::Error> {
            match value {
                0 => Ok(Self::Info),
                1 => Ok(Self::Error),
                _ => Err(glib::Error::new(
                    glib::ConvertError::Failed,
                    "invalid message type",
                )),
            }
        }
    }

    pub(super) fn show_prompt(webview: &WebView, text: &str, ty: PromptType) {
        let type_ = match ty {
            PromptType::Visible => "Visible",
            PromptType::Secret => "Secret",
        };
        let parameters = ["show_prompt", &format!("[\"{text}\", \"{type_}\"]")].to_variant();
        let message = UserMessage::new("greeter", Some(&parameters));
        webview.send_message_to_page(&message, Cancellable::NONE, |_| {});
    }

    pub(super) fn show_message(webview: &WebView, text: &str, ty: MessageType) {
        let type_ = match ty {
            MessageType::Info => "Info",
            MessageType::Error => "Error",
        };
        let parameters = ["show_message", &format!("[\"{text}\", \"{type_}\"]")].to_variant();
        let message = UserMessage::new("greeter", Some(&parameters));
        webview.send_message_to_page(&message, Cancellable::NONE, |_| {});
    }

    pub(super) fn authentication_complete(webview: &WebView) {
        let parameters = ["authentication_complete", "[]"].to_variant();
        let message = UserMessage::new("greeter", Some(&parameters));
        webview.send_message_to_page(&message, Cancellable::NONE, |_| {});
    }
}
