#[derive(Debug)]
pub struct BrewBackend {}

impl BrewBackend {
    pub fn new() -> BrewBackend {
        BrewBackend {}
    }
}

impl crate::UpmBackend for BrewBackend {
    fn update(&self) -> std::io::Result<()> {
        let brew = std::process::Command::new("brew").arg("update").output()?;
        if brew.status.success() == false {
            let output = String::from_utf8_lossy(&brew.stderr);
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                output.to_string(),
            ));
        }

        Ok(())
    }

    fn list_upgradable(&self) -> std::io::Result<Vec<crate::OutdateItem>> {
        let brew = std::process::Command::new("brew")
            .env("HOMEBREW_NO_ENV_HINTS", "1")
            .args(&["outdated", "--verbose"])
            .output()?;
        if brew.status.success() == false {
            let output = String::from_utf8_lossy(&brew.stderr);
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                output.to_string(),
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
