use clap::{Args, Parser, Subcommand};
use upm::UpmBackend;

#[derive(Debug, Parser)]
#[command(version, about)]
struct UpmArgs {
    #[command(subcommand)]
    mode: Option<ActionMode>,

    #[arg(long, default_value_t = false, hide = true)]
    privilege: bool,

    #[arg(long, hide = true)]
    fifo: Option<String>,
}

#[derive(Debug, Subcommand)]
enum ActionMode {
    Update,
    Install(InstallMode),
    Uninstall(UninstallMode),

    #[command(hide = true)]
    UpdateApt,
    #[command(hide = true)]
    UpdateBrew,
    #[command(hide = true)]
    UpdateFlatpak,

    #[command(hide = true)]
    OutdateApt,
    #[command(hide = true)]
    OutdateBrew,
    #[command(hide = true)]
    OutdateFlatpak,
}

#[derive(Debug, Args)]
struct InstallMode {
    name: String,
}

#[derive(Debug, Args)]
struct UninstallMode {
    name: String,
}

// fn spawn_privilege() -> std::io::Result<()> {
//     let uid = get_uid()?;
//     if uid == 0 {
//         return Ok(());
//     }
//     // re-run this process by spawning a child process with root privilege.
//     let path = std::env::current_exe()?;
//     let cmd = std::process::Command::new("sudo")
//         .args(path.to_str())
//         .status()?;
//     println!("{}", cmd);
//     Ok(())
// }

// fn start_privilege_daemon() -> std::io::Result<(std::process::Child, std::fs::File)> {
//     let tmp_dir = tempfile::tempdir().unwrap();
//     let fifo_path = tmp_dir.path().join("upm.fifo");
//     match nix::unistd::mkfifo(&fifo_path, nix::sys::stat::Mode::S_IRWXU) {
//         Ok(_) => {}
//         Err(e) => {
//             return Err(std::io::Error::new(
//                 std::io::ErrorKind::Other,
//                 format!("failed to create fifo: {}", e),
//             ));
//         }
//     }
//     let exec_path = std::env::current_exe()?;
//     let cmd = std::process::Command::new("sudo")
//         .args(exec_path.to_str())
//         .args(&["--privilege", "--fifo", fifo_path.to_str().unwrap()])
//         .spawn()?;
//     let file = std::fs::File::open(fifo_path)?;
//     Ok((cmd, file))
// }

fn list_package(pkg: &Vec<upm::OutdateItem>) -> std::io::Result<()> {
    for item in pkg {
        println!(
            "{}: {} -> {}",
            item.name, item.current_version, item.target_version
        );
    }

    Ok(())
}

fn main() {
    let args = UpmArgs::parse();

    // if args.privilege {
    //     if uid != 0 {
    //         eprintln!("This command requires root privilege.");
    //         std::process::exit(1);
    //     }
    //     start_privilege_daemon().unwrap();
    //     std::process::exit(0);
    // }

    // if let Some(f) = args.fifo {
    //     let _file = std::fs::File::open(f).unwrap();
    //     std::process::exit(0);
    // }

    // spawn_privilege().unwrap();
    // std::process::exit(0);

    let mode = match args.mode {
        Some(v) => v,
        None => ActionMode::Update,
    };

    let ret = match mode {
        ActionMode::UpdateApt => upm::backend::apt::AptBackend::new().update(),
        ActionMode::UpdateBrew => upm::backend::brew::BrewBackend::new().update(),
        ActionMode::UpdateFlatpak => upm::backend::flatpak::FlatpakBackend::new().update(),

        ActionMode::OutdateApt => {
            let pkgs = upm::backend::apt::AptBackend::new().list_upgradable();
            match pkgs {
                Ok(v) => list_package(&v),
                Err(e) => Err(e),
            }
        }
        ActionMode::OutdateBrew => {
            let pkgs = upm::backend::brew::BrewBackend::new().list_upgradable();
            match pkgs {
                Ok(v) => list_package(&v),
                Err(e) => Err(e),
            }
        }
        ActionMode::OutdateFlatpak => {
            let pkgs = upm::backend::flatpak::FlatpakBackend::new().list_upgradable();
            match pkgs {
                Ok(v) => list_package(&v),
                Err(e) => Err(e),
            }
        }
        _ => Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "not implementation.",
        )),
    };

    if let Err(e) = ret {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
