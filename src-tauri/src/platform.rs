//! OS and CPU architecture helpers for releases, updater keys, and plugin downloads.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlatformTarget {
    pub os: &'static str,
    pub arch: &'static str,
}

impl PlatformTarget {
    pub fn id(self) -> String {
        format!("{}-{}", self.os, self.arch)
    }

    /// Tauri updater platform key (see app-release latest.json builder).
    pub fn updater_key(self) -> &'static str {
        match (self.os, self.arch) {
            ("windows", "x64") => "windows-x86_64",
            ("windows", "arm64") => "windows-aarch64",
            ("macos", "x64") => "darwin-x86_64",
            ("macos", "arm64") => "darwin-aarch64",
            ("linux", "x64") => "linux-x86_64",
            ("linux", "arm64") => "linux-aarch64",
            _ => "unknown",
        }
    }
}

pub fn current_target() -> PlatformTarget {
    let arch = if cfg!(target_arch = "x86_64") {
        "x64"
    } else if cfg!(target_arch = "aarch64") {
        "arm64"
    } else if cfg!(target_arch = "x86") {
        "x86"
    } else {
        "unknown"
    };

    let os = if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        "unknown"
    };

    PlatformTarget { os, arch }
}

/// Installed filename inside Daz Studio's plugins folder (OS-specific, not arch-specific).
pub fn bridge_plugin_installed_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "DazPilotBridge.dll"
    } else if cfg!(target_os = "macos") {
        "libDazPilotBridge.dylib"
    } else {
        "libDazPilotBridge.so"
    }
}

/// GitHub release asset name for the bridge binary (includes arch on Windows).
pub fn bridge_plugin_release_asset_name() -> String {
    let target = current_target();
    match target.os {
        "windows" => format!("DazPilotBridge-{}.dll", target.arch),
        "macos" => format!("libDazPilotBridge-{}.dylib", target.arch),
        "linux" => format!("libDazPilotBridge-{}.so", target.arch),
        _ => bridge_plugin_installed_name().to_string(),
    }
}

/// Ordered URLs to try when downloading the bridge from GitHub Releases.
pub fn bridge_plugin_download_urls(tag: Option<&str>) -> Vec<String> {
    let base = match tag {
        Some(t) => format!("https://github.com/millsydotdev/DazPilot/releases/download/{t}"),
        None => "https://github.com/millsydotdev/DazPilot/releases/latest/download".to_string(),
    };
    let arch_asset = bridge_plugin_release_asset_name();
    let generic = bridge_plugin_installed_name().to_string();
    vec![format!("{base}/{arch_asset}"), format!("{base}/{generic}")]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_target_has_known_os() {
        let t = current_target();
        assert!(matches!(t.os, "windows" | "macos" | "linux" | "unknown"));
    }

    #[test]
    fn windows_release_asset_includes_arch() {
        if cfg!(target_os = "windows") {
            let name = bridge_plugin_release_asset_name();
            assert!(name.starts_with("DazPilotBridge-"));
            assert!(name.ends_with(".dll"));
        }
    }
}
