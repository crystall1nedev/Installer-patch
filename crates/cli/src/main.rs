#![allow(rustdoc::missing_crate_level_docs)]

mod cli;

#[cfg(any(target_os = "macos", target_os = "linux"))]
use {
    std::process::exit,
    vencord_installer_shared::error,
};

use cli::arguments;

#[cfg(any(target_os = "macos", target_os = "linux"))]
unsafe extern "C" {
    fn geteuid() -> u32;
}

fn main() {
    let matches = arguments::args_build().get_matches();

    // Linux needs root, mainly for places that are containerized
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    if unsafe { geteuid() } != 0 {
        error!("Please run this program using `sudo -E`!");
        exit(1);
    }

    // macOS don't need root
    #[cfg(any(target_os = "macos"))]
    if unsafe { geteuid() } == 0 {
        error!("Please run this program without root, and make sure your terminal has developer tool permissions and Full Disk Access!");
        exit(1);
    }

    arguments::arg_conflicts(&matches);
    arguments::arg_commands(&matches);
}
