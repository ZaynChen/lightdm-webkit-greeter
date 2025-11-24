use ext::prelude::*;

use crate::extension::{JscValueDefault, initialize_class_methods, initialize_class_properties};

pub fn greeter_comm_initialize(page: &wwpe::WebPage, context: &jsc::Context) {
    let greeter_comm_class = context
        .register_class("__GreeterComm", None)
        .expect("register __GreeterComm failed");
    let constructor = greeter_comm_class.add_constructor_variadic(None, |_| None);

    let properties = vec![("window_metadata".to_string(), JscValueDefault::Null)];
    let methods = vec![("broadcast".to_string(), JscValueDefault::Boolean)];
    initialize_class_properties(
        &greeter_comm_class,
        "greeter_comm".to_string(),
        properties,
        vec![],
        page,
        context,
    );
    initialize_class_methods(
        &greeter_comm_class,
        "greeter_comm".to_string(),
        methods,
        page,
        context,
    );

    let obj = constructor.unwrap().constructor_callv(&[]);
    let comm = jsc::Value::new_object(context, obj, Some(&greeter_comm_class));
    context
        .global_object()
        .unwrap()
        .object_set_property("greeter_comm", &comm);
}

pub fn handle_comm_broadcast(context: &jsc::Context, method: &str, json_params: &str) -> bool {
    if method != "_emit" {
        return false;
    }

    let params = jsc::Value::from_json(context, json_params);
    let data = params.object_get_property_at_index(0).unwrap();

    let global_object = context.global_object().unwrap();
    let dispatch_event = global_object.object_get_property("dispatchEvent").unwrap();

    let event_class = global_object.object_get_property("Event").unwrap();
    let broadcast_event = event_class
        .constructor_callv(&[jsc::Value::new_string(
            context,
            Some("GreeterBroadcastEvent"),
        )])
        .unwrap();

    broadcast_event.object_set_property("window", &jsc::Value::new_null(context));
    broadcast_event.object_set_property("data", &data);

    let _ = dispatch_event.function_callv(&[broadcast_event]);

    true
}
