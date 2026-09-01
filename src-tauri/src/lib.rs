//! dsh-launcher lib:注册表、进程操作与 Tauri 命令装配。

pub mod commands;
pub mod commands_envcheck;
pub mod commands_heavy;
pub mod commands_homes;
pub mod commands_processes;
pub mod commands_runtime;
pub mod envcheck;
pub mod envcheck_probe;
pub mod envcheck_sys;
pub mod homes;
pub mod launcher;
pub mod launcher_process;
pub mod registry;
pub mod runtime_install;
pub mod runtime_install_core;

#[cfg(test)]
#[path = "envcheck_tests.rs"]
mod envcheck_tests;

#[cfg(test)]
#[path = "launcher_process_tests.rs"]
mod launcher_process_tests;

#[cfg(test)]
#[path = "runtime_install_tests.rs"]
mod runtime_install_tests;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::list_versions,
            commands::add_manual_version,
            commands_heavy::fingerprint_version,
            commands_heavy::install_npm_version,
            commands::add_dev_version,
            commands_heavy::remove_version,
            commands_homes::list_homes,
            commands_homes::add_home,
            commands_homes::create_home,
            commands_homes::clone_home,
            commands_homes::bind_home_version,
            commands_homes::remove_home,
            commands_processes::list_profiles,
            commands_processes::start_profile,
            commands_processes::stop_profile,
            commands_processes::list_running,
            commands_envcheck::env_check,
            commands_envcheck::env_check_fast,
            commands_envcheck::env_snapshot,
            commands_runtime::install_runtime,
            commands_runtime::runtime_info,
        ])
        .manage(commands_processes::ProcessMap::default())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
