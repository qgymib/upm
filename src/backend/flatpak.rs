#[derive(Debug)]
pub struct FlatpakBackend {}

impl FlatpakBackend {
    pub fn new() -> FlatpakBackend {
        FlatpakBackend {}
    }
}

impl crate::UpmBackend for FlatpakBackend {
    fn setup(&self) -> crate::Result<crate::BackendSetup> {
        let child = std::process::Command::new("flatpak")
            .arg("--version")
            .output();
        let child = match child {
            Ok(v) => v,
            Err(e) => {
                return Err(crate::Error::new(
                    std::io::ErrorKind::Other,
                    e.to_string().as_str(),
                ));
            }
        };
        if child.status.success() == false {
            return Err(crate::Error::new(
                std::io::ErrorKind::NotFound,
                "flatpak is not found.",
            ));
        }

        let setup = crate::BackendSetup {
            privilege: crate::MethodPrivilege {
                update: true,
                outdate: false,
            },
        };

        Ok(setup)
    }

    fn update(&self) -> crate::Result<()> {
        let flatpak = std::process::Command::new("flatpak")
            .args(&["update", "--appstream"])
            .output();
        let flatpak = match flatpak {
            Ok(v) => v,
            Err(e) => {
                return Err(crate::Error::new(
                    std::io::ErrorKind::Other,
                    e.to_string().as_str(),
                ));
            }
        };

        if flatpak.status.success() == false {
            let output = String::from_utf8_lossy(&flatpak.stderr);
            return Err(crate::Error::new(
                std::io::ErrorKind::Other,
                output.to_string().as_str(),
            ));
        }
        Ok(())
    }

    fn outdated(&self) -> crate::Result<Vec<crate::OutdateItem>> {
        let mut ret = Vec::new();
        let updates = flatpak_remote_ls_updates()?;
        let installs = flatpak_ls()?;

        for item in updates {
            let install = installs.iter().find(|&x| x.name == item.name);
            if let Some(install) = install {
                let outdate_item = crate::OutdateItem {
                    name: item.name.clone(),
                    current_version: install.version.clone(),
                    target_version: item.version.clone(),
                };

                ret.push(outdate_item);
            }
        }

        Ok(ret)
    }
}

#[derive(Debug)]
struct FlatpakItem {
    name: String,
    version: String,
}

/// List updates from remote.
///
/// # Returns
/// A list of updates.
fn flatpak_remote_ls_updates() -> crate::Result<Vec<FlatpakItem>> {
    let flatpak = std::process::Command::new("flatpak")
        .args(&["remote-ls", "--updates"])
        .output();
    let flatpak = match flatpak {
        Ok(v) => v,
        Err(e) => {
            return Err(crate::Error::new(
                std::io::ErrorKind::Other,
                e.to_string().as_str(),
            ));
        }
    };
    if flatpak.status.success() == false {
        let output = String::from_utf8_lossy(&flatpak.stderr);
        return Err(crate::Error::new(
            std::io::ErrorKind::Other,
            output.to_string().as_str(),
        ));
    }

    let output = String::from_utf8_lossy(&flatpak.stdout).to_string();
    let lines: Vec<&str> = output.lines().collect();

    let mut ret = Vec::new();
    let re = regex::Regex::new(r"^(\S+)\s+(\S+)\s+(\S+)\s+(\S+)\s+(\S+)$").unwrap();
    for line in lines {
        let Some(caps) = re.captures(line) else {
            continue;
        };
        let name = caps.get(2).unwrap().as_str();
        let version = caps.get(3).unwrap().as_str();

        let item = FlatpakItem {
            name: name.to_string(),
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
fn flatpak_ls() -> crate::Result<Vec<FlatpakItem>> {
    let flatpak = std::process::Command::new("flatpak")
        .args(&["list"])
        .output();
    let flatpak = match flatpak {
        Ok(v) => v,
        Err(e) => {
            return Err(crate::Error::new(
                std::io::ErrorKind::Other,
                e.to_string().as_str(),
            ));
        }
    };
    if flatpak.status.success() == false {
        let output = String::from_utf8_lossy(&flatpak.stderr);
        return Err(crate::Error::new(
            std::io::ErrorKind::Other,
            output.to_string().as_str(),
        ));
    }

    let output = String::from_utf8_lossy(&flatpak.stdout).to_string();
    let lines: Vec<&str> = output.lines().collect();

    let mut ret = Vec::new();
    let re = regex::Regex::new(r"^(\S+)\s+(\S+)\s+(\S+)\s+(\S+)\s+(\S+)$").unwrap();
    for line in lines {
        let Some(caps) = re.captures(line) else {
            continue;
        };
        let name = caps.get(2).unwrap().as_str();
        let version = caps.get(3).unwrap().as_str();

        let item = FlatpakItem {
            name: name.to_string(),
            version: version.to_string(),
        };

        ret.push(item);
    }

    Ok(ret)
}
