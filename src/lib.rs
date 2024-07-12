pub mod backend;

/// The result type of the package manager.
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub struct Error {
    pub code: std::io::ErrorKind,
    pub message: String,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl Error {
    /// Create a new error.
    ///
    /// # Arguments
    ///
    /// * `code` - The error code.
    /// * `message` - The error message.
    ///
    /// # Returns
    /// The error.
    pub fn new(code: std::io::ErrorKind, message: &str) -> Self {
        Self {
            code,
            message: message.to_string(),
        }
    }
}

#[derive(Debug)]
pub struct OutdateItem {
    pub name: String,
    pub current_version: String,
    pub target_version: String,
}

#[derive(Debug, Clone, Copy)]
pub struct MethodPrivilege {
    /// The update() method requires root privilege.
    pub update: bool,

    /// The list_upgradable() method requires root privilege.
    pub outdate: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct BackendSetup {
    pub privilege: MethodPrivilege,
}

/// The backend of the package manager.
pub trait UpmBackend {
    /// Get the permission requirements of the package manager's method.
    ///
    /// # Returns
    /// The permission requirements of the package manager's method.
    fn setup(&self) -> Result<BackendSetup>;

    /// Update packages index.
    ///
    /// # Returns
    /// `Ok(())` if the update is successful, otherwise `Err(std::io::Error)`.
    fn update(&self) -> Result<()>;

    /// List upgradable packages.
    ///
    /// # Returns
    /// A list of upgradable packages.
    fn outdated(&self) -> Result<Vec<OutdateItem>>;

    /// Upgrade packages.
    ///
    /// # Returns
    /// `Ok(())` if the upgrade is successful, otherwise `Err(std::io::Error)`.
    fn upgrade(&self) -> Result<()>;
}

/// Check if the current user is root.
///
/// # Returns
/// `Ok(())` if the current user is root, otherwise `Err(std::io::Error)`.
pub fn require_privilege() -> Result<()> {
    if nix::unistd::geteuid().is_root() == false {
        return Err(Error::new(
            std::io::ErrorKind::PermissionDenied,
            "This command requires root privilege.",
        ));
    }

    Ok(())
}

/// Check if the current user is not root.
///
/// # Returns
/// `Ok(())` if the current user is not root, otherwise `Err(std::io::Error)`.
pub fn reject_privilege() -> Result<()> {
    if nix::unistd::geteuid().is_root() {
        return Err(Error::new(
            std::io::ErrorKind::PermissionDenied,
            "This command does not require root privilege.",
        ));
    }

    Ok(())
}
