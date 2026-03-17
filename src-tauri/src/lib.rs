mod commands;
mod core;

use commands::{masking, crypto, sandbox, rules, batch, database};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            masking::mask_file,
            masking::preview_masking,
            crypto::generate_passphrase,
            crypto::encrypt_mapping,
            crypto::decrypt_mapping,
            sandbox::verify_pin,
            sandbox::set_pin,
            sandbox::list_sandbox_files,
            sandbox::list_files_in_directory,
            sandbox::export_sandbox,
            sandbox::import_sandbox,
            rules::get_rules,
            rules::save_rules,
            batch::start_batch_job,
            batch::get_batch_status,
            batch::cancel_batch_job,
            database::initialize_database,
            database::add_log_entry,
            database::get_logs,
            database::get_logs_count,
            database::clear_all_logs,
            database::cleanup_old_logs,
            database::save_user_setting,
            database::get_user_setting,
            database::get_all_user_settings,
            database::delete_user_setting,
            database::add_processing_history,
            database::get_processing_history,
            database::get_statistics,
            database::get_database_info,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
