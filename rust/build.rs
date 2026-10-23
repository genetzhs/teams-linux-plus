fn main() {
    // Declaring the app commands generates an `allow-<command>` permission for
    // each one, so capabilities/ can grant the remote Teams origin only the
    // bridge commands and keep the configuration commands for local windows.
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&[
            "notify",
            "open_external",
            "set_zoom",
            "open_window",
            "switch_view",
            "rail_state",
            "get_config",
            "save_config",
            "join_meeting",
            "resource_usage",
            "app_info",
            "restart",
        ]),
    ))
    .expect("failed to run tauri-build");
}
