// Embeds icon, manifest (Common Controls v6, per-monitor DPI) and version info into GoNhanh.exe.
// Version comes from GONHANH_VERSION (git tag, set by scripts/build/windows.sh), default 0.0.0.
use std::{env, fs, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-env-changed=GONHANH_VERSION");
    println!("cargo:rerun-if-changed=res");

    let version = env::var("GONHANH_VERSION").unwrap_or_default();
    let version = version.trim_start_matches('v');
    let version = if version.is_empty() { "0.0.0" } else { version };
    println!("cargo:rustc-env=GONHANH_VERSION={version}");

    let mut parts: Vec<u32> = version.split('.').map(|p| p.parse().unwrap_or(0)).collect();
    parts.resize(4, 0);
    let commas = parts
        .iter()
        .map(u32::to_string)
        .collect::<Vec<_>>()
        .join(",");

    let res = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("res");
    let path = |name: &str| res.join(name).display().to_string().replace('\\', "\\\\");
    let rc = format!(
        r#"#include <winver.h>
1 ICON "{icon}"
1 24 "{manifest}"
VS_VERSION_INFO VERSIONINFO
FILEVERSION {commas}
PRODUCTVERSION {commas}
FILEOS VOS_NT_WINDOWS32
FILETYPE VFT_APP
BEGIN
    BLOCK "StringFileInfo"
    BEGIN
        BLOCK "040904b0"
        BEGIN
            VALUE "CompanyName", "Kha Phan"
            VALUE "FileDescription", "GoNhanh - Vietnamese input method"
            VALUE "FileVersion", "{version}"
            VALUE "InternalName", "GoNhanh"
            VALUE "LegalCopyright", "GPL-3.0-or-later"
            VALUE "OriginalFilename", "GoNhanh.exe"
            VALUE "ProductName", "GoNhanh"
            VALUE "ProductVersion", "{version}"
        END
    END
    BLOCK "VarFileInfo"
    BEGIN
        VALUE "Translation", 0x409, 1200
    END
END
"#,
        icon = path("app.ico"),
        manifest = path("app.manifest"),
    );
    let out = PathBuf::from(env::var("OUT_DIR").unwrap()).join("app.rc");
    fs::write(&out, rc).unwrap();
    embed_resource::compile(&out, embed_resource::NONE)
        .manifest_required()
        .unwrap();
}
