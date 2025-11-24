use gtk::glib::clone;

use ext::prelude::*;

pub fn lightdm_signal_initialize(context: &jsc::Context) -> Vec<jsc::Value> {
    let signal_class = context.register_class("__LightDMSignal", None).unwrap();
    signal_class.add_method_variadic("connect", connect);
    signal_class.add_method_variadic(
        "disconnect",
        clone!(
            #[strong]
            context,
            move |instance, params| disconnect(instance, params, &context)
        ),
    );
    signal_class.add_method_variadic("emit", emit);

    let signal_constructor = signal_class
        .add_constructor_variadic(None, |_| None)
        .expect("add signal constructor failed");

    let signals = [
        "authentication_complete",
        "autologin_timer_expired",
        "show_prompt",
        "show_message",
    ];

    signals
        .into_iter()
        .map(|name| signal_new(context, &signal_class, &signal_constructor, name))
        .collect()
}

fn signal_new(
    context: &jsc::Context,
    class: &jsc::Class,
    constructor: &jsc::Value,
    name: &str,
) -> jsc::Value {
    let value = constructor.constructor_callv(&[]);
    let signal = jsc::Value::new_object(context, value, Some(class));
    signal.object_set_property("_name", &jsc::Value::new_string(context, Some(name)));
    signal.object_set_property(
        "_callbacks",
        &jsc::Value::new_array_from_garray(context, &[]),
    );

    signal
}

fn connect(instance: &jsc::Value, params: &[jsc::Value]) -> Option<jsc::Value> {
    if params.is_empty() {
        return None;
    }

    let func = params.first().unwrap();
    if !func.is_function() {
        return None;
    }

    let arr = instance.object_get_property("_callbacks");
    if let Some(arr) = arr {
        let props = arr.object_enumerate_properties();
        arr.object_set_property_at_index(props.len() as u32, func);
    }
    None
}

fn disconnect(
    instance: &jsc::Value,
    params: &[jsc::Value],
    context: &jsc::Context,
) -> Option<jsc::Value> {
    if params.is_empty() {
        return None;
    }

    let func = params.first().unwrap();
    if !func.is_function() {
        return None;
    }

    let arr = instance.object_get_property("_callbacks");
    if let Some(arr) = arr {
        let props = arr.object_enumerate_properties();
        if props.is_empty() {
            return None;
        }
        let new_arr = jsc::Value::new_array_from_garray(context, &[]);
        props
            .iter()
            .enumerate()
            .filter_map(|(i, _)| {
                if let Some(obtained) = arr.object_get_property_at_index(i as u32)
                    && &obtained != func
                {
                    Some(obtained)
                } else {
                    None
                }
            })
            .enumerate()
            .for_each(|(i, p)| new_arr.object_set_property_at_index(i as u32, &p));
        instance.object_set_property("_callbacks", &new_arr);
    }
    None
}

fn emit(instance: &jsc::Value, params: &[jsc::Value]) -> Option<jsc::Value> {
    let arr = instance.object_get_property("_callbacks");
    if let Some(arr) = arr {
        let props = arr.object_enumerate_properties();
        if props.is_empty() {
            return None;
        }
        props.iter().enumerate().for_each(|(i, _)| {
            let obtained = arr.object_get_property_at_index(i as u32);
            if let Some(obtained) = obtained {
                let _ = obtained.function_callv(params);
            }
        })
    }
    None
}
