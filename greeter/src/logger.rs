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
