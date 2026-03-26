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

fn release_asset_name() -> Option<&'static str> {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", "x86_64") => Some("zellij-tab-config-linux-x86_64"),
        _ => None,
    }
}

/// Returns `Some(tag)` if the latest GitHub release is newer than the current
/// binary version, or `None` if already up to date.
pub fn check_version() -> Result<Option<String>, String> {
    if release_asset_name().is_none() {
        return Ok(None);
    }

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
    let asset = release_asset_name()
        .ok_or_else(|| format!("self-update is only supported on Linux x86_64, not {} {}", std::env::consts::OS, std::env::consts::ARCH))?;
    let url = format!("https://github.com/{REPO}/releases/download/{tag}/{asset}");
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

#[cfg(test)]
mod tests {
    use super::{parse_ver, release_asset_name};

    #[test]
    fn parses_versions_with_optional_v_prefix() {
        assert_eq!(parse_ver("v0.3.5"), Some((0, 3, 5)));
        assert_eq!(parse_ver("0.3.5"), Some((0, 3, 5)));
    }

    #[test]
    fn release_asset_matches_supported_targets() {
        match (std::env::consts::OS, std::env::consts::ARCH) {
            ("linux", "x86_64") => {
                assert_eq!(release_asset_name(), Some("zellij-tab-config-linux-x86_64"));
            }
            _ => assert_eq!(release_asset_name(), None),
        }
    }
}
