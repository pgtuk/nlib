use serde::Deserialize;
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::sync::LazyLock;

pub const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const GITHUB_REPO: &str = "pgtuk/nlib";

static GITHUB_TOKEN: LazyLock<Option<String>> = LazyLock::new(|| {
    std::env::var("GITHUB_TOKEN")
        .or_else(|_| std::env::var("GH_TOKEN"))
        .ok()
        .or_else(|| {
            option_env!("GITHUB_TOKEN")
                .or(option_env!("GH_TOKEN"))
                .map(str::to_string)
        })
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
});

#[derive(Debug, Clone)]
pub struct ReleaseInfo {
    pub version: String,
    pub download_url: String,
    pub release_notes: Option<String>,
}

#[derive(Deserialize, Debug)]
struct GithubRelease {
    tag_name: String,
    body: Option<String>,
    assets: Vec<GithubAsset>,
}

#[derive(Deserialize, Debug)]
struct GithubAsset {
    name: String,
    url: Option<String>,
    browser_download_url: String,
}

pub fn parse_version(tag: &str) -> Option<(u32, u32, u32)> {
    let clean = tag.trim_start_matches(|c| c == 'v' || c == 'V');
    let mut parts = clean.split('.');
    let major = parts.next()?.parse::<u32>().ok()?;
    let minor = parts.next().unwrap_or("0").parse::<u32>().ok()?;
    let patch_str = parts.next().unwrap_or("0").split('-').next()?;
    let patch = patch_str.parse::<u32>().ok()?;
    Some((major, minor, patch))
}

pub fn is_newer_version(latest: &str, current: &str) -> bool {
    match (parse_version(latest), parse_version(current)) {
        (Some(l), Some(c)) => l > c,
        _ => latest != current,
    }
}

pub async fn check_for_update() -> Result<Option<ReleaseInfo>, String> {
    let client = reqwest::Client::builder()
        .user_agent("nlib-updater")
        .build()
        .map_err(|e| e.to_string())?;

    let url = format!("https://api.github.com/repos/{GITHUB_REPO}/releases/latest");
    let mut req = client
        .get(&url)
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28");

    if let Some(token) = GITHUB_TOKEN.as_deref() {
        req = req.bearer_auth(token);
    }

    let resp = req.send().await.map_err(|e| e.to_string())?;

    if resp.status() == reqwest::StatusCode::NOT_FOUND {
        // No releases published yet or repository not found
        return Ok(None);
    }

    if !resp.status().is_success() {
        return Err(format!("GitHub API returned HTTP {}", resp.status()));
    }

    let release: GithubRelease = resp.json().await.map_err(|e| e.to_string())?;
    let latest_version = release
        .tag_name
        .trim_start_matches(|c| c == 'v' || c == 'V')
        .to_string();

    if !is_newer_version(&latest_version, CURRENT_VERSION) {
        return Ok(None);
    }

    let asset = release
        .assets
        .iter()
        .find(|a| a.name == "nlib" || a.name.starts_with("nlib-linux"))
        .or_else(|| release.assets.iter().find(|a| a.name.contains("nlib")))
        .ok_or_else(|| "No compatible binary asset found in latest release".to_string())?;

    // For private repositories, the API asset url with octet-stream header is preferred
    let download_url = if GITHUB_TOKEN.is_some() {
        asset
            .url
            .clone()
            .unwrap_or_else(|| asset.browser_download_url.clone())
    } else {
        asset.browser_download_url.clone()
    };

    Ok(Some(ReleaseInfo {
        version: latest_version,
        download_url,
        release_notes: release.body,
    }))
}

pub async fn apply_update(download_url: &str) -> Result<(), String> {
    let exe_path = std::env::current_exe()
        .map_err(|e| format!("Failed to get current executable path: {e}"))?;

    let real_exe_path = fs::canonicalize(&exe_path).unwrap_or(exe_path);
    let parent_dir = real_exe_path
        .parent()
        .ok_or_else(|| "Executable has no parent directory".to_string())?;

    let client = reqwest::Client::builder()
        .user_agent("nlib-updater")
        .build()
        .map_err(|e| e.to_string())?;

    let mut req = client.get(download_url);
    if let Some(token) = GITHUB_TOKEN.as_deref() {
        req = req
            .bearer_auth(token)
            .header("Accept", "application/octet-stream")
            .header("X-GitHub-Api-Version", "2022-11-28");
    }

    let bytes = req
        .send()
        .await
        .map_err(|e| format!("Download request failed: {e}"))?
        .bytes()
        .await
        .map_err(|e| format!("Failed to read downloaded binary: {e}"))?;

    let temp_path = parent_dir.join(format!(".nlib_update_{}", std::process::id()));

    fs::write(&temp_path, &bytes).map_err(|e| format!("Failed to write temporary file: {e}"))?;

    #[cfg(unix)]
    {
        let permissions = fs::Permissions::from_mode(0o755);
        if let Err(e) = fs::set_permissions(&temp_path, permissions) {
            let _ = fs::remove_file(&temp_path);
            return Err(format!("Failed to set executable permissions: {e}"));
        }
    }

    if let Err(e) = fs::rename(&temp_path, &real_exe_path) {
        let _ = fs::remove_file(&temp_path);
        return Err(format!("Failed to replace executable: {e}"));
    }

    Ok(())
}
