// Registering app commands here makes Tauri generate an `allow-<command>`
// permission for each one, so a command is only callable from a window
// whose capability grants it (see capabilities/main.json).
const COMMANDS: &[&str] = &[
    "list_workspaces",
    "create_workspace",
    "list_systems",
    "create_system",
];

fn main() {
    tauri_build::try_build(
        tauri_build::Attributes::new()
            .app_manifest(tauri_build::AppManifest::new().commands(COMMANDS)),
    )
    .unwrap_or_else(|e| panic!("tauri build failed: {e:#}"));
}
