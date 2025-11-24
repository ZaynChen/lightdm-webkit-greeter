mod greeter_comm;
mod greeter_config;
mod lightdm;
mod theme_utils;

pub use dispatcher::Dispatcher;

mod dispatcher {
    use gtk::glib::VariantTy;
    use webkit::UserMessage;

    use std::rc::Rc;

    use crate::{
        browser::{Browser, BrowserProperties},
        settings::Settings,
    };

    use super::{
        greeter_comm::GreeterComm, greeter_config::GreeterConfig, lightdm::LightDM,
        theme_utils::ThemeUtils,
    };

    pub struct Dispatcher {
        greeter_config: GreeterConfig,
        greeter_comm: GreeterComm,
        lightdm: LightDM,
        theme_utils: ThemeUtils,
    }

    impl Dispatcher {
        pub fn new(config: Settings, context: jsc::Context, browsers: Rc<Vec<Browser>>) -> Self {
            let theme = config.theme().to_string();
            let lightdm = LightDM::new(context.clone(), browsers.clone());
            let allowed_dirs = [
                config.themes_dir().unwrap().to_string(),
                config.branding_background_images_dir().to_string(),
                lightdm.shared_data_directory().to_string(),
            ];
            let theme_utils = ThemeUtils::new(context.clone(), &allowed_dirs, &theme);
            let greeter_config = GreeterConfig::new(context.clone(), config);
            let greeter_comm = GreeterComm::new(context, browsers);
            Self {
                greeter_config,
                greeter_comm,
                lightdm,
                theme_utils,
            }
        }

        pub fn send(&self, message: &UserMessage, win_props: &BrowserProperties) {
            let reply = match parse(message) {
                Message::GreeterConfig((method, _)) => {
                    // logger_warn!("greeter_config.{method}({json_params})");
                    let reply = self.greeter_config.handle(&method);
                    UserMessage::new("reply", Some(&reply))
                }
                Message::GreeterComm((method, json_params)) => {
                    // logger_warn!("greeter_comm.{method}({json_params})");
                    let reply = self.greeter_comm.handle(&method, &json_params, win_props);
                    UserMessage::new("reply", Some(&reply))
                }
                Message::LigthDM((method, json_params)) => {
                    // logger_warn!("lightdm.{method}({json_params})");
                    let reply = self.lightdm.handle(&method, &json_params);
                    UserMessage::new("reply", Some(&reply))
                }
                Message::ThemeUtils((method, json_params)) => {
                    // logger_warn!("theme_utils.{method}({json_params})");
                    let reply = self.theme_utils.handle(&method, &json_params);
                    UserMessage::new("reply", Some(&reply))
                }
                _ => {
                    logger_warn!("{:?}", message);
                    UserMessage::new("", None)
                }
            };
            // logger_warn!("{:?}", reply.parameters());
            message.send_reply(&reply);
        }
    }

    enum Message {
        GreeterConfig((String, String)),
        LigthDM((String, String)),
        GreeterComm((String, String)),
        ThemeUtils((String, String)),
        Unknown,
    }

    fn parse(message: &UserMessage) -> Message {
        let msg_param = message.parameters();
        if msg_param.is_none() {
            return Message::Unknown;
        }

        let msg_param = msg_param.unwrap();
        if msg_param.is_type(VariantTy::ARRAY) {
            let p_len = msg_param.n_children();
            if p_len == 0 || p_len > 2 {
                return Message::Unknown;
            }
        } else {
            return Message::Unknown;
        }

        let method_var = msg_param.child_value(0);
        let params_var = msg_param.child_value(1);

        let method = method_var.str().unwrap().to_string();
        let json_params = params_var.str().unwrap().to_string();

        if method.is_empty() {
            return Message::Unknown;
        }

        match message.name().as_deref() {
            Some("lightdm") => Message::LigthDM((method, json_params)),
            Some("greeter_config") => Message::GreeterConfig((method, json_params)),
            Some("greeter_comm") => Message::GreeterComm((method, json_params)),
            Some("theme_utils") => Message::ThemeUtils((method, json_params)),
            _ => Message::Unknown,
        }
    }
}
