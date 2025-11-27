use serde::{Deserialize, Serialize};

use crate::theme::{DEFAULT_THEMES_DIR, load_theme_html};

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
struct Branding {
    background_images_dir: String,
    logo_image: String,
    user_image: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Greeter {
    debug_mode: bool,
    detect_theme_errors: bool,
    screensaver_timeout: u32,
    secure_mode: bool,
    theme: String,
    icon_theme: String,
    time_language: String,
}

impl Default for Greeter {
    fn default() -> Self {
        Self {
            debug_mode: false,
            detect_theme_errors: true,
            screensaver_timeout: 300,
            secure_mode: true,
            theme: "gruvbox".to_string(),
            icon_theme: Default::default(),
            time_language: Default::default(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Backlight {
    enabled: bool,
    steps: u32,
    value: i32,
}

impl Default for Backlight {
    fn default() -> Self {
        Self {
            enabled: false,
            steps: 0,
            value: 10,
        }
    }
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
struct Features {
    battery: bool,
    backlight: Backlight,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Theme {
    pub primary_html: String,
    pub secondary_html: Option<String>,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            primary_html: "index.html".to_string(),
            secondary_html: Default::default(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Settings {
    branding: Branding,
    greeter: Greeter,
    features: Features,
    theme: Option<Theme>,
    themes_dir: Option<String>,
    layouts: Vec<String>,
}

impl Default for Settings {
    fn default() -> Self {
        load_configuration(false, None)
    }
}

impl Settings {
    pub fn new(debug: bool, theme: Option<&str>) -> Self {
        load_configuration(debug, theme)
    }

    pub fn debug_mode(&self) -> bool {
        self.greeter.debug_mode
    }

    pub fn detect_theme_errors(&self) -> bool {
        self.greeter.detect_theme_errors
    }

    pub fn screensaver_timeout(&self) -> u32 {
        self.greeter.screensaver_timeout
    }

    pub fn secure_mode(&self) -> bool {
        self.greeter.secure_mode
    }

    pub fn theme(&self) -> &str {
        &self.greeter.theme
    }

    pub fn icon_theme(&self) -> &str {
        &self.greeter.icon_theme
    }

    pub fn time_language(&self) -> &str {
        &self.greeter.time_language
    }

    pub fn branding_background_images_dir(&self) -> &str {
        &self.branding.background_images_dir
    }

    pub fn branding_logo_image(&self) -> &str {
        &self.branding.logo_image
    }

    pub fn branding_user_image(&self) -> &str {
        &self.branding.user_image
    }

    pub fn battery(&self) -> bool {
        self.features.battery
    }

    pub fn backlight_enabled(&self) -> bool {
        self.features.backlight.enabled
    }

    pub fn backlight_value(&self) -> i32 {
        self.features.backlight.value
    }

    pub fn backlight_steps(&self) -> u32 {
        self.features.backlight.steps
    }

    pub fn config_layouts(&self) -> &Vec<String> {
        &self.layouts
    }

    pub fn primary_html(&self) -> String {
        self.theme
            .as_ref()
            .map_or("index.html".to_string(), |h| h.primary_html.clone())
    }

    pub fn secondary_html(&self) -> Option<String> {
        self.theme
            .as_ref()
            .map_or(Some(self.primary_html()), |h| h.secondary_html.clone())
    }

    pub fn themes_dir(&self) -> Option<&str> {
        self.themes_dir.as_deref()
    }

    fn set_themes_dir(&mut self, themes_dir: &str) {
        self.themes_dir = Some(themes_dir.to_string())
    }

    fn set_debug_mode(&mut self, debug_mode: bool) {
        self.greeter.debug_mode |= debug_mode;
    }

    fn set_theme(&mut self, theme: &str) {
        self.greeter.theme = theme.to_string();
    }

    fn set_theme_html(&mut self, primary_html: String, secondary_html: Option<String>) {
        self.theme = Some(Theme {
            primary_html,
            secondary_html,
        })
    }
}

pub fn load_configuration(debug: bool, theme: Option<&str>) -> Settings {
    let path_to_config = "/etc/lightdm/web-greeter.yml";
    let content = std::fs::read_to_string(path_to_config).expect("Can not read config file");
    let mut config =
        serde_yaml_ng::from_str::<Settings>(&content).expect("config file structure error");
    if debug {
        config.set_debug_mode(true);
    }
    if let Some(theme) = theme {
        config.set_theme(theme);
    }
    if config.themes_dir().is_none() {
        config.set_themes_dir(DEFAULT_THEMES_DIR);
    }

    let (primary_html, secondary_html) =
        load_theme_html(config.theme(), config.themes_dir().unwrap());
    config.set_theme_html(primary_html, Some(secondary_html));

    logger_debug!("Configuration loaded");
    config
}
