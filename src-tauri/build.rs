/// Tauri commands the app defines (see `src/ipc/`). Listing them generates one `allow-<name>`
/// permission each, so only the windows whose capability grants a command can call it: the
/// settings window gets them all, the overlays none.
const COMMANDS: &[&str] = &[
    "get_config",
    "save_config",
    "reload_config",
    "get_status",
    "set_paused",
    "set_trigger_capture",
    "pick_file",
    "open_folder",
    "get_app_info",
];

fn main() {
    let manifest = tauri_build::AppManifest::new().commands(COMMANDS);
    if let Err(error) =
        tauri_build::try_build(tauri_build::Attributes::new().app_manifest(manifest))
    {
        panic!("tauri build step failed: {error:#}");
    }
}
