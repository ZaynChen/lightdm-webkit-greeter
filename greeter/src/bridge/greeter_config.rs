use gtk::glib::{Variant, variant::ToVariant};
use lightdm::prelude::*;

use ext::prelude::*;

use std::ops::Deref;

use crate::settings::Settings;

pub(super) struct GreeterConfig {
    context: jsc::Context,
    config: Settings,
}

impl Deref for GreeterConfig {
    type Target = Settings;

    fn deref(&self) -> &Self::Target {
        &self.config
    }
}

impl GreeterConfig {
    pub(super) fn new(context: jsc::Context, config: Settings) -> Self {
        Self { context, config }
    }

    pub(super) fn handle(&self, name: &str) -> Variant {
        let context = &self.context;
        let ret = match name {
            "branding" => self.branding(),
            "greeter" => self.greeter(),
            "features" => self.features(),
            "layouts" => self.layouts(),
            _ => jsc::Value::new_undefined(context),
        };

        if let Some(json) = ret.to_json(0) {
            json.to_variant()
        } else {
            "undefined".to_variant()
        }
    }

    fn branding(&self) -> jsc::Value {
        let images_dir = self.branding_background_images_dir();
        let logo_image = self.branding_logo_image();
        let user_image = self.branding_user_image();

        let context = &self.context;
        let value = jsc::Value::new_object(context, None, None);
        value.object_set_property(
            "background_images_dir",
            &jsc::Value::new_string(context, Some(images_dir)),
        );
        value.object_set_property(
            "logo_image",
            &jsc::Value::new_string(context, Some(logo_image)),
        );
        value.object_set_property(
            "user_image",
            &jsc::Value::new_string(context, Some(user_image)),
        );

        value
    }

    fn greeter(&self) -> jsc::Value {
        let debug_mode = self.debug_mode();
        let detect_theme_errors = self.detect_theme_errors();
        let screensaver_timeout = self.screensaver_timeout();
        let secure_mode = self.secure_mode();
        let theme = self.theme();
        let icon_theme = self.icon_theme();
        let time_language = self.time_language();

        let context = &self.context;
        let value = jsc::Value::new_object(context, None, None);
        value.object_set_property("debug_mode", &jsc::Value::new_boolean(context, debug_mode));
        value.object_set_property(
            "detect_theme_errors",
            &jsc::Value::new_boolean(context, detect_theme_errors),
        );
        value.object_set_property(
            "screensaver_timeout",
            &jsc::Value::new_number(context, screensaver_timeout as f64),
        );
        value.object_set_property(
            "secure_mode",
            &jsc::Value::new_boolean(context, secure_mode),
        );
        value.object_set_property("theme", &jsc::Value::new_string(context, Some(theme)));
        value.object_set_property(
            "icon_theme",
            &jsc::Value::new_string(context, Some(icon_theme)),
        );
        value.object_set_property(
            "time_language",
            &jsc::Value::new_string(context, Some(time_language)),
        );

        value
    }

    fn features(&self) -> jsc::Value {
        let battery = self.battery();
        let backlight_enabled = self.backlight_enabled();
        let backlight_value = self.backlight_value();
        let backlight_steps = self.backlight_steps();

        let context = &self.context;
        let backlight = jsc::Value::new_object(context, None, None);
        backlight.object_set_property(
            "enabled",
            &jsc::Value::new_boolean(context, backlight_enabled),
        );
        backlight.object_set_property(
            "value",
            &jsc::Value::new_number(context, backlight_value as f64),
        );
        backlight.object_set_property(
            "steps",
            &jsc::Value::new_number(context, backlight_steps as f64),
        );

        let value = jsc::Value::new_object(context, None, None);
        value.object_set_property("battery", &jsc::Value::new_boolean(context, battery));
        value.object_set_property("backlight", &backlight);

        value
    }

    fn layouts(&self) -> jsc::Value {
        let layouts = lightdm::functions::layouts();
        let config_layouts = self.config_layouts();

        let context = &self.context;
        let mut vals = Vec::new();
        for layout in layouts.iter() {
            let name = layout.name().unwrap();
            if config_layouts.iter().any(|l| l.replace(" ", "\t") == name) {
                let val = layout.to_jscvalue(context);
                vals.push(val);
            }
        }

        jsc::Value::new_array_from_garray(context, &vals)
    }
}
