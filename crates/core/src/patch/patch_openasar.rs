use crate::Error;
#[cfg(target_os = "windows")]
use crate::paths::locations::is_scuffed_install;
use crate::paths::shared::{resource_dir_path, use_appropriate_asar};
use crate::paths::branch::DiscordLocation;
use crate::update::download::download_file;

pub struct OpenAsarInstaller {
    discord_location: DiscordLocation,
}
    
impl OpenAsarInstaller {

    pub fn new(
        discord_location: DiscordLocation
    ) -> Self {
        OpenAsarInstaller { 
            discord_location 
        }
    }

    // MARK: - Patch
    pub async fn patch(&self, patched_asar_file_url: &str) -> Result<(), Error> {
        if self.discord_location.openasar {
            log::error!("This Discord install is already patched, nothing to do.");
            return Err(Error::ErrLocationPatched);
        }

        #[cfg(target_os = "windows")]
        if is_scuffed_install(&self.discord_location.name) {
            log::error!("You have a broken Discord install. Please reinstall Discord!");
            return Err(Error::ErrWindowsMovedDirectory);
        }

        let resource_dir = resource_dir_path(&self.discord_location, self.discord_location.is_system_electron);
        let asar_path = resource_dir.join(use_appropriate_asar(self.discord_location.patched));
        let dl_tmp_asar_path = resource_dir.join("app.asar.tmp");

        log::info!("Patching {} using remote asar: {}", self.discord_location.path.as_str(), patched_asar_file_url);

        download_file(
            patched_asar_file_url, 
            dl_tmp_asar_path.clone()
        ).await?;

        tokio::fs::rename(&asar_path, resource_dir.join("app.asar.backup")).await?;
        tokio::fs::rename(&dl_tmp_asar_path, &asar_path).await?;

        log::info!("Patch applied successfully!");

        Ok(())
    }
    
    // MARK: - Unpatch
    pub async fn unpatch(&self) -> Result<(), Error> {
        if !self.discord_location.openasar {
            log::error!("This Discord install is not patched, nothing to do.");
            return Err(Error::ErrLocationNotPatched);
        }

        let resource_dir = resource_dir_path(&self.discord_location, self.discord_location.is_system_electron);
        let asar_path = resource_dir.join(use_appropriate_asar(self.discord_location.patched));

        log::info!("Unpatching {}", self.discord_location.path.as_str());

        tokio::fs::remove_file(&asar_path).await?;

        let backup_paths = [
            resource_dir.join("app.asar.backup"),
            resource_dir.join("app.asar.original")
        ];

        match backup_paths.iter().find(|&path| path.exists()) {
            Some(backup) => tokio::fs::rename(backup, asar_path).await?,
            None => return Err(Error::ErrLocationInvalid),
        }

        log::info!("Unpatch applied successfully!");

        Ok(())
    }
}
