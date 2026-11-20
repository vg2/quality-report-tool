//! Tauri app shell: window setup, IPC commands, and wiring to the core crates.

mod commands;
mod error;

use qrt_secrets::KeyringStore;
use qrt_store::SqliteStore;
use std::sync::Arc;
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

pub struct AppState {
    pub store: SqliteStore,
}

/// Only the app's own pages may be loaded in the webview. External links are
/// never navigated to in-app (they will be opened in the system browser
/// after confirmation, in a later phase).
fn is_allowed_navigation(url: &tauri::Url) -> bool {
    let host = url.host_str().unwrap_or_default();
    match url.scheme() {
        "tauri" => true,
        "http" | "https" if host == "tauri.localhost" => true,
        "http" if cfg!(debug_assertions) => host == "127.0.0.1" && url.port() == Some(1420),
        _ => false,
    }
}

pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let result = tauri::Builder::default()
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            let store = SqliteStore::open(data_dir, Arc::new(KeyringStore))?;
            app.manage(AppState { store });

            WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
                .title("Quality Report Tool")
                .inner_size(1280.0, 800.0)
                .min_inner_size(900.0, 600.0)
                .on_navigation(|url| {
                    let allowed = is_allowed_navigation(url);
                    if !allowed {
                        tracing::warn!(scheme = url.scheme(), "blocked navigation");
                    }
                    allowed
                })
                .build()?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_workspaces,
            commands::create_workspace,
            commands::list_systems,
            commands::create_system,
        ])
        .run(tauri::generate_context!());

    if let Err(e) = result {
        tracing::error!(error = %e, "application failed");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::is_allowed_navigation;

    fn url(s: &str) -> tauri::Url {
        s.parse().unwrap_or_else(|e| panic!("bad url {s}: {e}"))
    }

    #[test]
    fn allows_app_origins_only() {
        assert!(is_allowed_navigation(&url("tauri://localhost/index.html")));
        assert!(is_allowed_navigation(&url(
            "http://tauri.localhost/index.html"
        )));
        assert!(!is_allowed_navigation(&url("https://example.com")));
        assert!(!is_allowed_navigation(&url("file:///etc/passwd")));
        assert!(!is_allowed_navigation(&url("http://127.0.0.1:8080")));
    }
}
