use ext::prelude::*;

use crate::extension::initialize_class_properties;

use super::JscValueDefault;

pub fn greeter_config_initialize(page: &wwpe::WebPage, context: &jsc::Context) {
    let global_object = context.global_object().unwrap();

    let greeter_class = context
        .register_class("__GreeterConfig", None)
        .expect("register __GreeterConfig failed");
    let constructor = greeter_class
        .add_constructor_variadic(None, |_| None)
        .expect("add greeter_config constructor failed");

    let properties_with_getter = vec![
        ("branding".to_string(), JscValueDefault::Null),
        ("greeter".to_string(), JscValueDefault::Null),
        ("features".to_string(), JscValueDefault::Null),
        ("layouts".to_string(), JscValueDefault::Null),
    ];

    initialize_class_properties(
        &greeter_class,
        "greeter_config".to_string(),
        properties_with_getter,
        vec![],
        page,
        context,
    );

    let greeter_config = constructor.constructor_callv(&[]);
    let greeter_config_obj = jsc::Value::new_object(context, greeter_config, Some(&greeter_class));
    global_object.object_set_property("greeter_config", &greeter_config_obj);
}
