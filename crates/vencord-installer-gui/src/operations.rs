use tokio::sync::mpsc;

use vencord_installer_core::{
    Error, OPENASAR_URL, download, get_dist_path, 
    patch::{
        patch_mod::Installer, 
        patch_openasar::OpenAsarInstaller
    },
    paths::branch::DiscordLocation
};

#[derive(Debug, Clone)]
pub enum AppOperation {
    Install(DiscordLocation),
    Uninstall(DiscordLocation),
    Repair(DiscordLocation),
    InstallOpenAsar(DiscordLocation),
    UninstallOpenAsar(DiscordLocation),
    OpenAppData,
}

#[derive(Debug, Clone)]
pub enum AppMessage {
    OperationSuccess,
    OperationError(String, bool),
}

pub struct AppActions {
    operation_rx: mpsc::UnboundedReceiver<AppOperation>,
    message_tx: mpsc::UnboundedSender<AppMessage>,
}

impl AppActions {
    pub fn new(
        operation_rx: mpsc::UnboundedReceiver<AppOperation>,
        message_tx: mpsc::UnboundedSender<AppMessage>,
    ) -> Self {
        Self {
            operation_rx,
            message_tx,
        }
    }

    pub async fn run(mut self) {
        while let Some(operation) = self.operation_rx.recv().await {
            let result = self.handle_operation(operation).await;
            
            let message = match result {
                Ok(()) => AppMessage::OperationSuccess,
                Err(err) => AppMessage::OperationError(
                    err.format_error(),
                    matches!(err, Error::ErrWindowsMovedDirectory)
                ),
            };
            
            let _ = self.message_tx.send(message);
        }
    }

    async fn handle_operation(&self, operation: AppOperation) -> Result<(), Error> {
        match operation {
            AppOperation::Install(location) => Self::install(location).await,
            AppOperation::Uninstall(location) => Self::uninstall(location).await,
            AppOperation::Repair(location) => Self::repair(location).await,
            AppOperation::InstallOpenAsar(location) => Self::install_openasar(location).await,
            AppOperation::UninstallOpenAsar(location) => Self::uninstall_openasar(location).await,
            AppOperation::OpenAppData => Self::open_appdata().await,
        }
    }

    async fn install(location: DiscordLocation) -> Result<(), Error> {
        if location.patched {
            return Err(Error::ErrLocationPatched);
        }
        
        if std::env::var("VENCORD_DEV_INSTALL").map_or(true, |v| v != "1") {
            download().await?;
        }

        let installer = Installer::new(location.clone(), Some(get_dist_path(None)));
        installer.patch().await?;

        #[cfg(target_os = "linux")]
        if location.is_flatpak {
            installer.grant_flatpak_permissions(location, &get_dist_path(None).to_string_lossy())?;
        }

        Ok(())
    }
    
    async fn uninstall(location: DiscordLocation) -> Result<(), Error> {
        if !location.patched {
            return Err(Error::ErrLocationNotPatched);
        }

        Installer::new(location, None).unpatch().await
    }

    async fn repair(location: DiscordLocation) -> Result<(), Error> {
        if location.patched {
            Self::uninstall(location.clone()).await?;
        }
        Self::install(location).await
    }

    async fn install_openasar(location: DiscordLocation) -> Result<(), Error> {
        if location.openasar {
            return Err(Error::ErrLocationPatched);
        }

        OpenAsarInstaller::new(location).patch(OPENASAR_URL).await
    }

    async fn uninstall_openasar(location: DiscordLocation) -> Result<(), Error> {
        if !location.openasar {
            return Err(Error::ErrLocationNotPatched);
        }

        OpenAsarInstaller::new(location).unpatch().await
    }

    async fn open_appdata() -> Result<(), Error> {
        #[cfg(target_os = "windows")]
        {
            use std::process::Command;
            Command::new("explorer")
                .arg("%APPDATA%")
                .spawn()
                .map_err(|e| Error::ErrIo(e))?;
        }

        Ok(())
    }
    
}
