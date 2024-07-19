#[derive(Debug)]
pub struct BrewBackend {}

impl BrewBackend {
    pub fn new() -> BrewBackend {
        BrewBackend {}
    }
}

impl crate::UpmBackend for BrewBackend {
    fn setup(&self) -> anyhow::Result<crate::BackendSetup> {
        let child = std::process::Command::new("brew")
            .arg("--version")
            .output()?;
        if child.status.success() == false {
            return Err(anyhow::anyhow!("brew is not found."));
        }

        let setup = crate::BackendSetup::Installed(crate::MethodPrivilege {
            update: false,
            outdated: false,
            upgrade: false,
        });
        Ok(setup)
    }

    fn update(&self) -> anyhow::Result<()> {
        let brew = std::process::Command::new("brew").arg("update").output()?;
        if brew.status.success() == false {
            let output = String::from_utf8_lossy(&brew.stderr);
            return Err(anyhow::anyhow!("{}", output.to_string()));
        }

        Ok(())
    }

    fn outdated(&self) -> anyhow::Result<crate::rpc::OutdatedResult> {
        let brew = std::process::Command::new("brew")
            .env("HOMEBREW_NO_ENV_HINTS", "1")
            .args(&["outdated", "--json=v2"])
            .output()?;
        if brew.status.success() == false {
            let output = String::from_utf8_lossy(&brew.stderr);
            return Err(anyhow::anyhow!("{}", output.to_string()));
        }

        let output = String::from_utf8_lossy(&brew.stdout).to_string();
        let output: serde_json::Value = serde_json::from_str(&output)?;
        let formulae = &output["formulae"];

        let mut ret = crate::rpc::OutdatedResult { pkgs: Vec::new() };
        for item in formulae.as_array().unwrap() {
            let name = item["name"].as_str().unwrap();
            let current_version = item["installed_versions"][0].as_str().unwrap();
            let target_version = item["current_version"].as_str().unwrap();

            let item = crate::rpc::OutdateItem {
                name: name.to_string(),
                vendor: "formulae".to_string(),
                current_version: current_version.to_string(),
                target_version: target_version.to_string(),
            };

            ret.pkgs.push(item);
        }

        Ok(ret)
    }

    fn upgrade(&self) -> anyhow::Result<()> {
        let brew = std::process::Command::new("brew")
            .args(&["upgrade"])
            .output()?;
        if brew.status.success() == false {
            let output = String::from_utf8_lossy(&brew.stderr);
            return Err(anyhow::anyhow!("{}", output.to_string()));
        }

        Ok(())
    }
}
