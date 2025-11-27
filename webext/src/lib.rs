mod extension;

use gtk::{
    gio::Cancellable,
    glib::{
        self, MainContext, clone,
        ffi::GVariant,
        translate::*,
        variant::{FromVariant, ToVariant},
    },
};
use wwpe::ffi::WebKitWebProcessExtension;

use std::cell::RefCell;

fn web_page_send_console_message_to_view(
    page: &wwpe::WebPage,
    text: &str,
    source_id: &str,
    line: u32,
    stop_prompts: &RefCell<bool>,
) {
    let params = ("ERROR", text, source_id, line).to_variant();
    let message = wwpe::UserMessage::new("console", Some(&params));
    if let Ok(reply) = MainContext::default().block_on(page.send_message_to_view_future(&message)) {
        stop_prompts.replace(
            bool::from_variant(&reply.parameters().unwrap()).expect("reply is not boolean"),
        );
    }
}

fn web_page_console_message_sent(
    page: &wwpe::WebPage,
    message: &wwpe::ConsoleMessage,
    detect_theme_errors: bool,
    stop_prompts: &RefCell<bool>,
) {
    let message = &mut message.clone();
    let text = message.text().unwrap().to_string();
    let source_id = message.source_id().unwrap().to_string();
    let line = message.line();

    let timestamp = glib::DateTime::now_local()
        .unwrap()
        .format("%Y-%m-%d %H:%M:%S")
        .unwrap();
    match message.level() {
        wwpe::ConsoleMessageLevel::Error => {
            eprintln!("{timestamp} [ ERROR ] {source_id} {line}: {text}");
            if !*stop_prompts.borrow() && detect_theme_errors {
                web_page_send_console_message_to_view(page, &text, &source_id, line, stop_prompts);
            }
        }
        wwpe::ConsoleMessageLevel::Warning => {
            eprintln!("{timestamp} [ WARNING ] {source_id} {line}: {text}");
        }
        _ => {}
    }
}

fn web_page_created(page: &wwpe::WebPage, secure_mode: bool, detect_theme_errors: bool) {
    let stop_prompts = RefCell::new(false);
    page.connect_document_loaded(clone!(
        #[strong]
        stop_prompts,
        move |page| {
            stop_prompts.replace(false);
            let message = wwpe::UserMessage::new("ready-to-show", None);
            page.send_message_to_view(&message, Cancellable::NONE, |_| {});
        }
    ));

    page.connect_console_message_sent(clone!(
        #[strong]
        stop_prompts,
        move |page, message| {
            web_page_console_message_sent(page, message, detect_theme_errors, &stop_prompts)
        }
    ));

    if secure_mode {
        page.connect_send_request(|_, request, _| {
            let uri = request.uri().unwrap();
            let scheme = glib::uri_parse_scheme(&uri);

            !matches!(
                scheme.as_deref(),
                Some("file") | Some("data") | Some("web-greeter")
            )
        });
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn webkit_web_process_extension_initialize_with_user_data(
    extension: *mut WebKitWebProcessExtension,
    user_data: *const GVariant,
) {
    let user_data: glib::Variant = unsafe { from_glib_none(user_data) };
    let secure_mode =
        bool::from_variant(&user_data.child_value(0)).expect("secure_mode is not a bool");
    let detect_theme_errors =
        bool::from_variant(&user_data.child_value(1)).expect("detect_theme_errors is not a bool");

    let extention: wwpe::WebProcessExtension = unsafe { from_glib_none(extension) };
    extention.connect_page_created(move |_, page| {
        web_page_created(page, secure_mode, detect_theme_errors)
    });

    let lightdm_api_script = String::from_variant(&user_data.child_value(2))
        .expect("lightdm_api_script is not a String");
    crate::extension::web_page_initialize(lightdm_api_script);
}
