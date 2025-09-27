// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::error::Error;
use std::rc::Rc;

slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    let app = AppWindow::new()?;

    let installs = vec![
        DiscordInstall { branch: DiscordBranch::Stable, folder_path: "/some/example/path".into() },
        DiscordInstall { branch: DiscordBranch::PTB, folder_path: "/some/example/path".into() },
        DiscordInstall { branch: DiscordBranch::Canary, folder_path: "/some/example/path".into() },
    ];
    let installs_model = Rc::new(slint::VecModel::from(installs));

    app.global::<DiscordInstallAdapter>().set_installs(installs_model.clone().into());

    let callbacks = app.global::<RustCallbacks>();

    callbacks.on_open_settings(|| {
        let dialog = SettingsDialog::new().unwrap();

        dialog.on_ok_clicked(|| println!("OK clicked"));
        dialog.on_apply_clicked(|| println!("Apply clicked"));
        dialog.on_cancel_clicked(|| println!("Cancel clicked"));

        dialog.show().unwrap();
    });

    app.run()?;

    Ok(())
}
