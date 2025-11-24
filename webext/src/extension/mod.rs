mod greeter_comm;
mod greeter_config;
mod lightdm;
mod lightdm_signal;
mod theme_utils;

use greeter_comm::{greeter_comm_initialize, handle_comm_broadcast};
use greeter_config::greeter_config_initialize;
use lightdm::{handle_lightdm_signal, lightdm_initialize};
use lightdm_signal::lightdm_signal_initialize;
use theme_utils::theme_utils_initialize;

pub fn web_page_initialize() {
    if let Some(world) = wwpe::ScriptWorld::default() {
        world.connect_window_object_cleared(|world, page, frame| {
            let context = frame.js_context_for_script_world(world).unwrap();
            let signals = lightdm_signal_initialize(&context);
            lightdm_initialize(page, &context, signals);
            greeter_config_initialize(page, &context);
            greeter_comm_initialize(page, &context);
            theme_utils_initialize(page, &context);

            page.connect_user_message_received(clone!(
                #[strong]
                context,
                move |_, message| user_message_received(message, &context)
            ));
        });
    }
}

fn user_message_received(message: &wwpe::UserMessage, context: &jsc::Context) -> bool {
    if !matches!(
        message.name().as_deref(),
        Some("lightdm") | Some("greeter_comm")
    ) {
        return false;
    }

    let msg_param = message.parameters().unwrap();
    if msg_param.is_type(gtk::glib::VariantTy::ARRAY) {
        let p_len = msg_param.n_children();
        if p_len == 0 || p_len > 2 {
            return false;
        }
    } else {
        return false;
    }

    let name_var = msg_param.child_value(0);
    let params_var = msg_param.child_value(1);

    let name = name_var.str().unwrap();
    let json_params = params_var.str().unwrap();

    match message.name().as_deref() {
        Some("lightdm") => handle_lightdm_signal(context, name, json_params),
        Some("greeter_comm") => handle_comm_broadcast(context, name, json_params),
        _ => false,
    }
}

use gtk::glib::{self, MainContext, clone, variant::ToVariant};

use ext::prelude::*;

fn initialize_class_properties(
    class: &jsc::Class,
    name: String,
    properties_with_getter: Vec<(String, JscValueDefault)>,
    properties_with_accessor: Vec<(String, JscValueDefault)>,
    page: &wwpe::WebPage,
    context: &jsc::Context,
) {
    properties_with_getter
        .into_iter()
        .for_each(|(prop, default)| {
            class.add_property(
                &prop,
                true,
                false,
                clone!(
                    #[strong]
                    page,
                    #[strong]
                    context,
                    #[strong]
                    name,
                    #[strong]
                    prop,
                    move |_, _| Some(getter_callback(&page, &context, &name, &prop, default))
                ),
            )
        });
    properties_with_accessor
        .into_iter()
        .for_each(|(prop, default)| {
            class.add_property(
                &prop,
                true,
                true,
                clone!(
                    #[strong]
                    page,
                    #[strong]
                    context,
                    #[strong]
                    name,
                    #[strong]
                    prop,
                    move |_, value| {
                        if let Some(value) = value {
                            setter_callback(&page, &context, &name, &prop, value);
                            None
                        } else {
                            Some(getter_callback(&page, &context, &name, &prop, default))
                        }
                    }
                ),
            )
        });
}

fn initialize_class_methods(
    class: &jsc::Class,
    name: String,
    methods: Vec<(String, JscValueDefault)>,
    page: &wwpe::WebPage,
    context: &jsc::Context,
) {
    methods.into_iter().for_each(|(method, default)| {
        class.add_method_variadic(
            &method,
            clone!(
                #[strong]
                page,
                #[strong]
                context,
                #[strong]
                name,
                #[strong]
                method,
                move |_, args| method_callback(&page, &context, &name, &method, args, default)
            ),
        )
    });
}

fn getter_callback(
    page: &wwpe::WebPage,
    context: &jsc::Context,
    name: &str,
    prop: &str,
    default: JscValueDefault,
) -> jsc::Value {
    let params = [prop, "[]"];
    let message = wwpe::UserMessage::new(name, Some(&params.to_variant()));
    if let Ok(reply) = MainContext::default().block_on(page.send_message_to_view_future(&message)) {
        jsc::Value::from_json(context, reply.parameters().unwrap().str().unwrap())
    } else {
        default_jsc_value(context, default)
    }
}

fn setter_callback(
    page: &wwpe::WebPage,
    context: &jsc::Context,
    name: &str,
    prop: &str,
    value: jsc::Value,
) {
    let params = jsc::Value::new_array_from_garray(context, &[value])
        .to_json(0)
        .expect("param parse to json failed");
    let message = wwpe::UserMessage::new(name, Some(&[prop, &params].to_variant()));
    match MainContext::default().block_on(page.send_message_to_view_future(&message)) {
        Ok(reply) => glib::g_warning!("", "{:?}", reply.name()),
        Err(e) => glib::g_warning!("", "{}", e.message()),
    }
}

fn method_callback(
    page: &wwpe::WebPage,
    context: &jsc::Context,
    name: &str,
    method: &str,
    args: &[jsc::Value],
    default: JscValueDefault,
) -> Option<jsc::Value> {
    let params = jsc::Value::new_array_from_garray(context, args)
        .to_json(0)
        .expect("param parse to json failed");
    let message = wwpe::UserMessage::new(name, Some(&[method, &params].to_variant()));
    if let Ok(reply) = MainContext::default().block_on(page.send_message_to_view_future(&message)) {
        Some(jsc::Value::from_json(
            context,
            reply.parameters().unwrap().str().unwrap(),
        ))
    } else {
        Some(default_jsc_value(context, default))
    }
}

#[derive(Copy, Clone, Debug)]
enum JscValueDefault {
    Array,
    Boolean,
    Null,
    Number(f64),
}

fn default_jsc_value(context: &jsc::Context, ty: JscValueDefault) -> jsc::Value {
    match ty {
        JscValueDefault::Array => jsc::Value::new_array_from_garray(context, &[]),
        JscValueDefault::Boolean => jsc::Value::new_boolean(context, false),
        JscValueDefault::Null => jsc::Value::new_null(context),
        JscValueDefault::Number(val) => jsc::Value::new_number(context, val),
    }
}
