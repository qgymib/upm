use clap::Parser;

#[derive(Debug, clap::Parser)]
struct Args {
    #[command(subcommand)]
    mode: Mode,
}

#[derive(Debug, clap::Subcommand)]
enum Mode {
    Track(TrackMode),
}

#[derive(Debug, clap::Args)]
struct TrackMode {
    pkg: String,
    url: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct PkgRecord {
    name: String,
    system: PkgSystem,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
enum PkgSystem {
    Github(PkgGitHub),
    Flatpak,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct PkgGitHub {
    owner: String,
    repo: String,
}

fn fetch_github_latest(owner: &str, repo: &str) -> std::io::Result<String> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/releases/latest",
        owner, repo
    );

    let curl = std::process::Command::new("curl")
        .arg("-H")
        .arg("Accept: application/vnd.github+json")
        .arg("-H")
        .arg("X-GitHub-Api-Version: 2022-11-28")
        .arg(url)
        .output()?;

    let output = String::from_utf8_lossy(&curl.stdout);
    let output: serde_json::Value = serde_json::from_str(output.as_ref()).unwrap();
    let version = output.get("tag_name").unwrap();

    return Ok(version.to_string());
}

fn dpkg_version(pkg: &str) -> std::io::Result<String> {
    let dpkg = std::process::Command::new("dpkg")
        .arg("--showformat='${Version}'")
        .arg("-s")
        .arg(pkg)
        .output()?;

    if dpkg.status.success() == false {
        let msg = format!("package `{}` is not installed", pkg);
        return Err(std::io::Error::new(std::io::ErrorKind::NotFound, msg));
    }

    let version = String::from_utf8_lossy(&dpkg.stdout);
    Ok(version.to_string())
}

fn track(pkg: &str, url: &str) -> std::io::Result<()> {
    let pkg_version = match dpkg_version(pkg) {
        Ok(v) => v,
        Err(e) => return Err(e),
    };

    let re = regex::Regex::new(r"https://github.com/([\w\d_-]+)/([\w\d_-]+)").unwrap();
    let r = re.captures(url).unwrap();
    let owner = r.get(1).unwrap().as_str();
    let repo = r.get(2).unwrap().as_str();
    let url_version = match fetch_github_latest(owner, repo) {
        Ok(v) => v,
        Err(e) => return Err(e),
    };

    println!("{}: local={},online={}", pkg, pkg_version, url_version);
    Ok(())
}

fn main() {
    let args = Args::parse();
    let m = args.mode;
    let c = match m {
        Mode::Track(v) => v,
    };

    if let Err(e) = track(c.pkg.as_str(), c.url.as_str()) {
        eprint!("{}", e);
        std::process::exit(1);
    }
}
