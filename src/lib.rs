pub mod backend;
pub mod rpc;

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
