use gtk::glib::clone;

use ext::prelude::*;

use super::{JscValueDefault, method_callback};

pub fn theme_utils_initialize(page: &wwpe::WebPage, context: &jsc::Context) {
    let global_object = context.global_object().expect("get global_object failed");

    let theme_utils_class = context
        .register_class("__ThemeUtils", None)
        .expect("register __ThemeUtils failed");
    let constructor = theme_utils_class
        .add_constructor_variadic(None, |_| None)
        .unwrap();

    theme_utils_class.add_method_variadic(
        "dirlist",
        clone!(
            #[strong]
            page,
            #[strong]
            context,
            move |_, args| dirlist(&page, &context, args),
        ),
    );

    theme_utils_class.add_method_variadic(
        "get_current_localized_date",
        clone!(
            #[strong]
            context,
            move |_, _| get_current_localized_date(&context)
        ),
    );
    theme_utils_class.add_method_variadic(
        "get_current_localized_time",
        clone!(
            #[strong]
            context,
            move |_, _| get_current_localized_time(&context)
        ),
    );

    let theme_utils = constructor.constructor_callv(&[]);
    let theme_utils_obj = jsc::Value::new_object(context, theme_utils, Some(&theme_utils_class));

    global_object.object_set_property("theme_utils", &theme_utils_obj);
}

fn get_current_localized_date(context: &jsc::Context) -> Option<jsc::Value> {
    let intl = context.value("Intl").expect("get Intl failed");
    let datetime_fmt = intl
        .object_get_property("DateTimeFormat")
        .expect("get DateTimeFormat failed");

    let time_language = context
        .evaluate("greeter_config.greeter.time_language")
        .unwrap()
        .to_str();
    let jsc_locales = if !time_language.is_empty() {
        jsc::Value::new_array_from_garray(
            context,
            &[jsc::Value::new_string(context, Some(&time_language))],
        )
    } else {
        jsc::Value::new_array_from_garray(context, &[])
    };

    let two_digit = jsc::Value::new_string(context, Some("2-digit"));
    let option_date = jsc::Value::new_object(context, None, None);
    option_date.object_set_property("day", &two_digit);
    option_date.object_set_property("month", &two_digit);
    option_date.object_set_property("year", &two_digit);

    let fmt_date = datetime_fmt
        .function_callv(&[jsc_locales.clone(), option_date])
        .expect("get fmt_date failed");
    let now = context.evaluate("new Date()").expect("new Date() failed");

    Some(
        fmt_date
            .object_invoke_methodv("format", &[now])
            .expect("format now failed"),
    )
}

fn get_current_localized_time(context: &jsc::Context) -> Option<jsc::Value> {
    let intl = context.value("Intl").expect("get Intl failed");
    let datetime_fmt = intl
        .object_get_property("DateTimeFormat")
        .expect("get DateTimeFormat failed");

    let time_language = context
        .evaluate("greeter_config.greeter.time_language")
        .unwrap()
        .to_str();
    let jsc_locales = if !time_language.is_empty() {
        jsc::Value::new_array_from_garray(
            context,
            &[jsc::Value::new_string(context, Some(&time_language))],
        )
    } else {
        jsc::Value::new_array_from_garray(context, &[])
    };

    let two_digit = jsc::Value::new_string(context, Some("2-digit"));
    let option_date = jsc::Value::new_object(context, None, None);
    option_date.object_set_property("hour", &two_digit);
    option_date.object_set_property("minute", &two_digit);

    let fmt_date = datetime_fmt
        .function_callv(&[jsc_locales.clone(), option_date])
        .expect("get fmt_date failed");
    let now = context.evaluate("new Date()").expect("new Date() failed");
    Some(
        fmt_date
            .object_invoke_methodv("format", &[now])
            .expect("format now failed"),
    )
}

fn dirlist(
    page: &wwpe::WebPage,
    context: &jsc::Context,
    args: &[jsc::Value],
) -> Option<jsc::Value> {
    if args.len() < 3 {
        return None;
    }
    let console = context.value("console").expect("get console failed");

    let path = args[0].to_str();
    let callback = &args[2];
    let empty_value = jsc::Value::new_array_from_garray(context, &[]);

    let res = if path.is_empty() {
        let _ = console.object_invoke_methodv(
            "error",
            &[jsc::Value::new_string(
                context,
                Some("theme_utils.dirlist(): path must be a non-empty string!"),
            )],
        );
        empty_value
    } else {
        method_callback(
            page,
            context,
            "theme_utils",
            "dirlist",
            args,
            JscValueDefault::Array,
        )
        .unwrap()
    };
    jsc_callback_call(
        context,
        &console,
        callback,
        &[res],
        "theme_utils.dirlist(): callback is not a function",
    );
    None
}

fn jsc_callback_call(
    context: &jsc::Context,
    console: &jsc::Value,
    callback: &jsc::Value,
    params: &[jsc::Value],
    err_msg: &str,
) {
    if callback.is_function() {
        let _ = callback.function_callv(params);
    } else {
        let _ = console
            .object_invoke_methodv("error", &[jsc::Value::new_string(context, Some(err_msg))]);
    }
}
