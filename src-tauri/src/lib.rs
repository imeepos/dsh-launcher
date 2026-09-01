//! dsh-launcher lib:注册表、进程操作与 Tauri 命令装配。

pub mod commands;
pub mod launcher;
pub mod registry;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::list_versions,
            commands::add_manual_version,
            commands::fingerprint_version,
            commands::install_npm_version,
            commands::add_dev_version,
            commands::remove_version,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
