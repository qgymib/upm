pub mod backend;
pub mod host;
pub mod rpc;
pub mod tui;
pub mod worker;

#[derive(Debug, Clone, Copy)]
pub struct MethodPrivilege {
    /// The update() method requires root privilege.
    pub update: bool,

    /// The outdated() method requires root privilege.
    pub outdated: bool,

    /// The upgrade() method requires root privilege.
    pub upgrade: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum BackendSetup {
    NotInstalled,
    Installed(MethodPrivilege),
}

/// The backend of the package manager.
pub trait UpmBackend {
    /// Get the permission requirements of the package manager's method.
    ///
    /// # Returns
    /// The permission requirements of the package manager's method.
    fn setup(&self) -> anyhow::Result<BackendSetup>;

    /// Update packages index.
    ///
    /// # Returns
    /// `Ok(())` if the update is successful, otherwise `Err(std::io::Error)`.
    fn update(&self) -> anyhow::Result<()>;

    /// List upgradable packages.
    ///
    /// # Returns
    /// A list of upgradable packages.
    fn outdated(&self) -> anyhow::Result<rpc::OutdatedResult>;

    /// Upgrade packages.
    ///
    /// # Returns
    /// `Ok(())` if the upgrade is successful, otherwise `Err(std::io::Error)`.
    fn upgrade(&self) -> anyhow::Result<()>;
}

type UmpBackendHashMap = std::collections::HashMap<&'static str, Box<dyn UpmBackend>>;
type UpmBackendSetupHashMap = std::collections::HashMap<String, BackendSetup>;

pub struct WorkerRouter {
    backends: UmpBackendHashMap,
    info: UpmBackendSetupHashMap,
}

impl WorkerRouter {
    pub fn new() -> Self {
        use backend::*;

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

    fn info(&mut self, name: &str) -> anyhow::Result<BackendSetup> {
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

impl rpc::server::Router for WorkerRouter {
    fn handshake(&self, _: rpc::HandeshakeParams) -> anyhow::Result<rpc::HandeshakeResult> {
        Ok(rpc::HandeshakeResult {
            privilige: nix::unistd::geteuid().is_root(),
        })
    }

    fn update(&self, params: rpc::UpdateParams) -> anyhow::Result<rpc::UpdateResult> {
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
        Ok(rpc::UpdateResult {})
    }

    fn outdated(&self, params: rpc::OutdatedParams) -> anyhow::Result<rpc::OutdatedResult> {
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

    fn upgrade(&self, params: rpc::UpgradeParams) -> anyhow::Result<rpc::UpgradeResult> {
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
        Ok(rpc::UpgradeResult {})
    }
}

/// Check if the current user is root.
///
/// # Returns
/// `Ok(())` if the current user is root, otherwise `Err(std::io::Error)`.
pub fn require_privilege() -> anyhow::Result<()> {
    if nix::unistd::geteuid().is_root() == false {
        return Err(anyhow::anyhow!("This command requires root privilege."));
    }

    Ok(())
}

/// Check if the current user is not root.
///
/// # Returns
/// `Ok(())` if the current user is not root, otherwise `Err(std::io::Error)`.
pub fn reject_privilege() -> anyhow::Result<()> {
    if nix::unistd::geteuid().is_root() {
        return Err(anyhow::anyhow!(
            "This command does not require root privilege."
        ));
    }

    Ok(())
}
