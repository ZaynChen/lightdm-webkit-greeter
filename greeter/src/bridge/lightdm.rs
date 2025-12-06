// SPDX-FileCopyrightText: 2025 ZaynChen
//
// SPDX-License-Identifier: GPL-3.0-or-later

use gtk::glib::{self, Variant, clone, variant::ToVariant};
use lightdm::prelude::*;

use ext::prelude::*;

use std::{cell::RefCell, rc::Rc};

use crate::browser::Browser;

pub(super) struct LightDM {
    context: jsc::Context,
    greeter: lightdm::Greeter,
    user_list: Option<lightdm::UserList>,
    shared_data_directory: String,
    brightness: RefCell<i32>,
}

impl LightDM {
    pub(super) fn new(context: jsc::Context, browsers: Rc<Vec<Browser>>) -> Self {
        let greeter = lightdm::Greeter::new();
        let user_list = lightdm::UserList::instance();

        greeter.connect_authentication_complete(clone!(
            #[weak]
            browsers,
            move |_| greeter::authentication_complete(&browsers)
        ));
        greeter.connect_autologin_timer_expired(clone!(
            #[weak]
            browsers,
            move |_| greeter::autologin_timer_expired(&browsers)
        ));
        greeter.connect_show_prompt(clone!(
            #[weak]
            context,
            #[weak]
            browsers,
            move |_, text, ty| greeter::show_prompt(&browsers, &context, text, ty)
        ));
        greeter.connect_show_message(clone!(
            #[weak]
            context,
            #[weak]
            browsers,
            move |_, text, ty| greeter::show_message(&browsers, &context, text, ty)
        ));

        if let Err(e) = greeter.connect_to_daemon_sync() {
            logger_error!("{}", e.message());
        }

        let shared_data_directory = match &user_list {
            Some(userlist) => match userlist.users().first() {
                Some(user) => {
                    match greeter.ensure_shared_data_dir_sync(
                        user.name().expect("Failed to get username").as_str(),
                    ) {
                        Ok(data_dir) => {
                            let s = data_dir.to_string();
                            let (substr, _) = s
                                .rsplit_once("/")
                                .unwrap_or_else(|| panic!("{} does not contain `/`", s));
                            substr.to_string()
                        }
                        Err(_) => "".to_string(),
                    }
                }
                None => "".to_string(),
            },
            None => "".to_string(),
        };

        logger_debug!("LightDM API connected");
        Self {
            context,
            greeter,
            user_list,
            shared_data_directory,
            brightness: RefCell::new(85),
        }
    }

    pub(super) fn shared_data_directory(&self) -> &str {
        &self.shared_data_directory
    }

    pub(super) fn handle(&self, name: &str, json_params: &str) -> Variant {
        let context = &self.context;
        let params = jsc::Value::from_json(context, json_params).to_vec();
        let ret = if params.is_empty() {
            match name {
                "authentication_user" => self.authentication_user(),
                "autologin_guest" => self.autologin_guest(),
                "autologin_timeout" => self.autologin_timeout(),
                "autologin_user" => self.autologin_user(),
                "brightness" => self.brightness(),
                "can_hibernate" => self.can_hibernate(),
                "can_restart" => self.can_restart(),
                "can_shutdown" => self.can_shutdown(),
                "can_suspend" => self.can_suspend(),
                "default_session" => self.default_session(),
                "has_guest_account" => self.has_guest_account(),
                "hide_users_hint" => self.hide_users_hint(),
                "hostname" => self.hostname(),
                "in_authentication" => self.in_authentication(),
                "is_authenticated" => self.is_authenticated(),
                "language" => self.language(),
                "languages" => self.languages(),
                "layout" => self.layout(),
                "layouts" => self.layouts(),
                "lock_hint" => self.lock_hint(),
                "remote_sessions" => self.remote_sessions(),
                "select_guest_hint" => self.select_guest_hint(),
                "select_user_hint" => self.select_user_hint(),
                "sessions" => self.sessions(),
                "shared_data_directory" => self.shared_data_directory_getter(),
                "show_manual_login_hint" => self.show_manual_login_hint(),
                "show_remote_login_hint" => self.show_remote_login_hint(),
                "users" => self.users(),
                "authenticate_as_guest" => self.authenticate_as_guest(),
                "cancel_authentication" => self.cancel_authentication(),
                "cancel_autologin" => self.cancel_autologin(),
                "hibernate" => self.hibernate(),
                "restart" => self.restart(),
                "shutdown" => self.shutdown(),
                "suspend" => self.suspend(),
                s => {
                    logger_warn!("{s} does not implemented");
                    jsc::Value::new_undefined(context)
                }
            }
        } else {
            match name {
                "brightness" => self.set_brightness(params[0].to_int32()),
                "layout" => self.set_layout(params[0].clone()),
                "authenticate" => self.authenticate(Some(&params[0].to_string())),
                "respond" => self.respond(&params[0].to_string()),
                "set_language" => self.set_language(&params[0].to_string()),
                "start_session" => self.start_session(Some(&params[0].to_string())),
                s => {
                    logger_warn!("{s} does not implemented");
                    jsc::Value::new_undefined(context)
                }
            }
        };

        if let Some(json) = ret.to_json(0) {
            json.to_variant()
        } else {
            "undefined".to_variant()
        }
    }

    fn authentication_user(&self) -> jsc::Value {
        let context = &self.context;
        if let Some(user) = self.greeter.authentication_user() {
            jsc::Value::new_string(context, Some(user.as_str()))
        } else {
            jsc::Value::new_null(context)
        }
    }

    fn autologin_guest(&self) -> jsc::Value {
        let value = self.greeter.is_autologin_guest_hint();
        jsc::Value::new_boolean(&self.context, value)
    }

    fn autologin_timeout(&self) -> jsc::Value {
        let value = self.greeter.autologin_timeout_hint();
        jsc::Value::new_number(&self.context, value as f64)
    }

    fn autologin_user(&self) -> jsc::Value {
        let context = &self.context;
        if let Some(value) = self.greeter.autologin_user_hint() {
            jsc::Value::new_string(context, Some(value.as_str()))
        } else {
            jsc::Value::new_null(context)
        }
    }

    fn can_hibernate(&self) -> jsc::Value {
        let value = lightdm::functions::can_hibernate();
        jsc::Value::new_boolean(&self.context, value)
    }

    fn can_restart(&self) -> jsc::Value {
        let value = lightdm::functions::can_restart();
        jsc::Value::new_boolean(&self.context, value)
    }

    fn can_shutdown(&self) -> jsc::Value {
        let value = lightdm::functions::can_shutdown();
        jsc::Value::new_boolean(&self.context, value)
    }

    fn can_suspend(&self) -> jsc::Value {
        let value = lightdm::functions::can_suspend();
        jsc::Value::new_boolean(&self.context, value)
    }

    fn brightness(&self) -> jsc::Value {
        jsc::Value::new_number(&self.context, *self.brightness.borrow() as f64)
    }

    fn set_brightness(&self, brightness: i32) -> jsc::Value {
        self.brightness.replace(brightness);
        jsc::Value::new_boolean(&self.context, true)
    }

    fn default_session(&self) -> jsc::Value {
        if let Some(session) = self.greeter.default_session_hint() {
            jsc::Value::new_string(&self.context, Some(session.as_str()))
        } else {
            jsc::Value::new_null(&self.context)
        }
    }

    fn has_guest_account(&self) -> jsc::Value {
        let value = self.greeter.has_guest_account_hint();
        jsc::Value::new_boolean(&self.context, value)
    }

    fn hide_users_hint(&self) -> jsc::Value {
        let value = self.greeter.hides_users_hint();
        jsc::Value::new_boolean(&self.context, value)
    }

    fn hostname(&self) -> jsc::Value {
        let context = &self.context;
        if let Some(value) = lightdm::functions::hostname() {
            jsc::Value::new_string(context, Some(value.as_str()))
        } else {
            jsc::Value::new_null(context)
        }
    }

    fn in_authentication(&self) -> jsc::Value {
        let value = self.greeter.is_in_authentication();
        jsc::Value::new_boolean(&self.context, value)
    }

    fn is_authenticated(&self) -> jsc::Value {
        let value = self.greeter.is_authenticated();
        jsc::Value::new_boolean(&self.context, value)
    }

    fn language(&self) -> jsc::Value {
        let context = &self.context;
        match lightdm::functions::language() {
            Some(language) => language.to_jscvalue(context),
            None => match lightdm::functions::languages().first() {
                Some(language) => language.to_jscvalue(context),
                None => jsc::Value::new_undefined(context),
            },
        }
    }

    fn languages(&self) -> jsc::Value {
        let context = &self.context;
        let languages: Vec<jsc::Value> = lightdm::functions::languages()
            .iter()
            .map(|language| language.to_jscvalue(context))
            .collect();
        jsc::Value::new_array_from_garray(context, &languages)
    }

    fn layout(&self) -> jsc::Value {
        let context = &self.context;
        match lightdm::functions::layout() {
            Some(layout) => layout.to_jscvalue(context),
            None => match lightdm::functions::layouts().first() {
                Some(layout) => layout.to_jscvalue(context),
                None => jsc::Value::new_undefined(context),
            },
        }
    }

    fn set_layout(&self, value: jsc::Value) -> jsc::Value {
        let context = &self.context;
        if !value.object_has_property("name")
            || !value.object_has_property("description")
            || !value.object_has_property("short_description")
        {
            context.throw("Invalid LightDMLayout");
        }

        let name = value.object_get_property("name").and_then(|s| {
            if s.is_string() {
                Some(s.to_string())
            } else {
                None
            }
        });

        let layout = lightdm::functions::layouts()
            .into_iter()
            .find(|l| name == l.name().map(|s| s.to_string()));
        lightdm::functions::set_layout(&layout.unwrap());
        jsc::Value::new_boolean(context, true)
    }

    fn layouts(&self) -> jsc::Value {
        let context = &self.context;
        let layouts: Vec<jsc::Value> = lightdm::functions::layouts()
            .iter()
            .map(|layout| layout.to_jscvalue(context))
            .collect();
        jsc::Value::new_array_from_garray(context, &layouts)
    }

    fn lock_hint(&self) -> jsc::Value {
        let value = self.greeter.is_lock_hint();
        jsc::Value::new_boolean(&self.context, value)
    }

    fn remote_sessions(&self) -> jsc::Value {
        let context = &self.context;
        let sessions: Vec<jsc::Value> = lightdm::functions::remote_sessions()
            .iter()
            .map(|session| session.to_jscvalue(context))
            .collect();
        jsc::Value::new_array_from_garray(context, &sessions)
    }

    fn select_guest_hint(&self) -> jsc::Value {
        let value = self.greeter.selects_guest_hint();
        jsc::Value::new_boolean(&self.context, value)
    }

    fn select_user_hint(&self) -> jsc::Value {
        let context = &self.context;
        match self.greeter.select_user_hint() {
            Some(value) => jsc::Value::new_string(context, Some(value.as_str())),
            None => jsc::Value::new_null(context),
        }
    }

    fn sessions(&self) -> jsc::Value {
        let context = &self.context;
        let sessions: Vec<jsc::Value> = lightdm::functions::sessions()
            .iter()
            .map(|session| session.to_jscvalue(context))
            .collect();
        jsc::Value::new_array_from_garray(context, &sessions)
    }

    fn shared_data_directory_getter(&self) -> jsc::Value {
        let context = &self.context;
        let dir = &self.shared_data_directory;
        if dir.is_empty() {
            jsc::Value::new_null(context)
        } else {
            jsc::Value::new_string(context, Some(dir))
        }
    }

    fn show_manual_login_hint(&self) -> jsc::Value {
        let value = self.greeter.shows_manual_login_hint();
        jsc::Value::new_boolean(&self.context, value)
    }

    fn show_remote_login_hint(&self) -> jsc::Value {
        let value = self.greeter.shows_remote_login_hint();
        jsc::Value::new_boolean(&self.context, value)
    }

    fn users(&self) -> jsc::Value {
        let context = &self.context;
        let users = match &self.user_list {
            Some(userlist) => userlist
                .users()
                .iter()
                .map(|user| user.to_jscvalue(context))
                .collect::<Vec<jsc::Value>>(),
            None => vec![],
        };
        jsc::Value::new_array_from_garray(context, &users)
    }

    fn authenticate(&self, username: Option<&str>) -> jsc::Value {
        let context = &self.context;
        if let Err(e) = self.greeter.authenticate(username) {
            logger_error!("{}", e.message());
            jsc::Value::new_boolean(context, false)
        } else {
            jsc::Value::new_boolean(context, true)
        }
    }

    fn authenticate_as_guest(&self) -> jsc::Value {
        let context = &self.context;
        if let Err(e) = self.greeter.authenticate_as_guest() {
            logger_error!("{}", e.message());
            jsc::Value::new_boolean(context, false)
        } else {
            jsc::Value::new_boolean(context, true)
        }
    }

    fn cancel_authentication(&self) -> jsc::Value {
        let context = &self.context;
        if let Err(e) = self.greeter.cancel_authentication() {
            logger_error!("{}", e.message());
            jsc::Value::new_boolean(context, false)
        } else {
            jsc::Value::new_boolean(context, true)
        }
    }

    fn cancel_autologin(&self) -> jsc::Value {
        self.greeter.cancel_autologin();
        jsc::Value::new_boolean(&self.context, true)
    }

    fn hibernate(&self) -> jsc::Value {
        let context = &self.context;
        if let Err(e) = lightdm::functions::hibernate() {
            logger_error!("{}", e.message());
            jsc::Value::new_boolean(context, false)
        } else {
            jsc::Value::new_boolean(context, true)
        }
    }

    fn respond(&self, response: &str) -> jsc::Value {
        let context = &self.context;
        if let Err(e) = self.greeter.respond(response) {
            logger_error!("{}", e.message());
            jsc::Value::new_boolean(context, false)
        } else {
            jsc::Value::new_boolean(context, true)
        }
    }

    fn restart(&self) -> jsc::Value {
        let context = &self.context;
        if let Err(e) = lightdm::functions::restart() {
            logger_error!("{}", e.message());
            jsc::Value::new_boolean(context, false)
        } else {
            jsc::Value::new_boolean(context, true)
        }
    }

    fn set_language(&self, language: &str) -> jsc::Value {
        let context = &self.context;
        if let Err(e) = self.greeter.set_language(language) {
            logger_error!("{}", e.message());
            jsc::Value::new_boolean(context, false)
        } else {
            jsc::Value::new_boolean(context, true)
        }
    }

    fn shutdown(&self) -> jsc::Value {
        let context = &self.context;
        if let Err(e) = lightdm::functions::shutdown() {
            logger_error!("{}", e.message());
            jsc::Value::new_boolean(context, false)
        } else {
            jsc::Value::new_boolean(context, true)
        }
    }

    fn suspend(&self) -> jsc::Value {
        let context = &self.context;
        if let Err(e) = lightdm::functions::suspend() {
            logger_error!("{}", e.message());
            jsc::Value::new_boolean(context, false)
        } else {
            jsc::Value::new_boolean(context, true)
        }
    }

    fn start_session(&self, session: Option<&str>) -> jsc::Value {
        let context = &self.context;
        if let Err(e) = self.greeter.start_session_sync(session) {
            logger_error!("{}", e.message());
            jsc::Value::new_boolean(context, false)
        } else {
            jsc::Value::new_boolean(context, true)
        }
    }
}

mod greeter {
    use gtk::{
        gio::Cancellable,
        glib::{translate::IntoGlib, variant::ToVariant},
    };
    use webkit::{UserMessage, prelude::WebViewExt};

    use super::Browser;

    pub(super) fn authentication_complete(browsers: &[Browser]) {
        browsers.iter().map(|b| b.webview()).for_each(|webview| {
            let parameters = ["authentication_complete", "[]"].to_variant();
            let message = UserMessage::new("lightdm", Some(&parameters));
            webview.send_message_to_page(&message, Cancellable::NONE, |_| {});
        });
    }

    pub(super) fn autologin_timer_expired(browsers: &[Browser]) {
        browsers.iter().map(|b| b.webview()).for_each(|webview| {
            let parameters = ["autologin_timer_expired", "[]"].to_variant();
            let message = UserMessage::new("lightdm", Some(&parameters));
            webview.send_message_to_page(&message, Cancellable::NONE, |_| {});
        });
    }

    pub(super) fn show_prompt(
        browsers: &[Browser],
        context: &jsc::Context,
        text: &str,
        ty: lightdm::PromptType,
    ) {
        browsers.iter().map(|b| b.webview()).for_each(|webview| {
            let param = jsc::Value::new_array_from_garray(
                context,
                &[
                    jsc::Value::new_string(context, Some(text)),
                    jsc::Value::new_number(context, ty.into_glib() as f64),
                ],
            )
            .to_json(0)
            .expect("param parse to json failed");
            let parameters = ["show_prompt", &param].to_variant();
            let message = UserMessage::new("lightdm", Some(&parameters));
            webview.send_message_to_page(&message, Cancellable::NONE, |_| {});
        });
    }

    pub(super) fn show_message(
        browsers: &[Browser],
        context: &jsc::Context,
        text: &str,
        ty: lightdm::MessageType,
    ) {
        browsers.iter().map(|b| b.webview()).for_each(|webview| {
            let param = jsc::Value::new_array_from_garray(
                context,
                &[
                    jsc::Value::new_string(context, Some(text)),
                    jsc::Value::new_number(context, ty.into_glib() as f64),
                ],
            )
            .to_json(0)
            .expect("param parse to json failed");

            let parameters = ["show_message", &param].to_variant();
            let message = UserMessage::new("lightdm", Some(&parameters));
            webview.send_message_to_page(&message, Cancellable::NONE, |_| {});
        });
    }
}
