pub fn init(level: log::LevelFilter) {
    env_logger::Builder::from_default_env().filter(None, level).init();
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        log::debug!($($arg)*)
    };
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        log::info!($($arg)*)
    };
}

#[macro_export]
macro_rules! warning {
    ($($arg:tt)*) => {
        log::warn!($($arg)*)
    };
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        log::error!($($arg)*)
    };
}
