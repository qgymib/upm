use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(version, about)]
struct UpmArgs {
    #[command(subcommand)]
    mode: Option<ActionMode>,

    #[arg(long, default_value_t = 0, hide = true)]
    worker: u16,
}

#[derive(Debug, Subcommand)]
enum ActionMode {
    Update(BackendName),
    Outdated(BackendName),
    Upgrade(BackendName),

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

fn list_package(pkg: &upm::rpc::OutdatedResult) -> anyhow::Result<()> {
    for item in pkg.pkgs.iter() {
        println!(
            "{}: {} -> {}",
            item.name, item.current_version, item.target_version
        );
    }

    Ok(())
}

type UmpBackendHashMap = std::collections::HashMap<&'static str, Box<dyn upm::UpmBackend>>;
type UpmBackendSetupHashMap = std::collections::HashMap<String, upm::BackendSetup>;

struct WorkerRouter {
    backends: UmpBackendHashMap,
    info: UpmBackendSetupHashMap,
}

impl WorkerRouter {
    fn new() -> Self {
        use upm::backend::*;

        let mut backends = UmpBackendHashMap::new();
        backends.insert("apt", Box::new(apt::AptBackend::new()));
        backends.insert("brew", Box::new(brew::BrewBackend::new()));
        backends.insert("flatpak", Box::new(flatpak::FlatpakBackend::new()));

        let info = UpmBackendSetupHashMap::new();

        Self {
            backends: backends,
            info: info,
        }
    }

    fn info(&mut self, name: &str) -> anyhow::Result<upm::BackendSetup> {
        if let Some(info) = self.info.get(name) {
            return Ok(info.clone());
        }

        let backend = match self.backends.get(&name) {
            Some(v) => v,
            None => {
                return Err(anyhow::anyhow!("backend '{}' not found.", name));
            }
        };

        let info = backend.setup()?;
        self.info.insert(name.to_string(), info.clone());

        Ok(info)
    }
}

impl upm::rpc::server::Router for WorkerRouter {
    fn handshake(
        &self,
        _: upm::rpc::HandeshakeParams,
    ) -> anyhow::Result<upm::rpc::HandeshakeResult> {
        Ok(upm::rpc::HandeshakeResult {
            privilige: nix::unistd::geteuid().is_root(),
        })
    }

    fn update(&self, params: upm::rpc::UpdateParams) -> anyhow::Result<upm::rpc::UpdateResult> {
        let backend = match self.backends.get(&params.backend_name.as_str()) {
            Some(v) => v,
            None => {
                return Err(anyhow::anyhow!(
                    "backend '{}' not found.",
                    params.backend_name
                ));
            }
        };
        backend.update()?;
        Ok(upm::rpc::UpdateResult {})
    }

    fn outdated(
        &self,
        params: upm::rpc::OutdatedParams,
    ) -> anyhow::Result<upm::rpc::OutdatedResult> {
        let backend = match self.backends.get(&params.backend_name.as_str()) {
            Some(v) => v,
            None => {
                return Err(anyhow::anyhow!(
                    "backend '{}' not found.",
                    params.backend_name
                ));
            }
        };
        let ret = backend.outdated()?;
        Ok(ret)
    }

    fn upgrade(&self, params: upm::rpc::UpgradeParams) -> anyhow::Result<upm::rpc::UpgradeResult> {
        let backend = match self.backends.get(&params.backend_name.as_str()) {
            Some(v) => v,
            None => {
                return Err(anyhow::anyhow!(
                    "backend '{}' not found.",
                    params.backend_name
                ));
            }
        };
        backend.upgrade()?;
        Ok(upm::rpc::UpgradeResult {})
    }
}

fn run_as_worker(port: u16) -> anyhow::Result<()> {
    let addr = format!("127.0.0.1:{}", port);
    let stream = std::net::TcpStream::connect(addr).unwrap();

    let mut server = upm::rpc::server::Server::new(stream);
    let router = WorkerRouter::new();
    server.serve(&router)?;

    Ok(())
}

struct Controller {
    normal_worker: upm::rpc::client::Client,
    root_worker: upm::rpc::client::Client,

    child1: std::process::Child,
    child2: std::process::Child,
}

impl Controller {
    fn shutdown(&mut self) -> anyhow::Result<()> {
        self.normal_worker.shutdown()?;
        self.root_worker.shutdown()?;

        self.child1.wait()?;
        self.child2.wait()?;

        Ok(())
    }
}

fn run_as_controller(args: &UpmArgs) -> anyhow::Result<()> {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    log::info!("server start on {}", addr);

    let worker_arg = format!("--worker={}", addr.port());

    let path = std::env::current_exe()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    log::debug!("exec_path: {}", path);

    let child1 = std::process::Command::new("sudo")
        .arg(&path)
        .arg(&worker_arg)
        .spawn()
        .unwrap();
    let child2 = std::process::Command::new(&path)
        .arg(&worker_arg)
        .spawn()
        .unwrap();

    let (stream1, _) = listener.accept().unwrap();
    let (stream2, _) = listener.accept().unwrap();

    let mut client1 = upm::rpc::client::Client::new(stream1);
    let mut client2 = upm::rpc::client::Client::new(stream2);

    let handshake = upm::rpc::HandeshakeParams {
        pid: nix::unistd::getpid().as_raw() as u32,
    };
    let rsp1 = client1.call::<upm::rpc::Handshake>(&handshake)?;
    let rsp2 = client2.call::<upm::rpc::Handshake>(&handshake)?;
    assert_ne!(rsp1.privilige, rsp2.privilige);
    println!("rsp1: {:?}", rsp1);
    println!("rsp2: {:?}", rsp2);

    let mut ctl = if rsp1.privilige {
        Controller {
            normal_worker: client2,
            root_worker: client1,
            child1: child1,
            child2: child2,
        }
    } else {
        Controller {
            normal_worker: client1,
            root_worker: client2,
            child1: child1,
            child2: child2,
        }
    };

    do_job(&mut ctl, args, WorkerRouter::new())?;

    ctl.shutdown()?;
    Ok(())
}

fn do_job_outdated_full(ctl: &mut Controller, router: &mut WorkerRouter) -> anyhow::Result<()> {
    let mut names = Vec::new();
    for (name, _) in router.backends.iter() {
        names.push(name.to_string());
    }
    for name in names {
        do_job_outdated_item(ctl, router, &name)?;
    }

    Ok(())
}

fn do_job_outdated_item(
    ctl: &mut Controller,
    router: &mut WorkerRouter,
    name: &str,
) -> anyhow::Result<()> {
    let info = match router.info(name)? {
        upm::BackendSetup::NotInstalled => {
            // The package manager is not installed.
            return Ok(());
        }
        upm::BackendSetup::Installed(v) => v,
    };

    let params = upm::rpc::OutdatedParams {
        backend_name: name.to_string(),
    };
    let rsp = if info.update {
        ctl.root_worker.call::<upm::rpc::Outdated>(&params)?
    } else {
        ctl.normal_worker.call::<upm::rpc::Outdated>(&params)?
    };
    list_package(&rsp)?;

    Ok(())
}

fn do_job_outdated(
    ctl: &mut Controller,
    router: &mut WorkerRouter,
    name: &Option<String>,
) -> anyhow::Result<()> {
    if let Some(name) = name {
        do_job_outdated_item(ctl, router, name)?;
    } else {
        do_job_outdated_full(ctl, router)?;
    }

    Ok(())
}

fn do_job_update_item(
    ctl: &mut Controller,
    router: &mut WorkerRouter,
    name: &str,
) -> anyhow::Result<()> {
    let info = match router.info(name)? {
        upm::BackendSetup::NotInstalled => {
            // The package manager is not installed.
            return Ok(());
        }
        upm::BackendSetup::Installed(v) => v,
    };

    let params = upm::rpc::UpdateParams {
        backend_name: name.to_string(),
    };
    if info.update {
        ctl.root_worker.call::<upm::rpc::Update>(&params)?;
    } else {
        ctl.normal_worker.call::<upm::rpc::Update>(&params)?;
    }

    Ok(())
}

fn do_jon_update_full(ctl: &mut Controller, router: &mut WorkerRouter) -> anyhow::Result<()> {
    let mut names = Vec::new();
    for (name, _) in router.backends.iter() {
        names.push(name.to_string());
    }
    for name in names {
        do_job_update_item(ctl, router, &name)?;
    }

    Ok(())
}

fn do_job_update(
    ctl: &mut Controller,
    router: &mut WorkerRouter,
    name: &Option<String>,
) -> anyhow::Result<()> {
    if let Some(name) = name {
        do_job_update_item(ctl, router, name)?;
    } else {
        do_jon_update_full(ctl, router)?;
    }

    Ok(())
}

fn do_job_upgrade_item(
    ctl: &mut Controller,
    router: &mut WorkerRouter,
    name: &str,
) -> anyhow::Result<()> {
    let info = match router.info(name)? {
        upm::BackendSetup::NotInstalled => {
            // The package manager is not installed.
            return Ok(());
        }
        upm::BackendSetup::Installed(v) => v,
    };

    let params = upm::rpc::UpgradeParams {
        backend_name: name.to_string(),
    };
    if info.upgrade {
        ctl.root_worker.call::<upm::rpc::Upgrade>(&params)?;
    } else {
        ctl.normal_worker.call::<upm::rpc::Upgrade>(&params)?;
    }

    Ok(())
}

fn do_job_upgrade_full(ctl: &mut Controller, router: &mut WorkerRouter) -> anyhow::Result<()> {
    let mut names = Vec::new();
    for (name, _) in router.backends.iter() {
        names.push(name.to_string());
    }
    for name in names {
        do_job_upgrade_item(ctl, router, &name)?;
    }

    Ok(())
}

fn do_job_upgrade(
    ctl: &mut Controller,
    router: &mut WorkerRouter,
    name: &Option<String>,
) -> anyhow::Result<()> {
    if let Some(name) = name {
        do_job_upgrade_item(ctl, router, name)?;
    } else {
        do_job_upgrade_full(ctl, router)?;
    }

    Ok(())
}

fn do_job(ctl: &mut Controller, args: &UpmArgs, mut router: WorkerRouter) -> anyhow::Result<()> {
    let mode = args
        .mode
        .as_ref()
        .unwrap_or(&ActionMode::Update(BackendName { name: None }));
    let ret = match mode {
        ActionMode::Update(v) => do_job_update(ctl, &mut router, &v.name),
        ActionMode::Outdated(v) => do_job_outdated(ctl, &mut router, &v.name),
        ActionMode::Upgrade(v) => do_job_upgrade(ctl, &mut router, &v.name),
        _ => Err(anyhow::anyhow!("not implementation.")),
    };

    ret
}

fn main() {
    let args = UpmArgs::parse();

    env_logger::init();

    let ret = if args.worker != 0 {
        run_as_worker(args.worker)
    } else {
        run_as_controller(&args)
    };

    if let Err(e) = ret {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
