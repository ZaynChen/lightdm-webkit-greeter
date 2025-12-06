// SPDX-FileCopyrightText: 2025 ZaynChen
//
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::jscext::JSCValueExtManual;
use lightdm::prelude::*;

pub trait ToJSCValue {
    fn to_jscvalue(&self, context: &jsc::Context) -> jsc::Value;
}

impl ToJSCValue for lightdm::User {
    fn to_jscvalue(&self, context: &jsc::Context) -> jsc::Value {
        let value = jsc::Value::new_object(context, None, None);

        let background = self.background();
        let display_name = self.display_name();
        let home_directory = self.home_directory();
        let image = self.image();
        let language = self.language();
        let layout = self.layout();
        let layouts: Vec<jsc::Value> = self
            .layouts()
            .iter()
            .map(|l| jsc::Value::new_string(context, Some(l.as_str())))
            .collect();

        let logged_in = self.is_logged_in();
        let session = self.session();
        let username = self.name();

        value.object_set_property(
            "background",
            &jsc::Value::new_string(context, background.as_ref().map(|s| s.as_str())),
        );
        value.object_set_property(
            "display_name",
            &jsc::Value::new_string(context, display_name.as_ref().map(|s| s.as_str())),
        );
        value.object_set_property(
            "home_directory",
            &jsc::Value::new_string(context, home_directory.as_ref().map(|s| s.as_str())),
        );
        value.object_set_property(
            "image",
            &jsc::Value::new_string(context, image.as_ref().map(|s| s.as_str())),
        );
        value.object_set_property(
            "language",
            &jsc::Value::new_string(context, language.as_ref().map(|s| s.as_str())),
        );
        value.object_set_property(
            "layout",
            &jsc::Value::new_string(context, layout.as_ref().map(|s| s.as_str())),
        );
        value.object_set_property(
            "layouts",
            &jsc::Value::new_array_from_garray(context, &layouts),
        );
        value.object_set_property("logged_in", &jsc::Value::new_boolean(context, logged_in));
        value.object_set_property(
            "session",
            &jsc::Value::new_string(context, session.as_ref().map(|s| s.as_str())),
        );
        value.object_set_property(
            "username",
            &jsc::Value::new_string(context, username.as_ref().map(|s| s.as_str())),
        );

        value
    }
}

impl ToJSCValue for lightdm::Session {
    fn to_jscvalue(&self, context: &jsc::Context) -> jsc::Value {
        let value = jsc::Value::new_object(context, None, None);

        let comment = self.comment();
        let key = self.key();
        let name = self.name();
        let session_type = self.session_type();

        value.object_set_property(
            "comment",
            &jsc::Value::new_string(context, comment.as_ref().map(|s| s.as_str())),
        );
        value.object_set_property(
            "key",
            &jsc::Value::new_string(context, key.as_ref().map(|s| s.as_str())),
        );
        value.object_set_property(
            "name",
            &jsc::Value::new_string(context, name.as_ref().map(|s| s.as_str())),
        );
        value.object_set_property(
            "type",
            &jsc::Value::new_string(context, session_type.as_ref().map(|s| s.as_str())),
        );

        value
    }
}

impl ToJSCValue for lightdm::Language {
    fn to_jscvalue(&self, context: &jsc::Context) -> jsc::Value {
        let value = jsc::Value::new_object(context, None, None);

        let code = self.code();
        let name = self.name();
        let territory = self.territory();

        value.object_set_property(
            "code",
            &jsc::Value::new_string(context, code.as_ref().map(|s| s.as_str())),
        );
        value.object_set_property(
            "name",
            &jsc::Value::new_string(context, name.as_ref().map(|s| s.as_str())),
        );
        value.object_set_property(
            "territory",
            &jsc::Value::new_string(context, territory.as_ref().map(|s| s.as_str())),
        );

        value
    }
}

impl ToJSCValue for lightdm::Layout {
    fn to_jscvalue(&self, context: &jsc::Context) -> jsc::Value {
        let value = jsc::Value::new_object(context, None, None);

        let name = self.name();
        let description = self.description();
        let short_description = self.short_description();

        value.object_set_property(
            "name",
            &jsc::Value::new_string(context, name.as_ref().map(|s| s.as_str())),
        );
        value.object_set_property(
            "description",
            &jsc::Value::new_string(context, description.as_ref().map(|s| s.as_str())),
        );
        value.object_set_property(
            "short_description",
            &jsc::Value::new_string(context, short_description.as_ref().map(|s| s.as_str())),
        );

        value
    }
}
