#[derive(Debug)]
pub struct BrewBackend {}

impl BrewBackend {
    pub fn new() -> BrewBackend {
        BrewBackend {}
    }
}

impl crate::UpmBackend for BrewBackend {
    fn setup(&self) -> crate::Result<crate::BackendSetup> {
        let child = std::process::Command::new("brew").arg("--version").output();
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
                "brew is not found.",
            ));
        }

        let setup = crate::BackendSetup {
            privilege: crate::MethodPrivilege {
                update: false,
                outdate: false,
            },
        };
        Ok(setup)
    }

    fn update(&self) -> crate::Result<()> {
        let brew = std::process::Command::new("brew").arg("update").output();
        let brew = match brew {
            Ok(v) => v,
            Err(e) => {
                return Err(crate::Error::new(
                    std::io::ErrorKind::Other,
                    e.to_string().as_str(),
                ));
            }
        };
        if brew.status.success() == false {
            let output = String::from_utf8_lossy(&brew.stderr);
            return Err(crate::Error::new(
                std::io::ErrorKind::Other,
                output.to_string().as_str(),
            ));
        }

        Ok(())
    }

    fn outdated(&self) -> crate::Result<Vec<crate::OutdateItem>> {
        let brew = std::process::Command::new("brew")
            .env("HOMEBREW_NO_ENV_HINTS", "1")
            .args(&["outdated", "--verbose"])
            .output();
        let brew = match brew {
            Ok(v) => v,
            Err(e) => {
                return Err(crate::Error::new(
                    std::io::ErrorKind::Other,
                    e.to_string().as_str(),
                ));
            }
        };
        if brew.status.success() == false {
            let output = String::from_utf8_lossy(&brew.stderr);
            return Err(crate::Error::new(
                std::io::ErrorKind::Other,
                output.to_string().as_str(),
            ));
        }

        let output = String::from_utf8_lossy(&brew.stdout).to_string();
        let lines: Vec<&str> = output.lines().collect();

        let mut ret = Vec::new();
        let re = regex::Regex::new(r"^(\S+)\s+\((\S+)\)\s+<\s+(\S+)$").unwrap();
        for line in lines {
            let Some(caps) = re.captures(line) else {
                continue;
            };
            let package_name = caps.get(1).unwrap().as_str();
            let current_version = caps.get(2).unwrap().as_str();
            let target_version = caps.get(3).unwrap().as_str();

            let item = crate::OutdateItem {
                name: package_name.to_string(),
                current_version: current_version.to_string(),
                target_version: target_version.to_string(),
            };

            ret.push(item);
        }

        Ok(ret)
    }
}
