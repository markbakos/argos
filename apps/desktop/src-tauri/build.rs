fn main() {
    println!("cargo:rerun-if-env-changed=ARGOS_PACKAGE_PROFILE");

    let profile = match std::env::var("ARGOS_PACKAGE_PROFILE").as_deref() {
        Ok("production") => "production",
        _ => "development",
    };

    println!("cargo:rustc-env=ARGOS_RUNTIME_PROFILE={profile}");
    tauri_build::build();
}
