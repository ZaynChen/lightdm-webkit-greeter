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
    dispatcher: &Dispatcher,
    loaded: &Rc<Cell<bool>>,
    win_props: &Rc<BrowserProperties>,
) -> bool {
    match message.name().as_deref() {
        Some("ready-to-show") => {
            let root = webview.root().expect("webview.root is not browser");
            match root.downcast::<gtk::ApplicationWindow>() {
                Ok(window) => {
                    if loaded.get() {
                        return true;
                    }
                    webview.grab_focus();
                    window.present();
                    loaded.set(true);
                    logger_debug!("Sea greeter started win: {}", window.id());
                    true
                }
                Err(_) => {
                    logger_error!("webview.root is not a browser");
                    false
                }
            }
        }
        Some("console") => {
            crate::webview::show_console_error_prompt(webview, message);
            true
        }
        Some(_) => {
            dispatcher.send(message, win_props);
            true
        }
        None => false,
    }
}

pub fn show_console_error_prompt(_webview: &WebView, _message: &UserMessage) {
    let dialog = gtk::AlertDialog::builder()
        .message("An error ocurred")
        .buttons(["_Cancel", "_Use default theme", "_Reload theme"])
        .build();
    let win = gtk::Window::builder().build();
    dialog.choose(Some(&win), Some(&Cancellable::new()), |_e| {});
    win.present();
}
