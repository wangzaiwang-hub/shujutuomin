mod commands;
mod core;

use commands::{masking, crypto, sandbox, rules, batch, database, proxy, webview};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 设置 panic hook 来捕获崩溃信息
    std::panic::set_hook(Box::new(|panic_info| {
        eprintln!("=== PANIC OCCURRED ===");
        eprintln!("Panic info: {}", panic_info);
        
        if let Some(location) = panic_info.location() {
            eprintln!("Panic location: {}:{}:{}", location.file(), location.line(), location.column());
        }
        
        if let Some(payload) = panic_info.payload().downcast_ref::<&str>() {
            eprintln!("Panic payload: {}", payload);
        } else if let Some(payload) = panic_info.payload().downcast_ref::<String>() {
            eprintln!("Panic payload: {}", payload);
        }
        
        eprintln!("=== END PANIC INFO ===");
        
        // 尝试写入崩溃日志文件
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("crash.log") 
        {
            use std::io::Write;
            let _ = writeln!(file, "[{}] PANIC: {}", chrono::Utc::now(), panic_info);
        }
    }));

    println!("=== CheersAI Vault Starting ===");
    
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
            database::migrate_old_database,
            proxy::fetch_webpage,
            webview::open_webview_window,
            webview::navigate_webview,
            webview::webview_reload,
            webview::close_webview_window,
            webview::get_webview_url,
            webview::webview_eval_script,
            webview::navigate_main_window_with_button,
        ])
        .setup(|_app| {
            println!("=== Tauri Setup Complete ===");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
