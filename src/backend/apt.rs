#[derive(Debug)]
pub struct AptBackend {}

impl AptBackend {
    pub fn new() -> AptBackend {
        AptBackend {}
    }
}

impl crate::UpmBackend for AptBackend {
    fn setup(&self) -> crate::Result<crate::BackendSetup> {
        let child = std::process::Command::new("apt-get")
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
                "apt-get is not found.",
            ));
        }

        let child = std::process::Command::new("apt").arg("--version").output();
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
                "apt is not found.",
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
        let apt = std::process::Command::new("apt-get").arg("update").output();
        let apt = match apt {
            Ok(v) => v,
            Err(e) => {
                return Err(crate::Error::new(
                    std::io::ErrorKind::Other,
                    e.to_string().as_str(),
                ));
            }
        };
        if apt.status.success() == false {
            let output = String::from_utf8_lossy(&apt.stderr);
            return Err(crate::Error::new(
                std::io::ErrorKind::Other,
                output.to_string().as_str(),
            ));
        }
        Ok(())
    }

    fn outdated(&self) -> crate::Result<Vec<crate::OutdateItem>> {
        let apt = std::process::Command::new("apt")
            .env("LANG", "en_US.UTF-8")
            .env("LANGUAGE", "en_US")
            .args(&["list", "--upgradable"])
            .output();
        let apt = match apt {
            Ok(v) => v,
            Err(e) => {
                return Err(crate::Error::new(
                    std::io::ErrorKind::Other,
                    e.to_string().as_str(),
                ));
            }
        };
        if apt.status.success() == false {
            let output = String::from_utf8_lossy(&apt.stderr);
            return Err(crate::Error::new(
                std::io::ErrorKind::Other,
                output.to_string().as_str(),
            ));
        }

        let output = String::from_utf8_lossy(&apt.stdout);
        let output = output.to_string();

        let mut lines: Vec<&str> = output.lines().collect();
        lines.remove(0);

        let mut ret = Vec::new();
        for line in lines {
            let parts: Vec<&str> = line.split_whitespace().collect();

            let package_name = parts[0];
            let target_version = parts[1];
            let current_version_part = line.split("upgradable from:").nth(1).unwrap().trim();
            let current_version = current_version_part.trim_end_matches(']');

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
