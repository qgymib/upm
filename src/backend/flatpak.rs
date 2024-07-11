#[derive(Debug)]
pub struct FlatpakBackend {}

impl FlatpakBackend {
    pub fn new() -> FlatpakBackend {
        FlatpakBackend {}
    }
}

impl crate::UpmBackend for FlatpakBackend {
    fn update(&self) -> std::io::Result<()> {
        crate::require_privilege()?;

        let flatpak = std::process::Command::new("flatpak")
            .args(&["update", "--appstream"])
            .output()?;

        if flatpak.status.success() == false {
            let output = String::from_utf8_lossy(&flatpak.stderr);
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                output.to_string(),
            ));
        }
        Ok(())
    }

    fn list_upgradable(&self) -> std::io::Result<Vec<crate::OutdateItem>> {
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
fn flatpak_remote_ls_updates() -> std::io::Result<Vec<FlatpakItem>> {
    let flatpak = std::process::Command::new("flatpak")
        .args(&["remote-ls", "--updates"])
        .output()?;
    if flatpak.status.success() == false {
        let output = String::from_utf8_lossy(&flatpak.stderr);
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            output.to_string(),
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
fn flatpak_ls() -> std::io::Result<Vec<FlatpakItem>> {
    let flatpak = std::process::Command::new("flatpak")
        .args(&["list"])
        .output()?;
    if flatpak.status.success() == false {
        let output = String::from_utf8_lossy(&flatpak.stderr);
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            output.to_string(),
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
