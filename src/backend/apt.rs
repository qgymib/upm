#[derive(Debug)]
pub struct AptBackend {}

impl AptBackend {
    pub fn new() -> AptBackend {
        AptBackend {}
    }
}

impl crate::UpmBackend for AptBackend {
    fn setup(&self) -> anyhow::Result<crate::BackendSetup> {
        let child = std::process::Command::new("apt-get")
            .arg("--version")
            .output()?;
        if child.status.success() == false {
            return Ok(crate::BackendSetup::NotInstalled);
        }

        let child = std::process::Command::new("apt")
            .arg("--version")
            .output()?;
        if child.status.success() == false {
            return Err(anyhow::anyhow!("apt is not found."));
        }

        let setup = crate::BackendSetup::Installed(crate::MethodPrivilege {
            update: true,
            outdated: false,
            upgrade: true,
        });
        Ok(setup)
    }

    fn update(&self) -> anyhow::Result<()> {
        let apt = std::process::Command::new("apt-get")
            .arg("update")
            .output()?;
        if apt.status.success() == false {
            let output = String::from_utf8_lossy(&apt.stderr);
            return Err(anyhow::anyhow!("{}", output.to_string()));
        }
        Ok(())
    }

    fn outdated(&self) -> anyhow::Result<crate::rpc::OutdatedResult> {
        let apt = std::process::Command::new("apt")
            .env("LANG", "en_US.UTF-8")
            .env("LANGUAGE", "en_US")
            .args(&["list", "--upgradable"])
            .output()?;
        if apt.status.success() == false {
            let output = String::from_utf8_lossy(&apt.stderr);
            return Err(anyhow::anyhow!("{}", output.to_string(),));
        }

        let output = String::from_utf8_lossy(&apt.stdout);
        let output = output.to_string();
        println!("{}", output);

        let lines: Vec<&str> = output.lines().collect();
        let re =
            regex::Regex::new(r"(\S+)/(\S+)\s+(\S+)\s+\(\S+)\s+[upgradable from: (\S+)\]").unwrap();

        let mut ret = crate::rpc::OutdatedResult { pkgs: Vec::new() };
        for line in lines {
            let Some(caps) = re.captures(line) else {
                continue;
            };

            let package_name = caps.get(1).unwrap().as_str();
            let ventor = caps.get(2).unwrap().as_str();
            let current_version = caps.get(5).unwrap().as_str();
            let target_version = caps.get(3).unwrap().as_str();

            let item = crate::rpc::OutdateItem {
                name: package_name.to_string(),
                vendor: ventor.to_string(),
                current_version: current_version.to_string(),
                target_version: target_version.to_string(),
            };
            ret.pkgs.push(item);
        }

        Ok(ret)
    }

    fn upgrade(&self) -> anyhow::Result<()> {
        let apt = std::process::Command::new("apt-get")
            .args(&["upgrade", "-y"])
            .output()?;
        if apt.status.success() == false {
            let output = String::from_utf8_lossy(&apt.stderr);
            return Err(anyhow::anyhow!("{}", output.to_string(),));
        }
        Ok(())
    }
}
