use std::process::exit;
use crate::{error, info};
use vencord_installer_core::paths::{
    branch::DiscordLocation,
    locations::get_discord_locations
};
use console::{Key, Term};

pub fn select_location() -> DiscordLocation {
    let prompt = "Use ↑ ↓ to navigate, Enter to select, Esc to cancel.";
    
    info!("{}", prompt);
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
            tags.push("Vencord");
        }
        if location.openasar {
            tags.push("OpenAsar");
        }

        let tags_str = if tags.is_empty() { String::new() } else { format!(" + {}", tags.join(", ")) };

        format!(
            "{}{} – {}",
            instance.join(", "),
            tags_str,
            location.path.to_string()
        )
    }).collect();

    let term = Term::stderr();
    let mut index: usize = 0;

    loop {
        term.clear_screen().ok();
        println!("{}", prompt);
        println!();

        for (i, item) in items.iter().enumerate() {
            if i == index {
                println!("{} {}", "→", item);
            } else {
                println!("  {}", item);
            }
        }

        match term.read_key() {
            Ok(Key::ArrowUp) => {
                if index > 0 { index -= 1; }
            }
            Ok(Key::ArrowDown) => {
                if index + 1 < items.len() { index += 1; }
            }
            Ok(Key::Char('\n')) | Ok(Key::Enter) => {
                return locations[index].clone();
            }
            Ok(Key::Escape) => {
                error!("Selection cancelled.");
                exit(1);
            }
            _ => {}
        }
    }
}
