fn main() {
    const COMMANDS: &[&str] = &[
        "load_workspace",
        "profile_action",
        "get_provider",
        "save_provider",
        "remove_provider",
        "test_provider",
        "generate_reply",
        "cancel_reply",
        "copy_reply",
        "window_action",
        "tone_templates",
        "sync_settings",
    ];
    tauri_build::try_build(
        tauri_build::Attributes::new()
            .app_manifest(tauri_build::AppManifest::new().commands(COMMANDS)),
    )
    .expect("Tauri build configuration failed");
}
