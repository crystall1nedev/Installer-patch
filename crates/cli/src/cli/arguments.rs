use std::process::exit;

use clap::{Arg, ArgMatches, Command};

use vencord_installer_shared::error;

use crate::cli::selections::select_options;

use super::commands::{install, uninstall};

pub fn args_build() -> Command {
    Command::new("vencord installer")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Vencord installer program")
        .arg(
            Arg::new("install")
                .long("install")
                .short('i')
                .help("Patches Discord with Vencord")
                .num_args(0),
        )
        .arg(
            Arg::new("uninstall")
                .long("uninstall")
                .short('u')
                .help("Unpatches Discord from Vencord")
                .num_args(0),
        )
        .arg(
            Arg::new("install-openasar")
                .long("install-openasar")
                .short('o')
                .help("Patches Discord with OpenAsar")
                .num_args(0),
        )
        .arg(
            Arg::new("uninstall-openasar")
                .long("uninstall-openasar")
                .short('x')
                .help("Unpatches Discord with OpenAsar")
                .num_args(0),
        )
        .arg(
            Arg::new("custom")
                .long("custom")
                .short('c')
                .help("Specify a custom Discord location")
                .value_name("PATH")
                .num_args(1),
        )
}

pub fn arg_conflicts(args: &ArgMatches) {
    let install = args.get_flag("install") || args.get_flag("install-openasar");
    let uninstall = args.get_flag("uninstall") || args.get_flag("uninstall-openasar");

    if install && uninstall {
        error!("You cannot use install and uninstall commands together.");
        exit(1);
    }

    if args.contains_id("custom") && !(install || uninstall) {
        error!("You must specify an install or uninstall when using --custom!");
        exit(1);
    }
}

pub fn arg_commands(args: &ArgMatches) {
    let custom_path = args.get_one::<String>("custom").cloned();

    let install_vencord = args.get_flag("install");
    let install_openasar = args.get_flag("install-openasar");

    let uninstall_vencord = args.get_flag("uninstall");
    let uninstall_openasar = args.get_flag("uninstall-openasar");

    if install_vencord || install_openasar {
        if let Err(err) = install(install_vencord, install_openasar, custom_path) {
            error!("{}", err);
            exit(1);
        }
        return;
    }

    if uninstall_vencord || uninstall_openasar {
        if let Err(err) = uninstall(uninstall_vencord, uninstall_openasar, custom_path) {
            error!("{}", err);
            exit(1);
        }
        return;
    }

    select_options();
}
