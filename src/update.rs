use anyhow::{bail, Result};
use std::path::Path;

const REPO: &str = "CianusDev/apix";
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(serde::Deserialize)]
struct GithubRelease {
    tag_name: String,
}

pub async fn run_update() -> Result<()> {
    println!("Checking for updates...");

    let client = reqwest::Client::builder()
        .user_agent(format!("apix/{}", CURRENT_VERSION))
        .build()?;

    let release: GithubRelease = client
        .get(format!(
            "https://api.github.com/repos/{}/releases/latest",
            REPO
        ))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let latest = release.tag_name.trim_start_matches('v');

    if latest == CURRENT_VERSION {
        println!("Already up to date (v{}).", CURRENT_VERSION);
        return Ok(());
    }

    println!("New version available: v{} → v{}", CURRENT_VERSION, latest);
    println!("Downloading...");

    let target = detect_target()?;
    let ext = if cfg!(windows) { "zip" } else { "tar.gz" };
    let filename = format!("apix-{}-{}.{}", release.tag_name, target, ext);
    let url = format!(
        "https://github.com/{}/releases/download/{}/{}",
        REPO, release.tag_name, filename
    );

    let bytes = client
        .get(&url)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;

    let binary = extract_binary(&bytes)?;
    let current_exe = std::env::current_exe()?;
    replace_binary(&current_exe, &binary)?;

    println!("Updated to v{}!", latest);
    Ok(())
}

fn detect_target() -> Result<&'static str> {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", "x86_64") => Ok("x86_64-unknown-linux-gnu"),
        ("macos", "aarch64") => Ok("aarch64-apple-darwin"),
        ("windows", "x86_64") => Ok("x86_64-pc-windows-msvc"),
        (os, arch) => bail!("Unsupported platform: {}/{}", os, arch),
    }
}

#[cfg(unix)]
fn extract_binary(bytes: &[u8]) -> Result<Vec<u8>> {
    use flate2::read::GzDecoder;
    use std::io::Read;
    use tar::Archive;

    let gz = GzDecoder::new(bytes);
    let mut archive = Archive::new(gz);

    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?;
        if path.file_name().is_some_and(|n| n == "apix") {
            let mut buf = Vec::new();
            entry.read_to_end(&mut buf)?;
            return Ok(buf);
        }
    }
    bail!("Binary not found in archive")
}

#[cfg(windows)]
fn extract_binary(bytes: &[u8]) -> Result<Vec<u8>> {
    use std::io::{Cursor, Read};
    use zip::ZipArchive;

    let cursor = Cursor::new(bytes);
    let mut archive = ZipArchive::new(cursor)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        if file.name().ends_with("apix.exe") {
            let mut buf = Vec::new();
            file.read_to_end(&mut buf)?;
            return Ok(buf);
        }
    }
    bail!("Binary not found in archive")
}

fn replace_binary(current_exe: &Path, binary: &[u8]) -> Result<()> {
    let parent = current_exe
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Cannot determine binary directory"))?;
    let tmp = parent.join(".apix_update_tmp");

    std::fs::write(&tmp, binary)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o755))?;
    }

    std::fs::rename(&tmp, current_exe)?;
    Ok(())
}
