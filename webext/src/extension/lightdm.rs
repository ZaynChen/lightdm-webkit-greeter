use ext::prelude::*;

use super::{JscValueDefault, initialize_class_methods, initialize_class_properties};

pub fn lightdm_initialize(page: &wwpe::WebPage, context: &jsc::Context, signals: Vec<jsc::Value>) {
    let global_object = context.global_object().unwrap();

    let lightdm_class = context.register_class("__LightDMGreeter", None).unwrap();
    let ldm_constructor = lightdm_class
        .add_constructor_variadic(None, |_| None)
        .unwrap();
    let properties_with_getter = vec![
        ("authentication_user".to_string(), JscValueDefault::Null),
        ("autologin_guest".to_string(), JscValueDefault::Boolean),
        (
            "autologin_timeout".to_string(),
            JscValueDefault::Number(0f64),
        ),
        ("autologin_user".to_string(), JscValueDefault::Null),
        ("can_hibernate".to_string(), JscValueDefault::Boolean),
        ("can_restart".to_string(), JscValueDefault::Boolean),
        ("can_shutdown".to_string(), JscValueDefault::Boolean),
        ("can_suspend".to_string(), JscValueDefault::Boolean),
        ("default_session".to_string(), JscValueDefault::Null),
        ("has_guest_account".to_string(), JscValueDefault::Boolean),
        ("hide_users_hint".to_string(), JscValueDefault::Boolean),
        ("hostname".to_string(), JscValueDefault::Null),
        ("in_authentication".to_string(), JscValueDefault::Boolean),
        ("is_authenticated".to_string(), JscValueDefault::Boolean),
        ("language".to_string(), JscValueDefault::Null),
        ("languages".to_string(), JscValueDefault::Array),
        ("layouts".to_string(), JscValueDefault::Array),
        ("lock_hint".to_string(), JscValueDefault::Boolean),
        ("remote_sessions".to_string(), JscValueDefault::Array),
        ("select_guest_hint".to_string(), JscValueDefault::Boolean),
        ("select_user_hint".to_string(), JscValueDefault::Null),
        ("sessions".to_string(), JscValueDefault::Array),
        ("shared_data_directory".to_string(), JscValueDefault::Null),
        (
            "show_manual_login_hint".to_string(),
            JscValueDefault::Boolean,
        ),
        (
            "show_remote_login_hint".to_string(),
            JscValueDefault::Boolean,
        ),
        ("users".to_string(), JscValueDefault::Array),
    ];
    let properties_with_accessor = vec![
        ("brightness".to_string(), JscValueDefault::Number(-1f64)),
        ("layout".to_string(), JscValueDefault::Null),
    ];
    let methods = vec![
        ("authenticate".to_string(), JscValueDefault::Boolean),
        (
            "authenticate_as_guest".to_string(),
            JscValueDefault::Boolean,
        ),
        (
            "cancel_authentication".to_string(),
            JscValueDefault::Boolean,
        ),
        ("cancel_autologin".to_string(), JscValueDefault::Boolean),
        ("hibernate".to_string(), JscValueDefault::Boolean),
        ("respond".to_string(), JscValueDefault::Boolean),
        ("restart".to_string(), JscValueDefault::Boolean),
        ("set_language".to_string(), JscValueDefault::Boolean),
        ("shutdown".to_string(), JscValueDefault::Boolean),
        ("start_session".to_string(), JscValueDefault::Boolean),
        ("suspend".to_string(), JscValueDefault::Boolean),
    ];
    initialize_class_properties(
        &lightdm_class,
        "lightdm".to_string(),
        properties_with_getter,
        properties_with_accessor,
        page,
        context,
    );
    initialize_class_methods(
        &lightdm_class,
        "lightdm".to_string(),
        methods,
        page,
        context,
    );

    let jsc_lightdm = ldm_constructor.constructor_callv(&[]);
    let lightdm_obj = jsc::Value::new_object(context, jsc_lightdm, Some(&lightdm_class));
    signals.into_iter().for_each(|signal| {
        let name = signal
            .object_get_property("_name")
            .expect("signal does not have property 'name'");
        lightdm_obj.object_set_property(&name.to_string(), &signal);
    });
    global_object.object_set_property("lightdm", &lightdm_obj);

    let event_class = global_object
        .object_get_property("Event")
        .expect("Event class does not exist");
    let ready_event = event_class
        .constructor_callv(&[jsc::Value::new_string(context, Some("GreeterReady"))])
        .expect("can not construct an event");
    global_object.object_set_property("_ready_event", &ready_event);
    page.connect_document_loaded(move |_| {
        if let Some(dispatch_event) = global_object.object_get_property("dispatchEvent") {
            let ready_event = global_object.object_get_property("_ready_event");
            let _ = dispatch_event.function_callv(&[ready_event.unwrap()]);
        }
    });
}

pub fn handle_lightdm_signal(context: &jsc::Context, signal: &str, json_params: &str) -> bool {
    let lightdm_obj = context
        .global_object()
        .unwrap()
        .object_get_property("lightdm")
        .unwrap();

    let jsc_signal = if let Some(signal) = lightdm_obj.object_get_property(signal) {
        signal
    } else {
        return false;
    };
    let params = jsc::Value::from_json(context, json_params);
    let _ = jsc_signal.object_invoke_methodv("emit", &[params]);

    true
}
