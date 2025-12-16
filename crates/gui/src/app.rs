use std::{env, error::Error};
use std::rc::Rc;

use vencord_installer_core::Error as CoreError;
use vencord_installer_core::patch::patch_openasar::OpenAsarInstaller;
use vencord_installer_core::update::version_check::{check_hash_from_release, check_local_version};
use vencord_installer_core::{
    paths::branch::{DiscordLocation as CoreDiscordLocation, DiscordBranch as CoreDiscordBranch},
    paths::locations::get_discord_locations,
    patch::patch_mod::Installer,
};
use vencord_installer_shared::{OPENASAR_URL, RELEASE_URL, RELEASE_URL_FALLBACK, USER_AGENT, download_assets, get_dist_path};

use tokio::runtime::Runtime;

slint::include_modules!();

pub struct VencordInstallerApp {
    app: AppWindow,
    app_weak: slint::Weak<AppWindow>,
}

impl VencordInstallerApp {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let app = AppWindow::new()?;
        let app_weak = app.as_weak();
        
        let mut gui_app = Self {
            app,
            app_weak,
        };
        
        gui_app.initialize()?;
        Ok(gui_app)
    }
    
    pub fn run(self) -> Result<(), slint::PlatformError> {
        self.app.run()
    }

    fn initialize(&mut self) -> Result<(), Box<dyn Error>> {
        self.app.global::<AppInfo>().set_version(env!("CARGO_PKG_VERSION").into());
        
        let app_weak = self.app_weak.clone();
        std::thread::spawn(move || {
            let rt = Runtime::new().unwrap();
            if let Some(remote_hash) = rt.block_on(check_hash_from_release(RELEASE_URL, Some(RELEASE_URL_FALLBACK), USER_AGENT)) {
                let app_weak_clone = app_weak.clone();
                let hash = remote_hash.clone();
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(app) = app_weak_clone.upgrade() {
                    app.global::<AppInfo>().set_remote_vc_version(hash.into());
                    }
                });
            }
            
            if let Some(local_hash) = rt.block_on(check_local_version(&get_dist_path(), r"// Vencord ([0-9a-zA-Z\.-]+)")) {
                let app_weak_clone = app_weak.clone();
                let hash = local_hash.clone();
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(app) = app_weak_clone.upgrade() {
                    app.global::<AppInfo>().set_local_vc_version(hash.into());
                    }
                });
            }
        });
        
        self.setup_callbacks();
        self.refresh_discord_locations();
        Ok(())
    }

    fn setup_callbacks(&self) {
        let callbacks = self.app.global::<RustCallbacks>();
        
        let app_weak_refresh = self.app_weak.clone();
        callbacks.on_refresh_locations(move || {
            if let Some(app) = app_weak_refresh.upgrade() {
                Self::refresh_locations_static(&app);
            }
        });
        
        let app_weak_install = self.app_weak.clone();
        callbacks.on_do_install(move |location| {
            Self::handle_install(location, app_weak_install.clone());
        });

        let app_weak_o_install = self.app_weak.clone();
        callbacks.on_do_o_install(move |location| {
            Self::handle_o_install(location, app_weak_o_install.clone());
        });
        
        let app_weak_uninstall = self.app_weak.clone();
        callbacks.on_do_uninstall(move |location| {
            Self::handle_uninstall(location, app_weak_uninstall.clone());
        });

        let app_weak_o_uninstall = self.app_weak.clone();
        callbacks.on_do_o_uninstall(move |location| {
            Self::handle_o_uninstall(location, app_weak_o_uninstall.clone());
        });

        let app_weak_repair = self.app_weak.clone();
        callbacks.on_do_repair(move |location| {
            Self::handle_repair(location, app_weak_repair.clone());
        });
    }
    
    fn refresh_discord_locations(&self) {
        if let Some(core_locations) = get_discord_locations() {
            let locations: Vec<DiscordLocation> = core_locations.iter().map(Into::into).collect();
            let locations_model = Rc::new(slint::VecModel::from(locations));
            self.app.global::<DiscordLocationAdapter>().set_locations(locations_model.into());
        }
        self.app.global::<PageManager>().set_current_page_index(0);
    }

    fn refresh_locations_static(app: &AppWindow) {
        if let Some(core_locations) = get_discord_locations() {
            let locations: Vec<DiscordLocation> = core_locations.iter().map(Into::into).collect();
            let locations_model = Rc::new(slint::VecModel::from(locations));
            app.global::<DiscordLocationAdapter>().set_locations(locations_model.into());
        }
        app.global::<PageManager>().set_current_page_index(0);
    }
    
    // MARK: Handlers

    fn handle_install(location: DiscordLocation, app_weak: slint::Weak<AppWindow>) {
        let core_location: CoreDiscordLocation = (&location).into();
        if core_location.patched { return; }
        
        let _ = slint::spawn_local(async move {
            match Self::install(core_location) {
                Ok(()) => {
                    if let Some(app) = app_weak.upgrade() {
                        Self::refresh_locations_static(&app);
                        app.global::<PageManager>().set_current_page_index(0);
                    }
                }
                Err(err) => eprintln!("Installation failed: {err}"),
            }
        });
    }

    fn handle_o_install(location: DiscordLocation, app_weak: slint::Weak<AppWindow>) {
        let core_location: CoreDiscordLocation = (&location).into();
        if core_location.openasar { return; }

        let _ = slint::spawn_local(async move {
            match Self::o_install(core_location) {
                Ok(()) => {
                    if let Some(app) = app_weak.upgrade() {
                        Self::refresh_locations_static(&app);
                        app.global::<PageManager>().set_current_page_index(0);
                    }
                }
                Err(err) => eprintln!("Installation failed: {err}"),
            }
        });
    }

    fn handle_uninstall(location: DiscordLocation, app_weak: slint::Weak<AppWindow>) {
        let core_location: CoreDiscordLocation = (&location).into();

        if !core_location.patched { return; }

        let _ = slint::spawn_local(async move {
            match Self::uninstall(core_location) {
                Ok(()) => {
                    if let Some(app) = app_weak.upgrade() {
                        Self::refresh_locations_static(&app);
                    }
                }
                Err(err) => eprintln!("Uninstallation failed: {err}"),
            }
        });
    }

    fn handle_o_uninstall(location: DiscordLocation, app_weak: slint::Weak<AppWindow>) {
        let core_location: CoreDiscordLocation = (&location).into();

        if !core_location.openasar { return; }

        let _ = slint::spawn_local(async move {
            match Self::o_uninstall(core_location) {
                Ok(()) => {
                    if let Some(app) = app_weak.upgrade() {
                        Self::refresh_locations_static(&app);
                    }
                }
                Err(err) => eprintln!("Uninstallation failed: {err}"),
            }
        });
    }

    fn handle_repair(location: DiscordLocation, app_weak: slint::Weak<AppWindow>) {
        if location.patched {
            Self::handle_uninstall(location.clone(), app_weak.clone());
        }

        Self::handle_install(location, app_weak);
    }
}

impl VencordInstallerApp {
    pub fn install(discord_location: CoreDiscordLocation) -> Result<(), CoreError> {
        let rt = Runtime::new().unwrap();
        
        if discord_location.patched {
            return Err(CoreError::ErrLocationPatched);
        }
        
        if env::var("VENCORD_DEV_INSTALL").map_or(true, |v| v != "1") {
            download_assets()?;
        }

        let installer = Installer::new(discord_location.clone());
        
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

        Ok(())
    }

    pub fn o_install(discord_location: CoreDiscordLocation) -> Result<(), CoreError> {
        let rt = Runtime::new().unwrap();

        if discord_location.openasar {
            return Err(CoreError::ErrLocationPatched);
        }

        let installer = OpenAsarInstaller::new(discord_location);
        rt.block_on(installer.patch(
            OPENASAR_URL, USER_AGENT
        ))?;

        Ok(())
    }
    
    pub fn uninstall(discord_location: CoreDiscordLocation) -> Result<(), CoreError> {
        let rt = Runtime::new().unwrap();
        
        if !discord_location.patched {
            return Err(CoreError::ErrLocationNotPatched);
        }
        
        let installer = Installer::new(discord_location.clone());
        rt.block_on(installer.unpatch())?;
        
        Ok(())
    }

    pub fn o_uninstall(discord_location: CoreDiscordLocation) -> Result<(), CoreError> {
        let rt = Runtime::new().unwrap();
        
        if !discord_location.openasar {
            return Err(CoreError::ErrLocationNotPatched);
        }

        let installer = OpenAsarInstaller::new(discord_location);
        rt.block_on(installer.unpatch())?;
        
        Ok(())
    }
}

// MARK: - Conversions

impl From<&CoreDiscordLocation> for DiscordLocation {
    fn from(core: &CoreDiscordLocation) -> Self {
        let branch = match core.branch {
            CoreDiscordBranch::Stable => DiscordBranch::Stable,
            CoreDiscordBranch::PTB => DiscordBranch::PTB,
            CoreDiscordBranch::Canary => DiscordBranch::Canary,
            CoreDiscordBranch::Development => DiscordBranch::Development,
        };
        
        Self {
            name: core.name.clone().into(),
            path: core.path.clone().into(),
            branch,
            patched: core.patched,
            openasar: core.openasar,
            is_flatpak: core.is_flatpak,
            is_system_electron: core.is_system_electron,
        }
    }
}

impl From<&DiscordLocation> for CoreDiscordLocation {
    fn from(slint_location: &DiscordLocation) -> Self {
        let branch = match slint_location.branch {
            DiscordBranch::Stable => CoreDiscordBranch::Stable,
            DiscordBranch::PTB => CoreDiscordBranch::PTB,
            DiscordBranch::Canary => CoreDiscordBranch::Canary,
            DiscordBranch::Development => CoreDiscordBranch::Development,
        };

        Self {
            name: slint_location.name.to_string(),
            path: slint_location.path.to_string(),
            branch,
            patched: slint_location.patched,
            openasar: slint_location.openasar,
            is_flatpak: slint_location.is_flatpak,
            is_system_electron: slint_location.is_system_electron,
        }
    }
}
