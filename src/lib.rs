pub mod backend;

#[derive(Debug)]
pub struct OutdateItem {
    pub name: String,
    pub current_version: String,
    pub target_version: String,
}

pub trait UpmBackend {
    /// Update packages index.
    fn update(&self) -> std::io::Result<()>;

    /// List upgradable packages.
    ///
    /// # Returns
    /// A list of upgradable packages.
    fn list_upgradable(&self) -> std::io::Result<Vec<OutdateItem>>;
}

/// Check if the current user is root.
///
/// # Returns
/// `Ok(())` if the current user is root, otherwise `Err(std::io::Error)`.
pub fn require_privilege() -> std::io::Result<()> {
    if nix::unistd::geteuid().is_root() == false {
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "This command requires root privilege.",
        ));
    }

    Ok(())
}
