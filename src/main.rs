use clap::{Args, Parser, Subcommand};
use upm::{backend, UpmBackend};

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
    Update(BackendName),
    Outdated(BackendName),
    Install(PackageName),
    Uninstall(PackageName),
}

#[derive(Debug, Args)]
struct PackageName {
    #[arg(help = "The name of the package")]
    name: String,
}

#[derive(Debug, Args)]
struct BackendName {
    #[arg(help = "The name of the backend")]
    name: Option<String>,
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

fn list_package(pkg: &Vec<upm::OutdateItem>) -> upm::Result<()> {
    for item in pkg {
        println!(
            "{}: {} -> {}",
            item.name, item.current_version, item.target_version
        );
    }

    Ok(())
}

struct UpmInstance {
    /// The permission of the package manager.
    setup: Option<upm::Result<upm::BackendSetup>>,

    /// The backend of the package manager.
    backend: Box<dyn upm::UpmBackend>,
}

impl UpmInstance {
    fn new(backend: impl UpmBackend + 'static) -> Self {
        Self {
            setup: None,
            backend: Box::new(backend),
        }
    }

    fn setup(&mut self) -> upm::Result<upm::BackendSetup> {
        if let Some(setup) = &self.setup {
            return setup.clone();
        } else {
            let setup = self.backend.setup();
            self.setup = Some(setup.clone());
            setup
        }
    }
}

type UmpBackendMap = std::collections::HashMap<String, UpmInstance>;

fn do_update(backend: &mut UmpBackendMap, name: &BackendName) -> upm::Result<()> {
    if let Some(name) = &name.name {
        let backend = match backend.get_mut(name) {
            Some(v) => v,
            None => {
                return Err(upm::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("backend '{}' not found.", name).as_str(),
                ));
            }
        };
        let setup = backend.setup()?;
        if setup.privilege.update {
            upm::require_privilege()?;
        }
        backend.backend.update()?;
    } else {
        let mut names = Vec::new();
        for (name, _) in backend.iter_mut() {
            names.push(BackendName {
                name: Some(name.clone()),
            });
        }
        for item in names {
            do_update(backend, &item)?;
        }
    }

    Ok(())
}

fn do_outdate(backend: &mut UmpBackendMap, name: &BackendName) -> upm::Result<()> {
    if let Some(name) = &name.name {
        let backend = match backend.get_mut(name) {
            Some(v) => v,
            None => {
                return Err(upm::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("backend '{}' not found.", name).as_str(),
                ));
            }
        };
        let setup = backend.setup()?;
        if setup.privilege.outdate {
            upm::require_privilege()?;
        }
        let ret = backend.backend.outdated()?;
        list_package(&ret)?;
    } else {
        let mut names = Vec::new();
        for (name, _) in backend.iter_mut() {
            names.push(BackendName {
                name: Some(name.clone()),
            });
        }
        for item in names {
            do_outdate(backend, &item)?;
        }
    }

    Ok(())
}

fn main() {
    let args = UpmArgs::parse();

    let mut backend_map = UmpBackendMap::new();
    backend_map.insert(
        "apt".to_string(),
        UpmInstance::new(backend::apt::AptBackend::new()),
    );
    backend_map.insert(
        "brew".to_string(),
        UpmInstance::new(backend::brew::BrewBackend::new()),
    );
    backend_map.insert(
        "flatpak".to_string(),
        UpmInstance::new(backend::flatpak::FlatpakBackend::new()),
    );

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
        None => ActionMode::Update(BackendName { name: None }),
    };

    let ret = match mode {
        ActionMode::Update(v) => do_update(&mut backend_map, &v),
        ActionMode::Outdated(v) => do_outdate(&mut backend_map, &v),
        _ => Err(upm::Error::new(
            std::io::ErrorKind::NotFound,
            "not implementation.",
        )),
    };

    if let Err(e) = ret {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
