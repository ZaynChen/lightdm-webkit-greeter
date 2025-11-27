// theme_utils.rs
//
// Copyright (C) 2025 ZaynChen
//
// This file is part of LightDM WebKit Greeter
//
// LightDM WebKit Greeter is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// LightDM WebKit Greeter is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use gtk::glib::{self, tmp_dir, variant::ToVariant};

use ext::prelude::*;

pub(super) struct ThemeUtils {
    context: jsc::Context,
    allowed_dirs: Vec<String>,
}

impl ThemeUtils {
    pub(super) fn new(context: jsc::Context, allowed_dirs: &[String], theme: &str) -> Self {
        let mut allowed_dirs: Vec<String> = allowed_dirs.iter().map(|s| s.to_string()).collect();
        if let Ok(path) = std::fs::canonicalize(theme) {
            let theme_dir = path.with_file_name("");
            allowed_dirs.push(theme_dir.to_string_lossy().to_string());
        }
        allowed_dirs.push(tmp_dir().to_string_lossy().to_string());
        Self {
            context,
            allowed_dirs,
        }
    }

    pub(super) fn handle(&self, name: &str, json_params: &str) -> glib::Variant {
        let context = &self.context;
        let params = jsc::Value::from_json(context, json_params).to_vec();
        let ret = if "dirlist" == name && !params.is_empty() {
            self.dirlist(&params)
        } else {
            jsc::Value::new_undefined(context)
        };

        if let Some(json) = ret.to_json(0) {
            json.to_variant()
        } else {
            "undefined".to_variant()
        }
    }

    fn dirlist(&self, params: &[jsc::Value]) -> jsc::Value {
        let context = &self.context;
        let null = jsc::Value::new_null(context);
        if params.len() < 2 {
            return null;
        }

        let jsc_path = &params[0];
        let jsc_only_images = &params[1];

        let value = jsc::Value::new_array_from_garray(context, &[]);
        if !jsc_path.is_string() || jsc_path.to_string().trim().is_empty() {
            return value;
        }
        let path = jsc_path.to_string();

        if path == "/" || path.starts_with("./") {
            return value;
        }

        let resolved = if let Ok(p) = std::fs::canonicalize(&path) {
            p
        } else {
            return value;
        };

        if !resolved.is_absolute() || !resolved.is_dir() {
            println!("{resolved:?} is not absolute nor an existing directory");
            return value;
        }

        if self.allowed_dirs.iter().all(|d| resolved.starts_with(d)) {
            logger_warn!("Path {resolved:?} is not allowed");
            return value;
        }

        let dir = match std::fs::read_dir(&resolved) {
            Ok(d) => d,
            Err(e) => {
                println!("Opendir error: '{e}'");
                return value;
            }
        };

        let mut files = vec![];
        let regex = glib::Regex::new(
            ".+\\.(jpe?g|png|gif|bmp|webp)",
            glib::RegexCompileFlags::CASELESS,
            glib::RegexMatchFlags::DEFAULT,
        )
        .expect("g_regex_new error")
        .expect("g_regex_new error");
        for entry in dir.filter_map(|v| v.ok()) {
            let filename = entry.file_name();
            if filename == "." || filename == ".." {
                continue;
            }

            let filepath = std::path::PathBuf::from("/")
                .join(&resolved)
                .join(&filename);
            let file_element = jsc::Value::new_string(context, filepath.to_str());
            if jsc_only_images.is_boolean() && jsc_only_images.to_boolean() {
                let s = glib::GStr::from_str_with_nul(filename.to_str().unwrap())
                    .expect("osstring to gstr error");
                if let Ok(ft) = entry.file_type()
                    && ft.is_file()
                    && regex
                        .match_(s, glib::RegexMatchFlags::DEFAULT)
                        .expect("g_regex_match error")
                        .matches()
                {
                    files.push(file_element);
                }
            } else {
                files.push(file_element);
            }
        }

        jsc::Value::new_array_from_garray(context, &files)
    }
}
