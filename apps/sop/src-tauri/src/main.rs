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
            commands::sop::sop_create_document,
            commands::sop::sop_list_documents,
            commands::sop::sop_update_document,
            commands::sop::sop_submit_for_approval,
            commands::sop::sop_list_approval_steps,
            commands::sop::sop_decide_approval,
            commands::sop::sop_publish_document,
            commands::sop::sop_list_versions,
            commands::sop::sop_assign_acknowledgments,
            commands::sop::sop_list_acknowledgments,
            commands::sop::sop_record_acknowledgment,
            commands::export::generate_export_pack,
        ])
        .run(tauri::generate_context!())
        .expect("error while running sop application");
}
