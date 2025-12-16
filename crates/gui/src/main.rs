#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let app = app::VencordInstallerApp::new()?;
    app.run().map_err(Into::into)
}
