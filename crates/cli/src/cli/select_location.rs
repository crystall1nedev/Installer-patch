use std::process::exit;
use colored::Colorize;
use logger_rs::{error, info};
use vencord_installer_core::paths::{
    branch::{DiscordBranch, DiscordLocation},
    locations::get_discord_locations
};
use console::{Key, Term};

fn make_colored_branch_string(branch: &DiscordBranch) -> String {
    match branch {
        DiscordBranch::Stable => "Stable".truecolor(93, 107, 243).bold().to_string(),
        DiscordBranch::PTB => "PTB".truecolor(67, 150, 226).bold().to_string(),
        DiscordBranch::Canary => "Canary".truecolor(251, 183, 71).bold().to_string(),
        DiscordBranch::Development => "Development".bold().to_string()
    }
}

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
        instance.push(make_colored_branch_string(&location.branch));
        if location.is_flatpak {
            instance.push("Flatpak".white().bold().to_string());
        }

        let mut tags = Vec::new();
        if location.patched {
            tags.push("Vencord".truecolor(255, 192, 203).bold().to_string());
        }
        if location.openasar {
            tags.push("OpenAsar".white().bold().to_string());
        }

        let tags_str = if tags.is_empty() { String::new() } else { format!(" + {}", tags.join(", ")) };

        format!(
            "{}{} – {}",
            instance.join(", "),
            tags_str,
            location.path.to_string().truecolor(168, 168, 168)
        )
    }).collect();

    let term = Term::stderr();
    let mut index: usize = 0;

    loop {
        term.clear_screen().ok();
        println!("{}", prompt.bold());
        println!();

        for (i, item) in items.iter().enumerate() {
            if i == index {
                println!("{} {}", "→".green().bold(), item);
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
