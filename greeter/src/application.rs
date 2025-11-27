// application.rs
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

use glib::translate::*;
use gtk::{
    Application, CssProvider,
    gdk::{Display, Monitor},
    gio::{ActionEntry, Cancellable, File, MenuModel},
    glib,
    prelude::*,
};

use std::rc::Rc;

use crate::{bridge::Dispatcher, browser::Browser, settings::Settings, webview::webview_new};

const PRIMARY_MONITOR: usize = 0;
const WEB_EXTENSIONS_DIR: &str = "/usr/lib/lightdm-webkit-greeter";

pub fn on_activate(app: &Application, debug: bool, theme: Option<&str>) {
    let config = Settings::new(debug, theme);
    let debug = config.debug_mode();

    let secure_mode = config.secure_mode();
    let detect_theme_error = config.detect_theme_errors();

    let api = if let Ok((content, _)) =
        File::for_uri("resource:///com/github/zaynchen/lightdm-webkit-greeter/lightdm.js")
            .load_contents(Cancellable::NONE)
    {
        String::from_utf8(content.to_vec()).unwrap()
    } else {
        "".to_string()
    };

    let webcontext = webkit::WebContext::default().expect("default web context does not exist");
    webcontext.set_cache_model(webkit::CacheModel::DocumentViewer);
    webcontext.connect_initialize_web_process_extensions(move |context: &webkit::WebContext| {
        let data = (secure_mode, detect_theme_error, &api).to_variant();
        logger_debug!("Extension initialized");

        context.set_web_process_extensions_directory(WEB_EXTENSIONS_DIR);
        context.set_web_process_extensions_initialization_user_data(&data);
    });

    let display = Display::default().expect("Default display does not exist");
    let provider = CssProvider::new();
    provider.load_from_resource("/com/github/zaynchen/lightdm-webkit-greeter/style.css");

    gtk::style_context_add_provider_for_display(
        &display,
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
    set_cursor(&display);

    let primary_html = config.primary_html();
    let secondary_html = config.secondary_html().unwrap();
    let browsers: Vec<Browser> = display
        .monitors()
        .iter::<Monitor>()
        .filter_map(|m| m.ok())
        .map(|m| (gen_id(&m), m.geometry()))
        .enumerate()
        .map(|(idx, (id, geometry))| {
            let is_primary = idx == PRIMARY_MONITOR;
            let theme_file = if is_primary {
                &primary_html
            } else {
                &secondary_html
            };
            Browser::builder()
                .debug_mode(debug)
                .id(id)
                .geometry(geometry)
                .primary(is_primary)
                .application(app)
                .webview(webview_new(debug, theme_file))
                .build()
        })
        .collect();
    let browsers = Rc::new(browsers);
    let dispatcher = Rc::new(Dispatcher::new(
        config,
        jsc::Context::default(),
        browsers.clone(),
    ));
    browsers.iter().for_each(|browser| {
        browser.connect_user_message_received(dispatcher.clone());
    })
}

pub fn on_startup(app: &Application) {
    app.set_accels_for_action("app.quit", &["<Ctl>Q"]);
    app.set_accels_for_action("win.toggle-inspector", &["<Ctl><Shift>I", "F12"]);

    app.set_accels_for_action("win.undo", &["<Ctl>Z"]);
    app.set_accels_for_action("win.redo", &["<Ctl><Shift>Z"]);
    app.set_accels_for_action("win.cut", &["<Ctl>X"]);
    app.set_accels_for_action("win.copy", &["<Ctl>C"]);
    app.set_accels_for_action("win.paste", &["<Ctl>V"]);
    app.set_accels_for_action("win.paste-plain", &["<Ctl><Shift>V"]);
    app.set_accels_for_action("win.select-all", &["<Ctl>A"]);

    app.set_accels_for_action("win.zoom-normal", &["<Ctl>0", "<Ctl>KP_0"]);
    app.set_accels_for_action(
        "win.zoom-in",
        &["<Ctl>plus", "<Ctl>equal", "<Ctl>KP_Add", "ZoomIn"],
    );
    app.set_accels_for_action(
        "win.zoom-out",
        &["<Ctl>minus", "<Ctl>KP_Subtract", "ZoomOut"],
    );
    app.set_accels_for_action("win.fullscreen", &["F11"]);
    app.set_accels_for_action("win.reload", &["<Ctl>R", "F5", "Refresh", "Reload"]);
    app.set_accels_for_action("win.force-reload", &["<Ctl><Shift>R", "<Shift>F5"]);

    app.set_accels_for_action("win.close", &["<Ctl>W"]);
    app.set_accels_for_action("win.minimize", &["<Ctl>M"]);

    app.add_action_entries([ActionEntry::builder("quit")
        .activate(|app: &Application, _, _| app.quit())
        .build()]);

    app.set_menubar(
        gtk::Builder::from_resource("/com/github/zaynchen/lightdm-webkit-greeter/menubar.ui")
            .object::<MenuModel>("menu")
            .as_ref(),
    );
}

fn set_cursor(display: &gtk::gdk::Display) {
    if display.backend().is_x11() {
        logger_debug!("Setup root window cursor: GDK backend is X11");
        let display = display
            .downcast_ref::<gdkx::X11Display>()
            .expect("the display should be x11");
        let root_window = display.xrootwindow();
        unsafe {
            let cursor = gdkx::x11::xlib::XCreateFontCursor(display.xdisplay(), 68);
            gdkx::x11::xlib::XDefineCursor(display.xdisplay(), root_window, cursor);
        }
    }
}

fn gen_id(monitor: &Monitor) -> u64 {
    let manufacture = monitor.manufacturer();
    let model = monitor.model();
    let manufacture_hash = manufacture.map_or(0, |m| unsafe {
        glib::ffi::g_str_hash(m.into_glib_ptr() as glib::ffi::gconstpointer)
    }) as u64;
    let model_hash = model.map_or(0, |m| unsafe {
        glib::ffi::g_str_hash(m.into_glib_ptr() as glib::ffi::gconstpointer)
    }) as u64;

    (manufacture_hash << 24) | (model_hash << 8)
}
