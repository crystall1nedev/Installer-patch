pub mod cli;

pub const RELEASE_URL: &str = "https://api.github.com/repos/Vendicated/Vencord/releases/latest";
pub const RELEASE_URL_FALLBACK: &str = "https://vencord.dev/releases/vencord";
pub const RELEASE_TAG_DOWNLOAD: &str = "https://github.com/Vendicated/Vencord/releases/download/devbuild";
pub const OPENASAR_URL: &str = "https://github.com/GooseMod/OpenAsar/releases/download/nightly/app.asar";
pub const USER_AGENT: &str = "VencordInstaller (https://github.com/Vencord/Installer)";

pub const INFO: &str = "INFO";
pub const WARN: &str = "WARN";
pub const ERROR: &str = "ERROR";
pub const FATAL: &str = "FATAL";
pub const SUCCESS: &str = "SUCCESS";

pub fn log(tag: &str, message: &str) {
    println!("{:<7} {}", tag, message);
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => (
        $crate::log($crate::INFO, &format!($($arg)*))
    )
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => (
        $crate::log($crate::WARN, &format!($($arg)*))
    )
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => (
        $crate::log($crate::ERROR, &format!($($arg)*))
    )
}

#[macro_export]
macro_rules! fatal {
    ($($arg:tt)*) => (
        $crate::log($crate::FATAL, &format!($($arg)*))
    )
}

#[macro_export]
macro_rules! success {
    ($($arg:tt)*) => (
        $crate::log($crate::SUCCESS, &format!($($arg)*))
    )
}
