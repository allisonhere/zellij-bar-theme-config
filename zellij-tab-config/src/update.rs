use std::io::Read;

const REPO: &str = "allisonhere/zellij-bar-theme-config";
const CURRENT: &str = env!("CARGO_PKG_VERSION");

pub enum UpdateMsg {
    VersionChecked(Result<Option<String>, String>),
    UpdateComplete(Result<(), String>),
}

fn parse_ver(s: &str) -> Option<(u32, u32, u32)> {
    let s = s.trim_start_matches('v');
    let mut parts = s.splitn(3, '.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    let patch = parts.next()?.parse().ok()?;
    Some((major, minor, patch))
}

/// Returns `Some(tag)` if the latest GitHub release is newer than the current
/// binary version, or `None` if already up to date.
pub fn check_version() -> Result<Option<String>, String> {
    let url = format!("https://api.github.com/repos/{REPO}/releases/latest");
    let resp: serde_json::Value = ureq::get(&url)
        .set("User-Agent", "zellij-tab-config")
        .call()
        .map_err(|e| e.to_string())?
        .into_json()
        .map_err(|e| e.to_string())?;
    let tag = resp["tag_name"].as_str().ok_or("no tag_name in response")?;
    let latest = parse_ver(tag).ok_or_else(|| format!("unparseable tag: {tag}"))?;
    let current = parse_ver(CURRENT).ok_or_else(|| format!("unparseable current version: {CURRENT}"))?;
    if latest > current {
        Ok(Some(tag.to_string()))
    } else {
        Ok(None)
    }
}

/// Downloads the release binary for `tag` and replaces the running executable.
pub fn download_and_replace(tag: &str) -> Result<(), String> {
    let url = format!(
        "https://github.com/{REPO}/releases/download/{tag}/zellij-tab-config-linux-x86_64"
    );
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let tmp = exe.with_extension("tmp");

    let resp = ureq::get(&url)
        .set("User-Agent", "zellij-tab-config")
        .call()
        .map_err(|e| e.to_string())?;

    let mut bytes = Vec::new();
    resp.into_reader()
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;

    std::fs::write(&tmp, &bytes).map_err(|e| e.to_string())?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o755))
            .map_err(|e| e.to_string())?;
    }

    // Atomic replace — safe on Linux even while the binary is running.
    std::fs::rename(&tmp, &exe).map_err(|e| e.to_string())?;

    Ok(())
}
