//! GitHub release check + in-place update (like macOS UpdateManager)

use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};

use crate::json;

/// /releases (not /releases/latest) to get the highest version, same as macOS UpdateChecker
const API_URL: &str = "https://api.github.com/repos/khaphanspace/gonhanh.org/releases";
const ZIP_SUFFIX: &str = "-win-x64.zip";
pub const VERSION: &str = env!("GONHANH_VERSION");

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Info {
    pub version: String,
    pub page_url: String,
    /// None: the release has no Windows zip → open the page instead
    pub zip_url: Option<String>,
}

/// Newer stable release than this build, or None
pub fn check() -> Result<Option<Info>, String> {
    let body = crate::win::http_get(API_URL)?;
    pick_update(&String::from_utf8_lossy(&body), VERSION)
}

pub fn compare_versions(a: &str, b: &str) -> std::cmp::Ordering {
    let parts = |v: &str| -> Vec<u64> {
        let mut p: Vec<u64> = v
            .split('.')
            .map(|x| x.trim().parse().unwrap_or(0))
            .collect();
        p.resize(4, 0);
        p
    };
    parts(a).cmp(&parts(b))
}

/// Pure part of `check`, for tests
pub fn pick_update(releases_json: &str, current: &str) -> Result<Option<Info>, String> {
    let root = json::parse(releases_json).ok_or("Dữ liệu từ GitHub không hợp lệ")?;
    let mut best: Option<Info> = None;
    for release in root.items() {
        if release.get("draft").is_true() || release.get("prerelease").is_true() {
            continue;
        }
        let tag = release
            .get("tag_name")
            .as_str()
            .trim_start_matches(['v', 'V']);
        if !tag.starts_with(|c: char| c.is_ascii_digit()) {
            continue;
        }
        if best
            .as_ref()
            .is_some_and(|b| compare_versions(tag, &b.version).is_le())
        {
            continue;
        }
        let zip_url = release
            .get("assets")
            .items()
            .iter()
            .find(|a| a.get("name").as_str().to_lowercase().ends_with(ZIP_SUFFIX))
            .map(|a| a.get("browser_download_url").as_str().to_string());
        best = Some(Info {
            version: tag.to_string(),
            page_url: release.get("html_url").as_str().to_string(),
            zip_url,
        });
    }
    Ok(best.filter(|b| compare_versions(&b.version, current).is_gt()))
}

/// Download + swap the exe. Ok(true) = restart now. No zip → opens the page, Ok(false).
pub fn install(info: &Info) -> Result<bool, String> {
    let Some(zip_url) = &info.zip_url else {
        crate::win::open_url(&info.page_url);
        return Ok(false);
    };

    let work = std::env::temp_dir().join(format!("GoNhanh-update-{}", info.version));
    let _ = std::fs::remove_dir_all(&work);
    let files = work.join("files");
    std::fs::create_dir_all(&files).map_err(|e| format!("Không tạo được thư mục tạm: {e}"))?;

    let zip = work.join("update.zip");
    std::fs::write(&zip, crate::win::http_get(zip_url)?)
        .map_err(|e| format!("Không ghi được file cập nhật: {e}"))?;

    // tar.exe ships with Windows 10 1803+ and reads zip
    let tar = PathBuf::from(std::env::var_os("SystemRoot").unwrap_or("C:\\Windows".into()))
        .join(r"System32\tar.exe");
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let status = std::process::Command::new(tar)
        .arg("-xf")
        .arg(&zip)
        .arg("-C")
        .arg(&files)
        .creation_flags(CREATE_NO_WINDOW)
        .status()
        .map_err(|_| "Không giải nén được bản cập nhật".to_string())?;
    if !status.success() {
        return Err("Giải nén bản cập nhật thất bại".into());
    }
    let new_exe = find_file(&files, "GoNhanh.exe").ok_or("Gói cập nhật không có GoNhanh.exe")?;

    // A running exe can be renamed but not overwritten
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let old = old_path(&exe);
    std::fs::rename(&exe, &old).map_err(|_| "Không thay được file ứng dụng".to_string())?;
    if std::fs::copy(&new_exe, &exe).is_err() {
        let _ = std::fs::rename(&old, &exe); // put the working version back
        return Err("Không chép được bản mới".into());
    }
    Ok(true)
}

/// Delete GoNhanh.exe.old left by the previous update (fails harmlessly if absent or still locked)
pub fn cleanup_old_version() {
    if let Ok(exe) = std::env::current_exe() {
        let _ = std::fs::remove_file(old_path(&exe));
    }
}

fn old_path(exe: &Path) -> PathBuf {
    let mut p = exe.as_os_str().to_owned();
    p.push(".old");
    p.into()
}

fn find_file(dir: &Path, name: &str) -> Option<PathBuf> {
    for entry in std::fs::read_dir(dir).ok()?.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(found) = find_file(&path, name) {
                return Some(found);
            }
        } else if path
            .file_name()
            .is_some_and(|n| n.eq_ignore_ascii_case(name))
        {
            return Some(path);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    const RELEASES: &str = r#"[
        {"tag_name":"v1.0.170","draft":false,"prerelease":true,"html_url":"p170","assets":[]},
        {"tag_name":"v1.0.166","draft":false,"prerelease":false,"html_url":"p166",
         "assets":[{"name":"GoNhanh.dmg","browser_download_url":"dmg"},{"name":"GoNhanh-1.0.166-win-x64.zip","browser_download_url":"zip166"}]},
        {"tag_name":"v1.0.165","draft":false,"prerelease":false,"html_url":"p165","assets":[]}]"#;

    #[test]
    fn picks_newest_stable_with_windows_zip() {
        let u = pick_update(RELEASES, "1.0.165").unwrap().unwrap();
        assert_eq!(u.version, "1.0.166");
        assert_eq!(u.zip_url.as_deref(), Some("zip166"));
        assert_eq!(u.page_url, "p166");
    }

    #[test]
    fn no_update_when_current() {
        assert_eq!(pick_update(RELEASES, "1.0.166").unwrap(), None);
    }

    #[test]
    fn version_compare() {
        assert!(compare_versions("1.0.165", "1.0.99").is_gt());
        assert!(compare_versions("1.0", "1.0.0").is_eq());
    }

    #[test]
    fn bad_json_is_an_error() {
        assert!(pick_update("not json", "1.0.0").is_err());
    }
}
