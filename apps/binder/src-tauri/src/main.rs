#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
#![allow(dead_code)]

mod app_state;
mod commands;
mod error_map;

use app_state::AppState;
use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            app.manage(AppState::new());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::vault::vault_create,
            commands::vault::vault_open,
            commands::vault::vault_close,
            commands::license::check_license_status,
            commands::license::install_license,
            commands::evidence::list_evidence,
            commands::evidence::import_evidence,
            commands::binder::binder_create_control,
            commands::binder::binder_list_controls,
            commands::binder::binder_link_evidence,
            commands::binder::binder_set_control_status,
            commands::binder::binder_status_summary,
            commands::export::generate_export_pack,
        ])
        .run(tauri::generate_context!())
        .expect("error while running binder application");
}
