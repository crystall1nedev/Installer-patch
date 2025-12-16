use std::{error::Error, rc::Rc};
use tokio::sync::mpsc;

use vencord_installer_core::{
    RELEASE_URL, 
    RELEASE_URL_FALLBACK,
    get_dist_path,
    paths::{
        branch::{DiscordBranch as CoreDiscordBranch, DiscordLocation as CoreDiscordLocation},
        locations::get_discord_locations
    }, 
    update::version_check::{check_hash_from_release, check_local_version}
};

use crate::operations::{AppActions, AppMessage, AppOperation};

slint::include_modules!();

pub struct VencordInstallerApp {
    app: AppWindow,
    app_weak: slint::Weak<AppWindow>,
    operation_tx: mpsc::UnboundedSender<AppOperation>,
    message_rx: mpsc::UnboundedReceiver<AppMessage>,
}

impl VencordInstallerApp {
    pub async fn new() -> Result<Self, Box<dyn Error>> {
        let app = AppWindow::new()?;
        let app_weak = app.as_weak();
        
        let (operation_tx, operation_rx) = mpsc::unbounded_channel();
        let (message_tx, message_rx) = mpsc::unbounded_channel();
        
        let actions = AppActions::new(operation_rx, message_tx);
        tokio::spawn(actions.run());
        
        let mut gui_app = Self {
            app,
            app_weak: app_weak.clone(),
            operation_tx,
            message_rx,
        };
        
        gui_app.initialize().await?;
        gui_app.start_message_handler();
        Ok(gui_app)
    }
    
    pub fn run(self) -> Result<(), slint::PlatformError> {
        self.app.run()
    }

    async fn initialize(&mut self) -> Result<(), Box<dyn Error>> {
        self.app.global::<AppInfo>().set_version(env!("CARGO_PKG_VERSION").into());
        
        self.check_versions().await;
        self.setup_callbacks();
        self.refresh_discord_locations();
        Ok(())
    }

    async fn check_versions(&self) {
        if let Some(remote_hash) = check_hash_from_release(RELEASE_URL, Some(RELEASE_URL_FALLBACK)).await {
            self.update_ui(move |app| {
                app.global::<AppInfo>().set_remote_vc_version(remote_hash.into());
            });
        }
        
        if let Some(local_hash) = check_local_version(&get_dist_path(None), None).await {
            self.update_ui(move |app| {
                app.global::<AppInfo>().set_local_vc_version(local_hash.into());
            });
        }
    }

    fn update_ui<F>(&self, f: F) 
    where 
        F: FnOnce(&AppWindow) + Send + 'static 
    {
        Self::invoke_ui_update(self.app_weak.clone(), f);
    }

    fn setup_callbacks(&self) {
        let callbacks = self.app.global::<RustCallbacks>();
        
        let app_weak = self.app_weak.clone();
        callbacks.on_refresh_locations(move || {
            if let Some(app) = app_weak.upgrade() {
                Self::refresh_locations(&app);
            }
        });
        
        let tx_install = self.operation_tx.clone();
        callbacks.on_do_install(move |location| {
            let loc: CoreDiscordLocation = (&location).into();
            if !loc.patched {
                tx_install.send(AppOperation::Install(loc)).ok();
            }
        });

        let tx_uninstall = self.operation_tx.clone();
        callbacks.on_do_uninstall(move |location| {
            let loc: CoreDiscordLocation = (&location).into();
            if loc.patched {
                tx_uninstall.send(AppOperation::Uninstall(loc)).ok();
            }
        });
        
        let tx_o_install = self.operation_tx.clone();
        callbacks.on_do_o_install(move |location| {
            let loc: CoreDiscordLocation = (&location).into();
            if !loc.openasar {
                tx_o_install.send(AppOperation::InstallOpenAsar(loc)).ok();
            }
        });

        let tx_o_uninstall = self.operation_tx.clone();
        callbacks.on_do_o_uninstall(move |location| {
            let loc: CoreDiscordLocation = (&location).into();
            if loc.openasar {
                tx_o_uninstall.send(AppOperation::UninstallOpenAsar(loc)).ok();
            }
        });
        
        let tx_repair = self.operation_tx.clone();
        callbacks.on_do_repair(move |location| {
            tx_repair.send(AppOperation::Repair((&location).into())).ok();
        });

        #[cfg(target_os = "windows")]
        {
            let tx_open_appdata = self.operation_tx.clone();
            callbacks.on_do_open_appdata(move || {
                tx_open_appdata.send(AppOperation::OpenAppData()).ok();
            });
        }
    }
    
    fn refresh_discord_locations(&self) {
        Self::refresh_locations(&self.app);
    }

    fn refresh_locations(app: &AppWindow) {
        if let Some(core_locations) = get_discord_locations() {
            let locations: Vec<DiscordLocation> = core_locations.iter().map(Into::into).collect();
            let locations_model = Rc::new(slint::VecModel::from(locations));
            app.global::<DiscordLocationAdapter>().set_locations(locations_model.into());
        }
        app.global::<PageManager>().set_current_page_index(0);
    }

    fn start_message_handler(&mut self) {
        let app_weak = self.app_weak.clone();
        let mut message_rx = std::mem::replace(
            &mut self.message_rx,
            mpsc::unbounded_channel().1
        );
        
        tokio::spawn(async move {
            while let Some(message) = message_rx.recv().await {
                Self::handle_message(message, &app_weak);
            }
        });
    }
    
    fn handle_message(message: AppMessage, app_weak: &slint::Weak<AppWindow>) {
        match message {
            AppMessage::OperationSuccess => {
                Self::invoke_ui_update(app_weak.clone(), |app| {
                    Self::refresh_locations(app);
                });
            }
            AppMessage::OperationError(error, show_open_appdata) => {
                Self::show_error_dialog(app_weak.clone(), error, show_open_appdata);
            }
        }
    }
    
    fn invoke_ui_update<F>(app_weak: slint::Weak<AppWindow>, f: F)
    where
        F: FnOnce(&AppWindow) + Send + 'static,
    {
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(app) = app_weak.upgrade() {
                f(&app);
            }
        });
    }
    
    fn show_error_dialog(app_weak: slint::Weak<AppWindow>, error: String, show_open_appdata: bool) {
        Self::invoke_ui_update(app_weak, move |app| {
            app.global::<ErrorDialog>().set_message(error.into());
            app.global::<ErrorDialog>().set_visible(true);
            app.global::<ErrorDialog>().set_open_appdata(show_open_appdata);
        });
    }
}

// MARK: - Type Conversions
impl From<&CoreDiscordLocation> for DiscordLocation {
    fn from(core: &CoreDiscordLocation) -> Self {
        Self {
            name: core.name.clone().into(),
            path: core.path.clone().into(),
            branch: convert_branch_to_slint(&core.branch),
            patched: core.patched,
            openasar: core.openasar,
            is_flatpak: core.is_flatpak,
            is_system_electron: core.is_system_electron,
        }
    }
}

impl From<&DiscordLocation> for CoreDiscordLocation {
    fn from(slint_location: &DiscordLocation) -> Self {
        Self {
            name: slint_location.name.to_string(),
            path: slint_location.path.to_string(),
            branch: convert_branch_to_core(&slint_location.branch),
            patched: slint_location.patched,
            openasar: slint_location.openasar,
            is_flatpak: slint_location.is_flatpak,
            is_system_electron: slint_location.is_system_electron,
        }
    }
}

fn convert_branch_to_slint(core_branch: &CoreDiscordBranch) -> DiscordBranch {
    match core_branch {
        CoreDiscordBranch::Stable => DiscordBranch::Stable,
        CoreDiscordBranch::PTB => DiscordBranch::PTB,
        CoreDiscordBranch::Canary => DiscordBranch::Canary,
        CoreDiscordBranch::Development => DiscordBranch::Development,
    }
}

fn convert_branch_to_core(slint_branch: &DiscordBranch) -> CoreDiscordBranch {
    match slint_branch {
        DiscordBranch::Stable => CoreDiscordBranch::Stable,
        DiscordBranch::PTB => CoreDiscordBranch::PTB,
        DiscordBranch::Canary => CoreDiscordBranch::Canary,
        DiscordBranch::Development => CoreDiscordBranch::Development,
    }
}
