use std::process::exit;

use vencord_installer_core::paths::{
    branch::DiscordLocation,
    locations::get_discord_locations
};

use console::style;
use dialoguer::Select;

use crate::{info, error};

use super::commands::{install, uninstall};

pub fn select_location() -> DiscordLocation {
    let locations = get_discord_locations().unwrap_or_default();
    if locations.is_empty() {
        error!("No matching Discord locations found!");
        exit(1);
    }

    let items: Vec<String> = locations.iter().map(|location| {
        let mut instance = Vec::new();
        instance.push(location.branch.to_string());
        if location.is_flatpak {
            instance.push("Flatpak".to_owned());
        }

        let mut tags = Vec::new();
        if location.patched {
            tags.push("[PATCHED]");
        }
        if location.openasar {
            tags.push("[OPENASAR]");
        }

        let tags_str = if tags.is_empty() { String::new() } else { format!("{}", tags.join(" ")) };

        format!(
            "{} {} – {}",
            instance.join(", "),
            tags_str,
            location.path.to_string()
        )
    }).collect();

    let selection = Select::new()
        .with_prompt(style("Use ↑ ↓ and Enter to select a Discord location").bold().to_string())
        .items(&items)
        .default(0)
        .interact();

    match selection {
        Ok(idx) => locations[idx].clone(),
        Err(_) => {
            error!("Selection cancelled.");
            exit(1);
        }
    }
}

pub fn select_options() {
    let options = [
        "Install Vencord",
        "Uninstall Vencord",
        "Install OpenAsar",
        "Uninstall OpenAsar",
        "Exit",
    ];

    let selection = Select::new()
        .with_prompt(style("Use ↑ ↓ and Enter to select an option").bold().to_string())
        .items(&options)
        .default(0)
        .interact();

    let Ok(choice) = selection else {
        error!("Failed to read selection");
        exit(1);
    };

    match choice {
        0 => {
            if let Err(err) = install(true, false, None) {
            error!("{}", err);
            exit(1);
            }
        }
        1 => {
            if let Err(err) = uninstall(true, false, None) {
                error!("{}", err);
                exit(1);
            }
        }
        2 => {
            if let Err(err) = install(false, true, None) {  
                error!("{}", err);
                exit(1);
            }
        }
        3 => {
            if let Err(err) = uninstall(false, true, None) {  
                error!("{}", err);
                exit(1);
            }
        }
        _ => {
            info!("Exiting.");
        }
    }
}
