//! Thin Tauri composition boundary for Argos.

/// Runs the desktop host without placing application behavior in Tauri.
pub fn run() -> tauri::Result<()> {
    tauri::Builder::default().run(tauri::generate_context!())
}
