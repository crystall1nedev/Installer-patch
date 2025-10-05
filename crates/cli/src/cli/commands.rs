use std::env;

use vencord_installer_core::Error;
use vencord_installer_core::paths::{branch::DiscordLocation, shared::get_custom_discord_location};
use vencord_installer_core::patch::{patch_mod::Installer, patch_openasar::OpenAsarInstaller};

use vencord_installer_shared::{OPENASAR_URL, USER_AGENT};
use vencord_installer_shared::{info, success, warn};
use vencord_installer_shared::{download_assets, get_dist_path};

use tokio::runtime::Runtime;

use super::selections::select_location;

// MARK: - Install

pub fn install(client_mod: bool, openasar: bool, custom_path: Option<String>) -> Result<(), Error> {
    let mut selected_location: DiscordLocation;

    let rt = Runtime::new().unwrap();
    
    if let Some(path) = custom_path {
        selected_location = match get_custom_discord_location(&path) {
            Some(location) => location,
            None => return Err(Error::ErrLocationInvalid),
        };
    } else {
        selected_location = select_location();
    }

    info!("You selected {:?}, attempting to patch!", selected_location.path);
    info!("Using this path for dist: {}", get_dist_path().display());

    if client_mod && !selected_location.patched {
        // user may forget to set this variable..
        if env::var("VENCORD_DEV_INSTALL").map_or(true, |v| v != "1") {
            download_assets()?;
        }


        let installer = Installer::new(selected_location.clone());

        rt.block_on(installer.write_app_asar(
            &get_dist_path().join("app.asar").to_string_lossy(), 
            &get_dist_path().join("patcher.js").to_string_lossy()
        ))?;

        rt.block_on(installer.patch(
            &get_dist_path().join("app.asar").to_string_lossy()
        ))?;

        #[cfg(target_os = "linux")]
        if selected_location.is_flatpak {
            installer.grant_flatpak_permissions(selected_location.clone(), &get_dist_path().to_string_lossy())?;
        }

        selected_location.patched = true;

        success!("Successfully patched Discord!");
    } else if client_mod {
        warn!("Discord is already patched with Vencord, skipping!");
    }

    if openasar && !selected_location.openasar {
        let installer = OpenAsarInstaller::new(selected_location.clone());

        rt.block_on(installer.patch(OPENASAR_URL, USER_AGENT))?;

        success!("Successfully patched Discord with OpenAsar!");
    } else if openasar {
        warn!("Discord is already patched with OpenAsar, skipping!");
    }

    Ok(())
}

// MARK: - Uninstall

pub fn uninstall(client_mod: bool, openasar: bool, custom_path: Option<String>) -> Result<(), Error> {
    let mut selected_location: DiscordLocation;

    let rt = Runtime::new().unwrap();

    if let Some(path) = custom_path {
        selected_location = match get_custom_discord_location(&path) {
            Some(location) => location,
            None => return Err(Error::ErrLocationInvalid),
        };
    } else {
        selected_location = select_location();
    }

    info!("You selected {:?}, attempting to unpatch...", selected_location.path);

    if client_mod && selected_location.patched {
        let installer = Installer::new(selected_location.clone());
        rt.block_on(installer.unpatch())?;

        selected_location.patched = false;

        success!("Successfully unpatched Discord!");
    } else if client_mod {
        warn!("Discord is not patched with Vencord, skipping!");
    }

    if openasar && selected_location.openasar  {
        let installer = OpenAsarInstaller::new(selected_location.clone());
        rt.block_on(installer.unpatch())?;

        success!("Successfully unpatched Discord with OpenAsar!");
    } else if openasar {
        warn!("Discord is not patched with OpenAsar, skipping!");
    }

    Ok(())
}
