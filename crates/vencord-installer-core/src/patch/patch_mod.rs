#[cfg(feature = "generate_asar")]
use {
    std::collections::HashMap,
    serde::Serialize,
    tokio::{fs::File, io::AsyncWriteExt}
};
use std::path::PathBuf;
#[cfg(target_os = "linux")]
use std::{env, process::Command};

use crate::Error;
#[cfg(target_os = "windows")]
use crate::paths::locations::is_scuffed_install;
use crate::paths::shared::resource_dir_path;
use crate::paths::branch::DiscordLocation;

#[cfg(target_os = "linux")]
unsafe extern "C" {
    unsafe fn geteuid() -> u32;
}

pub struct Installer {
    discord_location: DiscordLocation,
    data_path: Option<PathBuf>,
}

impl Installer {
    pub fn new(
        discord_location: DiscordLocation, 
        data_path: Option<PathBuf>
    ) -> Self {
        Installer {
            discord_location,
            data_path
        }
    }
    
    // MARK: - Patch
    pub async fn patch(&self) -> Result<(), Error> {
        if self.discord_location.patched {
            log::error!("This Discord install is already patched, nothing to do.");
            return Err(Error::ErrLocationPatched);
        }

        let data_path = &self.data_path.clone().ok_or(Error::ErrNoDataPath)?;

        #[cfg(target_os = "windows")]
        if is_scuffed_install(&self.discord_location.name) {
            log::error!("You have a broken Discord install. Please reinstall Discord!");
            return Err(Error::ErrWindowsMovedDirectory);
        }

        self.write_app_asar().await?;

        let resource_dir = resource_dir_path(&self.discord_location, self.discord_location.is_system_electron);
        let asar_path = resource_dir.join("app.asar");
        let _asar_path = resource_dir.join("_app.asar");

        log::info!("Patching {} using custom asar: {:?}", self.discord_location.path.as_str(), data_path.join("app.asar"));

        super::rename(&asar_path, &_asar_path).await?;
        super::copy(&data_path.join("app.asar"), &asar_path).await?;

        #[cfg(target_os = "linux")]
        if self.discord_location.is_system_electron {
            let asar_path = resource_dir.join("app.asar.unpacked");
            let _asar_path = resource_dir.join("_app.asar.unpacked");

            super::rename(&asar_path, &_asar_path).await?;
        }

        #[cfg(target_os = "linux")]
        if self.discord_location.is_flatpak {
            self.grant_flatpak_permissions()?;
        }

        log::info!("Patch applied successfully!");

        Ok(())
    }

    // MARK: - Unpatch
    pub async fn unpatch(&self) -> Result<(), Error> {
        if !self.discord_location.patched {
            log::error!("This Discord install is not patched, nothing to do.");
            return Err(Error::ErrLocationNotPatched);
        }

        let resource_dir = resource_dir_path(&self.discord_location, self.discord_location.is_system_electron);
        let asar_path = resource_dir.join("app.asar");
        let _asar_path = resource_dir.join("_app.asar");

        log::info!("Unpatching {}...", self.discord_location.path.as_str());

        if asar_path.exists() {
            super::remove_file(&asar_path).await?;
        }
        super::rename(&_asar_path, &asar_path).await?;

        #[cfg(target_os = "linux")]
        if self.discord_location.is_system_electron {
            let asar_path = resource_dir.join("app.asar.unpacked");
            let _asar_path = resource_dir.join("_app.asar.unpacked");

            super::rename(&_asar_path, &asar_path).await?;
        }

        log::info!("Unpatch applied successfully!");

        Ok(())
    }
}

#[cfg(feature = "generate_asar")]
#[derive(Serialize)]
struct AsarEntry {
    size: i32,
    offset: String,
}

impl Installer {
    #[cfg(feature = "generate_asar")]
    pub async fn write_app_asar(&self) -> Result<(), Error> {
        let data_path = &self.data_path.clone().ok_or(Error::ErrNoDataPath)?;
        let index_js = format!("require({})", serde_json::to_string(&data_path.join("patcher.js"))?);
        let pkg_json = "{ \"name\": \"discord\", \"main\": \"index.js\" }";
        
        let mut files = HashMap::new();
        
        files.insert("index.js".to_string(), AsarEntry {
            size: index_js.len() as i32,
            offset: "0".to_string(),
        });

        files.insert("package.json".to_string(), AsarEntry {
            size: pkg_json.len() as i32,
            offset: index_js.len().to_string(),
        });

        let header = serde_json::to_string(&HashMap::from([("files".to_string(), files)]))?;
        let aligned_size = (header.len() as u32 + 3) & !3;

        let mut file = File::create(data_path.join("app.asar")).await?;

        for size in [4u32, aligned_size + 8, aligned_size + 4, header.len() as u32] {
            file.write_all(&(size as i32).to_le_bytes()).await?;
        }

        file.write_all(format!("{:<width$}", header, width = aligned_size as usize).as_bytes()).await?;
        file.write_all(index_js.as_bytes()).await?;
        file.write_all(pkg_json.as_bytes()).await?;

        log::debug!("Generated app.asar at {:?}", data_path.join("app.asar"));

        Ok(())
    }
    
    #[cfg(target_os = "linux")]
    pub fn grant_flatpak_permissions(&self) -> Result<(), Error> {
        let data_path = self.data_path.clone().ok_or(Error::ErrNoDataPath)?;

        log::info!("Location is flatpak, granting perms to {}", data_path.to_string_lossy());

        let name = self.discord_location.path
            .split('/')
            .find(|s| s.starts_with("com.discordapp."))
            .unwrap_or("");

        let is_system_flatpak = self.discord_location.path.contains("/var");

        let mut args = vec![];

        if !is_system_flatpak {
            args.push("--user");
        }
        args.push("override");
        args.push(name);
        let filesystem_arg = format!("--filesystem={}", &data_path.to_string_lossy());
        args.push(&filesystem_arg);
        let full_cmd = format!("flatpak {}", args.join(" "));

        if !is_system_flatpak && unsafe { geteuid() } == 0 {
            Command::new("sudo")
                .arg("-u")
                .arg(env::var("SUDO_USER").unwrap())
                .arg("sh")
                .arg("-c")
                .arg(&full_cmd)
                .output()?;
        } else {
            Command::new("sh")
                .arg("-c")
                .arg(&full_cmd)
                .output()?;
        };

        Ok(())
    }

    #[cfg(target_os = "linux")]
    pub fn fix_permissions(&self) {
        todo!();
    }
}
