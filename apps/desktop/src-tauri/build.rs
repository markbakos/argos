fn main() {
    println!("cargo:rerun-if-env-changed=ARGOS_PACKAGE_PROFILE");

    let profile = match std::env::var("ARGOS_PACKAGE_PROFILE").as_deref() {
        Ok("production") => "production",
        _ => "development",
    };

    println!("cargo:rustc-env=ARGOS_RUNTIME_PROFILE={profile}");
    println!(
        "cargo:rustc-env=ARGOS_BUILD_KIND={}",
        std::env::var("PROFILE").unwrap_or_else(|_| "unknown".to_owned())
    );
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&[
            "core_boundary_proof",
            "core_get_build_info",
            "core_get_settings",
            "core_get_system_identity",
            "core_list_modules",
            "core_set_theme",
            "task_manager_process_details",
            "task_manager_snapshot",
        ]),
    ))
    .unwrap_or_else(|error| panic!("Tauri build failed: {error}"));
}
