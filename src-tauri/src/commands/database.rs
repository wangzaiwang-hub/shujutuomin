use serde::{Deserialize, Serialize};
use crate::core::database::{Database, LogEntry, ProcessingHistory};
use chrono::Utc;
use uuid::Uuid;
use std::sync::LazyLock;
use tokio::sync::Mutex;

// 全局数据库实例
static DATABASE: LazyLock<Mutex<Option<Database>>> = LazyLock::new(|| Mutex::new(None));

/// 初始化数据库
async fn get_database() -> Result<Database, String> {
    let mut db_guard = DATABASE.lock().await;
    
    if db_guard.is_none() {
        let db = Database::new().await.map_err(|e| format!("Failed to initialize database: {}", e))?;
        *db_guard = Some(db);
    }
    
    // 克隆数据库实例（Database 应该实现 Clone）
    // 由于 SqlitePool 实现了 Clone，我们需要修改 Database 结构
    match db_guard.as_ref() {
        Some(_) => {
            // 重新创建连接，因为 SqlitePool 是可以安全克隆的
            Database::new().await.map_err(|e| format!("Failed to get database: {}", e))
        },
        None => Err("Database not initialized".to_string()),
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogEntryRequest {
    pub level: String,
    pub message: String,
    pub details: Option<String>,
    pub file_path: Option<String>,
    pub operation_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogQueryParams {
    pub limit: Option<i32>,
    pub offset: Option<i32>,
    pub level_filter: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessingHistoryRequest {
    pub file_path: String,
    pub output_path: String,
    pub rule_ids: Vec<String>,
    pub file_size: i64,
    pub masked_count: i32,
    pub processing_time_ms: i64,
    pub status: String,
    pub error_message: Option<String>,
}

// === 日志相关命令 ===

#[tauri::command]
pub async fn add_log_entry(request: LogEntryRequest) -> Result<(), String> {
    let db = get_database().await?;
    
    let entry = LogEntry {
        id: Uuid::new_v4().to_string(),
        timestamp: Utc::now(),
        level: request.level,
        message: request.message,
        details: request.details,
        file_path: request.file_path,
        operation_type: request.operation_type,
        user_id: None, // 可以后续添加用户系统
    };
    
    db.add_log(&entry).await.map_err(|e| format!("Failed to add log: {}", e))?;
    Ok(())
}

#[tauri::command]
pub async fn get_logs(params: LogQueryParams) -> Result<Vec<LogEntry>, String> {
    let db = get_database().await?;
    
    db.get_logs(
        params.limit, 
        params.offset, 
        params.level_filter.as_deref()
    ).await.map_err(|e| format!("Failed to get logs: {}", e))
}

#[tauri::command]
pub async fn get_logs_count(level_filter: Option<String>) -> Result<i64, String> {
    let db = get_database().await?;
    
    db.get_logs_count(level_filter.as_deref())
        .await
        .map_err(|e| format!("Failed to count logs: {}", e))
}

#[tauri::command]
pub async fn clear_all_logs() -> Result<(), String> {
    let db = get_database().await?;
    db.clear_logs().await.map_err(|e| format!("Failed to clear logs: {}", e))?;
    Ok(())
}

#[tauri::command]
pub async fn cleanup_old_logs(days: i32) -> Result<u64, String> {
    let db = get_database().await?;
    db.cleanup_old_logs(days).await.map_err(|e| format!("Failed to cleanup logs: {}", e))
}

// === 用户设置相关命令 ===

#[tauri::command]
pub async fn save_user_setting(key: String, value: String) -> Result<(), String> {
    let db = get_database().await?;
    db.save_setting(&key, &value).await.map_err(|e| format!("Failed to save setting: {}", e))?;
    Ok(())
}

#[tauri::command]
pub async fn get_user_setting(key: String) -> Result<Option<String>, String> {
    let db = get_database().await?;
    db.get_setting(&key).await.map_err(|e| format!("Failed to get setting: {}", e))
}

#[tauri::command]
pub async fn get_all_user_settings() -> Result<Vec<crate::core::database::UserSetting>, String> {
    let db = get_database().await?;
    db.get_all_settings().await.map_err(|e| format!("Failed to get all settings: {}", e))
}

#[tauri::command]
pub async fn delete_user_setting(key: String) -> Result<(), String> {
    let db = get_database().await?;
    db.delete_setting(&key).await.map_err(|e| format!("Failed to delete setting: {}", e))?;
    Ok(())
}

// === 处理历史相关命令 ===

#[tauri::command]
pub async fn add_processing_history(request: ProcessingHistoryRequest) -> Result<(), String> {
    let db = get_database().await?;
    
    let history = ProcessingHistory {
        id: Uuid::new_v4().to_string(),
        file_path: request.file_path,
        output_path: request.output_path,
        rule_ids: serde_json::to_string(&request.rule_ids).unwrap_or_default(),
        file_size: request.file_size,
        masked_count: request.masked_count,
        processing_time_ms: request.processing_time_ms,
        status: request.status,
        error_message: request.error_message,
        created_at: Utc::now(),
    };
    
    db.add_processing_history(&history).await.map_err(|e| format!("Failed to add processing history: {}", e))?;
    Ok(())
}

#[tauri::command]
pub async fn get_processing_history(limit: Option<i32>, offset: Option<i32>) -> Result<Vec<ProcessingHistory>, String> {
    let db = get_database().await?;
    db.get_processing_history(limit, offset).await.map_err(|e| format!("Failed to get processing history: {}", e))
}

#[tauri::command]
pub async fn get_statistics() -> Result<serde_json::Value, String> {
    let db = get_database().await?;
    db.get_statistics().await.map_err(|e| format!("Failed to get statistics: {}", e))
}

// === 数据库维护命令 ===

#[tauri::command]
pub async fn initialize_database() -> Result<(), String> {
    let _db = get_database().await?;
    Ok(())
}

#[tauri::command]
pub async fn get_database_info() -> Result<serde_json::Value, String> {
    let db = get_database().await?;
    
    // 获取各表的记录数
    let log_count = db.get_logs(Some(1), Some(0), None).await
        .map(|_| "available")
        .unwrap_or("error");
        
    Ok(serde_json::json!({
        "status": "connected",
        "log_table": log_count,
        "initialized_at": Utc::now()
    }))
}