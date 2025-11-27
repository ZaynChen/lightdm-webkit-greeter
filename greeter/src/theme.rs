// theme.rs
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

use std::{collections::BTreeMap, path::PathBuf};

pub const DEFAULT_THEMES_DIR: &str = "/usr/share/web-greeter/themes/";
const DEFAULT_THEME: &str = "default";

fn list_themes() -> Vec<String> {
    let mut themes = match std::fs::read_dir(DEFAULT_THEMES_DIR) {
        Ok(dir) => dir
            .filter_map(|ent| ent.ok())
            .filter(|ent| ent.file_type().unwrap().is_dir())
            .map(|ent| ent.file_name().to_string_lossy().to_string())
            .collect(),
        Err(_) => {
            println!("Threre are no themes located at {DEFAULT_THEMES_DIR}");
            vec![]
        }
    };
    themes.sort();
    themes
}

pub fn print_themes() {
    let themes = list_themes();
    if themes.is_empty() {
        return;
    }

    println!("Themes are located at {DEFAULT_THEMES_DIR}\n");
    themes.iter().for_each(|t| println!("- {t}"));
}

pub fn load_theme_html(theme: &str, themes_dir: &str) -> (String, String) {
    let theme_path = PathBuf::from(theme);
    let themes_dir_path = PathBuf::from(themes_dir);
    assert!(
        themes_dir_path.is_absolute(),
        "'{themes_dir}' is not an absolute pathname"
    );

    // get absolute path
    let absolute_path = if theme_path.is_absolute() {
        theme_path
    } else if theme_path.components().count() == 1 {
        themes_dir_path.join(theme_path)
    } else {
        match std::env::current_dir() {
            Ok(cwd) => cwd.join(theme_path),
            Err(_) => themes_dir_path.join(DEFAULT_THEME),
        }
    };

    // get theme dirname
    let final_dir = if absolute_path.is_file() && theme.ends_with(".html") {
        absolute_path.with_file_name("")
    } else {
        absolute_path.clone()
    };

    // check dir existence
    let theme_dir = if final_dir.is_dir() {
        final_dir
    } else {
        logger_warn!("{theme} theme does not exists. Using {DEFAULT_THEME} theme",);
        themes_dir_path.join(DEFAULT_THEME)
    };

    let (primary, secondary) = load_theme_config(&theme_dir);
    let primary_html = if absolute_path.is_file() && theme.ends_with(".html") {
        absolute_path.to_string_lossy().to_string()
    } else {
        let primary_path = PathBuf::from(&theme_dir).join(&primary);
        if primary_path.is_file() && primary.ends_with(".html") {
            primary_path.to_string_lossy().to_string()
        } else {
            PathBuf::from(DEFAULT_THEMES_DIR)
                .join(DEFAULT_THEME)
                .join("index.html")
                .to_string_lossy()
                .to_string()
        }
    };

    if let Some(path) = secondary.map(|s| PathBuf::from(&theme_dir).join(s))
        && path.is_file()
    {
        (primary_html, path.to_string_lossy().to_string())
    } else {
        (primary_html.clone(), primary_html)
    }
}

fn load_theme_config(theme_dir: &PathBuf) -> (String, Option<String>) {
    match std::fs::read_to_string(PathBuf::from(&theme_dir).join("index.yml")) {
        Ok(content) => match serde_yaml_ng::from_str::<BTreeMap<String, String>>(&content) {
            Ok(config_map) => {
                let primary = config_map
                    .get("primary_html")
                    .unwrap_or(&"index.html".to_string())
                    .clone();
                let secondary = config_map.get("secondary_html");
                if let Some(s) = secondary {
                    (primary, Some(s.clone()))
                } else {
                    (primary, None)
                }
            }
            Err(e) => {
                logger_error!("Parsing failed: \n\t{e}");
                ("index.html".to_string(), None)
            }
        },
        Err(e) => {
            logger_error!("Theme config was not loaded:\n\t{e}");
            ("index.html".to_string(), None)
        }
    }
}
