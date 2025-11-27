use gtk::{gdk, gio::Cancellable};
use webkit::{HardwareAccelerationPolicy, Settings, UserMessage, WebView, prelude::*};

use std::{cell::Cell, rc::Rc};

use crate::{bridge::Dispatcher, browser::BrowserProperties};

pub fn webview_new(debug: bool, theme_file: &str) -> WebView {
    let settings = Settings::builder()
        .allow_file_access_from_file_urls(true)
        .allow_universal_access_from_file_urls(true)
        .enable_page_cache(true)
        .enable_html5_local_storage(true)
        .enable_webgl(true)
        .hardware_acceleration_policy(HardwareAccelerationPolicy::Always)
        .enable_developer_extras(debug)
        .build();

    let webview = WebView::builder().settings(&settings).build();

    let rgba = gdk::RGBA::parse("#000000").unwrap();
    webview.set_background_color(&rgba);

    let uri = "file://".to_string() + theme_file;
    webview.load_uri(&uri);
    logger_debug!("Theme loaded");

    webview
}

pub fn user_message_received(
    webview: &WebView,
    message: &UserMessage,
    dispatcher: &Rc<Dispatcher>,
    loaded: &Rc<Cell<bool>>,
    win_props: &Rc<BrowserProperties>,
) -> bool {
    match message.name().as_deref() {
        Some("ready-to-show") => {
            if loaded.get() {
                return true;
            }

            let root = webview.root().expect("webview.root is None");
            let window = root
                .downcast_ref::<gtk::ApplicationWindow>()
                .expect("webview.root is not a ApplicationWindow");
            webview.grab_focus();
            window.present();

            loaded.set(true);
            logger_debug!("Lightdm webkit greeter started win: {}", window.id());
            true
        }
        Some("console") => {
            crate::webview::show_console_error_prompt(webview, message, dispatcher);
            true
        }
        Some(_) => {
            dispatcher.send(message, win_props);
            true
        }
        None => false,
    }
}

pub fn show_console_error_prompt(
    webview: &WebView,
    message: &UserMessage,
    dispatcher: &Rc<Dispatcher>,
) {
    let params = message.parameters().unwrap();

    let msg_var = params.child_value(1);
    let msg = msg_var.str().unwrap();
    let source_id_var = params.child_value(2);
    let source_id = source_id_var.str().unwrap();
    let line = u32::from_variant(&params.child_value(3)).unwrap();

    let dialog = gtk::AlertDialog::builder()
        .message("An error ocurred. Do you want to change to default theme? (litarvan)")
        .detail(format!(r##"{source_id} {line}: {msg}"##))
        .buttons(["_Cancel", "_Use default theme", "_Reload theme"])
        .build();

    let root = webview.root().expect("webview.root is not a browser");
    let window = root
        .downcast_ref::<gtk::ApplicationWindow>()
        .expect("webview.root is not a ApplicationWindow");

    dialog.choose(
        Some(window),
        Some(&Cancellable::new()),
        gtk::glib::clone!(
            #[strong]
            message,
            #[strong]
            dispatcher,
            move |res| {
                let response = res.unwrap();

                let stop_prompts = match response {
                    0 => false,
                    1 => {
                        dispatcher.change_theme(Some("gruvbox"));
                        true
                    }
                    2 => {
                        dispatcher.change_theme(None);
                        true
                    }
                    _ => false,
                };

                message.send_reply(&UserMessage::new(
                    "console-done",
                    Some(&stop_prompts.to_variant()),
                ));
            }
        ),
    );
}
