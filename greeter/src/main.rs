// SPDX-FileCopyrightText: 2025 ZaynChen
//
// SPDX-License-Identifier: GPL-3.0-or-later

#[macro_use]
mod logger;
mod application;
mod bridge;
mod browser;
mod settings;
mod theme;
mod webview;

use gtk::prelude::*;
use gtk::{gio, glib};

use crate::application::{on_activate, on_startup};
use crate::theme::print_themes;

fn main() -> glib::ExitCode {
    let args = CliArgs::parse();
    if args.list {
        print_themes();
        return glib::ExitCode::SUCCESS;
    }

    gio::resources_register_include!("greeter.gresource").expect("Failed to register resources.");

    let webinfo = webkit::ApplicationInfo::new();
    webinfo.set_name("com.github.zaynchen.lightdm-webkit-greeter");

    let app = gtk::Application::builder()
        .application_id("com.github.zaynchen.lightdm-webkit-greeter")
        .flags(Default::default())
        .build();

    let debug = args.debug_mode();
    let theme = args.theme().map(|s| s.to_string());
    app.connect_activate(move |app| on_activate(app, debug, theme.as_deref()));
    app.connect_startup(on_startup);

    let exit_code = app.run_with_args::<glib::GString>(&[]);
    logger_debug!("LightDM WebKit Greeter stopped");
    exit_code
}

use clap::{Parser, ValueEnum};
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Mode {
    Debug,
    Normal,
}

/// A modern, visually appealing greeter for LightDM.
#[derive(Debug, Parser)]
#[command(version, about)]
struct CliArgs {
    /// Debug mode
    #[arg(short, long, group = "debug_mode")]
    debug: bool,
    /// Normal mode
    #[arg(short, long, group = "debug_mode")]
    normal: bool,
    /// Mode
    #[arg(long, group = "debug_mode")]
    mode: Option<Mode>,
    /// Theme
    #[arg(long)]
    theme: Option<String>,
    /// List installed themes
    #[arg(long)]
    list: bool,
}

impl CliArgs {
    fn debug_mode(&self) -> bool {
        self.debug || self.mode == Some(Mode::Debug)
    }

    fn theme(&self) -> Option<&str> {
        self.theme.as_deref()
    }
}
