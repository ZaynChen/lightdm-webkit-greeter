// SPDX-FileCopyrightText: 2025 ZaynChen
//
// SPDX-License-Identifier: GPL-3.0-or-later

use gtk::glib::{self, MainContext, clone, variant::ToVariant};

use ext::prelude::*;

pub fn web_page_initialize(api_code: String) {
    wwpe::ScriptWorld::default()
        .expect("get default ScriptWorld failed")
        .connect_window_object_cleared(move |world, page, frame| {
            let context = frame.js_context_for_script_world(world).unwrap();
            let global_object = context.global_object().unwrap();

            global_object.object_set_property("send_request", &send_request(page, &context));
            context.evaluate(&api_code);

            page.connect_document_loaded(move |_| {
                if let Some(ready) = global_object.object_get_property("dispatch_ready_event")
                    && ready.is_function()
                {
                    let _ = ready.function_callv(&[]);
                }
            });

            page.connect_user_message_received(clone!(
                #[strong]
                context,
                move |_, message| user_message_received(message, &context)
            ));
        });
}

fn send_request(page: &wwpe::WebPage, context: &jsc::Context) -> jsc::Value {
    jsc::Value::new_function_variadic(
        context,
        Some("send_request"),
        clone!(
            #[strong]
            page,
            #[strong]
            context,
            move |args| {
                if args.len() != 1 {
                    glib::g_warning!(
                        "",
                        "Invalid number of arguments for send_request: len {}",
                        args.len()
                    );
                    return None;
                }
                let request = &args[0];
                if !request.object_has_property("target")
                    && !request.object_has_property("method")
                    && !request.object_has_property("args")
                {
                    glib::g_warning!("", "request is not a valid Request(target, method, args)");
                    return None;
                }

                let target = request.object_get_property("target").unwrap().to_str();
                let method = request.object_get_property("method").unwrap().to_str();
                let params = request
                    .object_get_property("args")
                    .unwrap()
                    .to_json(0)
                    .unwrap_or("[]".into());

                let message =
                    wwpe::UserMessage::new(&target, Some(&[method.as_str(), &params].to_variant()));
                MainContext::default()
                    .block_on(page.send_message_to_view_future(&message))
                    .ok()
                    .map(|reply| {
                        jsc::Value::from_json(&context, reply.parameters().unwrap().str().unwrap())
                    })
            }
        ),
    )
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
        Some("lightdm") => {
            let _ = context
                .global_object()
                .unwrap()
                .object_get_property("lightdm")
                .unwrap()
                .object_get_property(name)
                .unwrap_or_else(|| panic!("lightdm does not has signal {name}"))
                .object_invoke_methodv("emit", &[jsc::Value::from_json(context, json_params)]);

            true
        }
        Some("greeter_comm") => {
            if name != "_emit" {
                return false;
            }

            let data = jsc::Value::from_json(context, json_params)
                .object_get_property_at_index(0)
                .unwrap();

            let _ = context
                .global_object()
                .unwrap()
                .object_get_property("greeter_comm")
                .unwrap()
                .object_invoke_methodv("_emit", &[data]);

            true
        }
        _ => false,
    }
}
