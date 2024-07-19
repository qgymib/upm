#[derive(Debug)]
pub struct FlatpakBackend {}

impl FlatpakBackend {
    pub fn new() -> FlatpakBackend {
        FlatpakBackend {}
    }
}

impl crate::UpmBackend for FlatpakBackend {
    fn setup(&self) -> anyhow::Result<crate::BackendSetup> {
        let child = std::process::Command::new("flatpak")
            .arg("--version")
            .output()?;
        if child.status.success() == false {
            return Err(anyhow::anyhow!("flatpak is not found."));
        }

        let setup = crate::BackendSetup::Installed(crate::MethodPrivilege {
            update: true,
            outdated: false,
            upgrade: false,
        });

        Ok(setup)
    }

    fn update(&self) -> anyhow::Result<()> {
        let flatpak = std::process::Command::new("flatpak")
            .args(&["update", "--appstream"])
            .output()?;

        if flatpak.status.success() == false {
            let output = String::from_utf8_lossy(&flatpak.stderr);
            return Err(anyhow::anyhow!("{}", output.to_string()));
        }
        Ok(())
    }

    fn outdated(&self) -> anyhow::Result<crate::rpc::OutdatedResult> {
        let mut ret = crate::rpc::OutdatedResult { pkgs: Vec::new() };
        let updates = flatpak_remote_ls_updates()?;
        let installs = flatpak_ls()?;

        for item in updates {
            let install = installs.iter().find(|&x| x.name == item.name);
            if let Some(install) = install {
                let outdate_item = crate::rpc::OutdateItem {
                    name: item.name.clone(),
                    vendor: item.vendor.clone(),
                    current_version: install.version.clone(),
                    target_version: item.version.clone(),
                };

                ret.pkgs.push(outdate_item);
            }
        }

        Ok(ret)
    }

    fn upgrade(&self) -> anyhow::Result<()> {
        let flatpak = std::process::Command::new("flatpak")
            .args(&["update", "--noninteractive"])
            .output()?;

        if flatpak.status.success() == false {
            let output = String::from_utf8_lossy(&flatpak.stderr);
            return Err(anyhow::anyhow!("{}", output.to_string()));
        }
        Ok(())
    }
}

#[derive(Debug)]
struct FlatpakItem {
    name: String,
    vendor: String,
    version: String,
}

/// List updates from remote.
///
/// # Returns
/// A list of updates.
fn flatpak_remote_ls_updates() -> anyhow::Result<Vec<FlatpakItem>> {
    let flatpak = std::process::Command::new("flatpak")
        .args(&[
            "remote-ls",
            "--updates",
            "--columns=application,version,origin",
        ])
        .output()?;

    if flatpak.status.success() == false {
        let output = String::from_utf8_lossy(&flatpak.stderr);
        return Err(anyhow::anyhow!("{}", output.to_string()));
    }

    let output = String::from_utf8_lossy(&flatpak.stdout).to_string();
    let lines: Vec<&str> = output.lines().collect();

    let mut ret = Vec::new();
    let re = regex::Regex::new(r"^(\S+)\s+(\S+)\s+(\S+)$").unwrap();
    for line in lines {
        let Some(caps) = re.captures(line) else {
            continue;
        };
        let name = caps.get(1).unwrap().as_str();
        let version = caps.get(2).unwrap().as_str();
        let vendor = caps.get(3).unwrap().as_str();

        let item = FlatpakItem {
            name: name.to_string(),
            vendor: vendor.to_string(),
            version: version.to_string(),
        };

        ret.push(item);
    }

    Ok(ret)
}

/// List installed flatpak packages.
///
/// # Returns
/// A list of installed flatpak packages.
fn flatpak_ls() -> anyhow::Result<Vec<FlatpakItem>> {
    let flatpak = std::process::Command::new("flatpak")
        .args(&["list", "--columns=application,version,origin"])
        .output()?;
    if flatpak.status.success() == false {
        let output = String::from_utf8_lossy(&flatpak.stderr);
        return Err(anyhow::anyhow!("{}", output.to_string()));
    }

    let output = String::from_utf8_lossy(&flatpak.stdout).to_string();
    let lines: Vec<&str> = output.lines().collect();

    let mut ret = Vec::new();
    let re = regex::Regex::new(r"^(\S+)\s+(\S+)\s+(\S+)$").unwrap();
    for line in lines {
        let Some(caps) = re.captures(line) else {
            continue;
        };
        let name = caps.get(1).unwrap().as_str();
        let version = caps.get(2).unwrap().as_str();
        let vendor = caps.get(3).unwrap().as_str();

        let item = FlatpakItem {
            name: name.to_string(),
            vendor: vendor.to_string(),
            version: version.to_string(),
        };

        ret.push(item);
    }

    Ok(ret)
}
