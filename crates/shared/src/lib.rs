use std::path::PathBuf;

use vencord_installer_core::Error;
use vencord_installer_core::paths::locations::get_data_path;
use vencord_installer_core::update::{download::prepare_dist_directory, version_check::{check_hash_from_release, check_local_version}};

use tokio::runtime::Runtime;

pub const RELEASE_URL: &str = "https://api.github.com/repos/Vendicated/Vencord/releases/latest";
pub const RELEASE_URL_FALLBACK: &str = "https://vencord.dev/releases/vencord";
pub const RELEASE_TAG_DOWNLOAD: &str = "https://github.com/Vendicated/Vencord/releases/download/devbuild";
pub const OPENASAR_URL: &str = "https://github.com/GooseMod/OpenAsar/releases/download/nightly/app.asar";
pub const USER_AGENT: &str = "VencordInstaller (https://github.com/Vencord/Installer)";

pub fn get_dist_path() -> PathBuf {
    if let Ok(path) = std::env::var("VENCORD_USER_DATA_DIR") {
        PathBuf::from(path).join("dist")
    } else {
        get_data_path("Vencord")
    }
}

pub fn download_assets() -> Result<(), Error> {
    let rt = Runtime::new().unwrap();
    
    info!("Checking for dist files to download...");
    
    let latest_version = rt.block_on(check_hash_from_release(RELEASE_URL, Some(RELEASE_URL_FALLBACK), USER_AGENT));
    let local_version = rt.block_on(check_local_version(&get_dist_path(), r"// Vencord ([0-9a-zA-Z\.-]+)"));
    
    info!("Latest version: {}", latest_version.clone().unwrap_or_default());
    info!("Local version: {}", local_version.clone().unwrap_or_default());
    
    if latest_version.is_some() && latest_version != local_version {
        info!("Downloading dist files...");
        
        rt.block_on(prepare_dist_directory(
            &get_dist_path(),
            RELEASE_TAG_DOWNLOAD,
            USER_AGENT,
            [
                "patcher.js".to_string(),
                "preload.js".to_string(),
                "renderer.js".to_string(),
                "renderer.css".to_string(),
            ],
        ))?;
    } else {
        info!("Nothing new to download, skipping!");
    }
    
    Ok(())
}

// MARK: - Logging

pub const INFO: &str = "INFO";
pub const WARN: &str = "WARN";
pub const ERROR: &str = "ERROR";
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
macro_rules! success {
    ($($arg:tt)*) => (
        $crate::log($crate::SUCCESS, &format!($($arg)*))
    )
}
