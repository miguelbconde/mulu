//! Mulu (目录): the desktop application.
//!
//! Version 0.0.1 is a rehearsal build: it only opens the window, so that the whole
//! delivery path (automatic build, download, install without administrator rights,
//! first launch) can be tested before search exists.

// On Windows, do not open a console window next to the application.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("Mulu could not start");
}

#[cfg(test)]
mod tests {
    /// The version shown to the user, the installer version and the package version
    /// must always be the same.
    #[test]
    fn versions_agree() {
        let version = env!("CARGO_PKG_VERSION");
        let config = include_str!("../tauri.conf.json");
        assert!(
            config.contains(&format!("\"version\": \"{version}\"")),
            "tauri.conf.json"
        );
        let page = include_str!("../../../ui/index.html");
        assert!(
            page.contains(&format!("data-version=\"{version}\"")),
            "ui/index.html"
        );
    }

    /// Mulu makes no network connections (privacy principle): the page may only load
    /// its own files and talk to the application itself.
    #[test]
    fn content_security_policy_is_local_only() {
        let config = include_str!("../tauri.conf.json");
        let csp = config
            .lines()
            .find(|line| line.trim_start().starts_with("\"csp\""))
            .expect("a content security policy is set");
        assert!(csp.contains("default-src 'self'"));
        let without_ipc = csp.replace("http://ipc.localhost", "");
        for remote in ["http:", "https:", "ws:", "wss:", "*"] {
            assert!(
                !without_ipc.contains(remote),
                "the policy must not allow {remote}"
            );
        }
    }
}
