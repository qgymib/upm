use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(version, about)]
pub struct UpmArgs {
    #[command(subcommand)]
    pub mode: Option<ActionMode>,

    #[arg(long, default_value_t = 0, hide = true)]
    pub worker: u16,
}

#[derive(Debug, Subcommand)]
pub enum ActionMode {
    Update(BackendName),
    Outdated(BackendName),
    Upgrade(BackendName),

    Install(PackageName),
    Uninstall(PackageName),
}

#[derive(Debug, Args)]
pub struct PackageName {
    #[arg(help = "The name of the package")]
    name: String,
}

#[derive(Debug, Args)]
pub struct BackendName {
    #[arg(help = "The name of the backend")]
    name: Option<String>,
}

struct Controller {
    normal_worker: crate::rpc::client::Client,
    root_worker: crate::rpc::client::Client,

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

pub fn run_as_controller(args: &UpmArgs) -> anyhow::Result<()> {
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

    let mut client1 = crate::rpc::client::Client::new(stream1);
    let mut client2 = crate::rpc::client::Client::new(stream2);

    let handshake = crate::rpc::HandeshakeParams {
        pid: nix::unistd::getpid().as_raw() as u32,
    };
    let rsp1 = client1.call::<crate::rpc::Handshake>(&handshake)?;
    let rsp2 = client2.call::<crate::rpc::Handshake>(&handshake)?;
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

    do_job(&mut ctl, args, crate::WorkerRouter::new())?;

    ctl.shutdown()?;
    Ok(())
}

fn do_job(
    ctl: &mut Controller,
    args: &UpmArgs,
    mut router: crate::WorkerRouter,
) -> anyhow::Result<()> {
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

fn do_job_upgrade(
    ctl: &mut Controller,
    router: &mut crate::WorkerRouter,
    name: &Option<String>,
) -> anyhow::Result<()> {
    if let Some(name) = name {
        do_job_upgrade_item(ctl, router, name)?;
    } else {
        do_job_upgrade_full(ctl, router)?;
    }

    Ok(())
}

fn do_job_upgrade_full(
    ctl: &mut Controller,
    router: &mut crate::WorkerRouter,
) -> anyhow::Result<()> {
    let mut names = Vec::new();
    for (name, _) in router.backends.iter() {
        names.push(name.to_string());
    }
    for name in names {
        do_job_upgrade_item(ctl, router, &name)?;
    }

    Ok(())
}

fn do_jon_update_full(
    ctl: &mut Controller,
    router: &mut crate::WorkerRouter,
) -> anyhow::Result<()> {
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
    router: &mut crate::WorkerRouter,
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
    router: &mut crate::WorkerRouter,
    name: &str,
) -> anyhow::Result<()> {
    let info = match router.info(name)? {
        crate::BackendSetup::NotInstalled => {
            // The package manager is not installed.
            return Ok(());
        }
        crate::BackendSetup::Installed(v) => v,
    };

    let params = crate::rpc::UpgradeParams {
        backend_name: name.to_string(),
    };
    if info.upgrade {
        ctl.root_worker.call::<crate::rpc::Upgrade>(&params)?;
    } else {
        ctl.normal_worker.call::<crate::rpc::Upgrade>(&params)?;
    }

    Ok(())
}

fn do_job_update_item(
    ctl: &mut Controller,
    router: &mut crate::WorkerRouter,
    name: &str,
) -> anyhow::Result<()> {
    let info = match router.info(name)? {
        crate::BackendSetup::NotInstalled => {
            // The package manager is not installed.
            return Ok(());
        }
        crate::BackendSetup::Installed(v) => v,
    };

    let params = crate::rpc::UpdateParams {
        backend_name: name.to_string(),
    };
    if info.update {
        ctl.root_worker.call::<crate::rpc::Update>(&params)?;
    } else {
        ctl.normal_worker.call::<crate::rpc::Update>(&params)?;
    }

    Ok(())
}

fn do_job_outdated(
    ctl: &mut Controller,
    router: &mut crate::WorkerRouter,
    name: &Option<String>,
) -> anyhow::Result<()> {
    if let Some(name) = name {
        do_job_outdated_item(ctl, router, name)?;
    } else {
        do_job_outdated_full(ctl, router)?;
    }

    Ok(())
}

fn do_job_outdated_item(
    ctl: &mut Controller,
    router: &mut crate::WorkerRouter,
    name: &str,
) -> anyhow::Result<()> {
    let info = match router.info(name)? {
        crate::BackendSetup::NotInstalled => {
            // The package manager is not installed.
            return Ok(());
        }
        crate::BackendSetup::Installed(v) => v,
    };

    let params = crate::rpc::OutdatedParams {
        backend_name: name.to_string(),
    };
    let rsp = if info.update {
        ctl.root_worker.call::<crate::rpc::Outdated>(&params)?
    } else {
        ctl.normal_worker.call::<crate::rpc::Outdated>(&params)?
    };
    list_package(&rsp)?;

    Ok(())
}

fn do_job_outdated_full(
    ctl: &mut Controller,
    router: &mut crate::WorkerRouter,
) -> anyhow::Result<()> {
    let mut names = Vec::new();
    for (name, _) in router.backends.iter() {
        names.push(name.to_string());
    }
    for name in names {
        do_job_outdated_item(ctl, router, &name)?;
    }

    Ok(())
}

fn list_package(pkg: &crate::rpc::OutdatedResult) -> anyhow::Result<()> {
    for item in pkg.pkgs.iter() {
        println!(
            "{}: {} -> {}",
            item.name, item.current_version, item.target_version
        );
    }

    Ok(())
}
