// logger.rs
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

macro_rules! logger_debug {
    ($($arg:tt)*) => {
        logger_raw!("DEBUG", $($arg)*);
    }
}

macro_rules! logger_warn {
    ($($arg:tt)*) => {
        logger_raw!("WARN", $($arg)*);
    }
}

macro_rules! logger_error {
    ($($arg:tt)*) => {
        logger_raw!("ERROR", $($arg)*);
    }
}

macro_rules! logger_raw {
    ($log_domain:literal, $($arg:tt)*) => {{
        let timestamp = gtk::glib::DateTime::now_local()
            .unwrap()
            .format("%Y-%m-%d %H:%M:%S")
            .unwrap();
        let domain = $log_domain;
        eprintln!(
            "{timestamp} [ {domain} ] {} {}: {}",
            file!(),
            line!(),
            format_args!($($arg)*)
        );
    }}
}
