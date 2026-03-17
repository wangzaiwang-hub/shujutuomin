use sqlx::{sqlite::SqlitePool, Row};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::path::PathBuf;
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub level: String,
    pub message: String,
    pub details: Option<String>,
    pub file_path: Option<String>,
    pub operation_type: Option<String>,
    pub user_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSetting {
    pub key: String,
    pub value: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingHistory {
    pub id: String,
    pub file_path: String,
    pub output_path: String,
    pub rule_ids: String, // JSON array as string
    pub file_size: i64,
    pub masked_count: i32,
    pub processing_time_ms: i64,
    pub status: String, // "success", "failed", "cancelled"
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
}

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    /// 初始化数据库连接
    pub async fn new() -> Result<Self> {
        let db_path = get_database_path()?;
        
        // 确保数据库目录存在
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let database_url = format!("sqlite:{}", db_path.display());
        let pool = SqlitePool::connect(&database_url).await?;
        
        let db = Database { pool };
        db.init_tables().await?;
        
        Ok(db)
    }
    
    /// 创建数据库表
    async fn init_tables(&self) -> Result<()> {
        // 日志表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS logs (
                id TEXT PRIMARY KEY,
                timestamp DATETIME NOT NULL,
                level TEXT NOT NULL,
                message TEXT NOT NULL,
                details TEXT,
                file_path TEXT,
                operation_type TEXT,
                user_id TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&self.pool)
        .await?;
        
        // 用户设置表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS user_settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&self.pool)
        .await?;
        
        // 处理历史表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS processing_history (
                id TEXT PRIMARY KEY,
                file_path TEXT NOT NULL,
                output_path TEXT NOT NULL,
                rule_ids TEXT NOT NULL,
                file_size INTEGER NOT NULL,
                masked_count INTEGER NOT NULL,
                processing_time_ms INTEGER NOT NULL,
                status TEXT NOT NULL,
                error_message TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&self.pool)
        .await?;
        
        // 创建索引
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_logs_timestamp ON logs(timestamp)")
            .execute(&self.pool)
            .await?;
            
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_logs_level ON logs(level)")
            .execute(&self.pool)
            .await?;
            
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_processing_history_created_at ON processing_history(created_at)")
            .execute(&self.pool)
            .await?;
        
        Ok(())
    }
    
    // === 日志操作 ===
    
    /// 添加日志条目
    pub async fn add_log(&self, entry: &LogEntry) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO logs (id, timestamp, level, message, details, file_path, operation_type, user_id)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&entry.id)
        .bind(&entry.timestamp)
        .bind(&entry.level)
        .bind(&entry.message)
        .bind(&entry.details)
        .bind(&entry.file_path)
        .bind(&entry.operation_type)
        .bind(&entry.user_id)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    /// 获取日志列表
    pub async fn get_logs(&self, limit: Option<i32>, offset: Option<i32>, level_filter: Option<&str>) -> Result<Vec<LogEntry>> {
        let limit = limit.unwrap_or(100);
        let offset = offset.unwrap_or(0);
        
        let query = if let Some(level) = level_filter {
            sqlx::query(
                r#"
                SELECT id, timestamp, level, message, details, file_path, operation_type, user_id
                FROM logs 
                WHERE level = ?
                ORDER BY timestamp DESC 
                LIMIT ? OFFSET ?
                "#,
            )
            .bind(level)
            .bind(limit)
            .bind(offset)
        } else {
            sqlx::query(
                r#"
                SELECT id, timestamp, level, message, details, file_path, operation_type, user_id
                FROM logs 
                ORDER BY timestamp DESC 
                LIMIT ? OFFSET ?
                "#,
            )
            .bind(limit)
            .bind(offset)
        };
        
        let rows = query.fetch_all(&self.pool).await?;
        
        let mut logs = Vec::new();
        for row in rows {
            logs.push(LogEntry {
                id: row.get("id"),
                timestamp: row.get("timestamp"),
                level: row.get("level"),
                message: row.get("message"),
                details: row.get("details"),
                file_path: row.get("file_path"),
                operation_type: row.get("operation_type"),
                user_id: row.get("user_id"),
            });
        }
        
        Ok(logs)
    }
    
    /// 获取日志数量
    pub async fn get_logs_count(&self, level_filter: Option<&str>) -> Result<i64> {
        let count: i64 = if let Some(level) = level_filter {
            sqlx::query_scalar("SELECT COUNT(*) FROM logs WHERE level = ?")
                .bind(level)
                .fetch_one(&self.pool)
                .await?
        } else {
            sqlx::query_scalar("SELECT COUNT(*) FROM logs")
                .fetch_one(&self.pool)
                .await?
        };
        
        Ok(count)
    }
    
    /// 清空日志
    pub async fn clear_logs(&self) -> Result<()> {
        sqlx::query("DELETE FROM logs").execute(&self.pool).await?;
        Ok(())
    }
    
    /// 删除旧日志（保留最近N天）
    pub async fn cleanup_old_logs(&self, days: i32) -> Result<u64> {
        let cutoff = Utc::now() - chrono::Duration::days(days as i64);
        
        let result = sqlx::query("DELETE FROM logs WHERE timestamp < ?")
            .bind(cutoff)
            .execute(&self.pool)
            .await?;
            
        Ok(result.rows_affected())
    }
    
    // === 用户设置操作 ===
    
    /// 保存用户设置
    pub async fn save_setting(&self, key: &str, value: &str) -> Result<()> {
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO user_settings (key, value, updated_at)
            VALUES (?, ?, ?)
            "#,
        )
        .bind(key)
        .bind(value)
        .bind(Utc::now())
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    /// 获取用户设置
    pub async fn get_setting(&self, key: &str) -> Result<Option<String>> {
        let row = sqlx::query("SELECT value FROM user_settings WHERE key = ?")
            .bind(key)
            .fetch_optional(&self.pool)
            .await?;
            
        Ok(row.map(|r| r.get("value")))
    }
    
    /// 获取所有用户设置
    pub async fn get_all_settings(&self) -> Result<Vec<UserSetting>> {
        let rows = sqlx::query("SELECT key, value, updated_at FROM user_settings ORDER BY key")
            .fetch_all(&self.pool)
            .await?;
            
        let mut settings = Vec::new();
        for row in rows {
            settings.push(UserSetting {
                key: row.get("key"),
                value: row.get("value"),
                updated_at: row.get("updated_at"),
            });
        }
        
        Ok(settings)
    }
    
    /// 删除用户设置
    pub async fn delete_setting(&self, key: &str) -> Result<()> {
        sqlx::query("DELETE FROM user_settings WHERE key = ?")
            .bind(key)
            .execute(&self.pool)
            .await?;
            
        Ok(())
    }
    
    // === 处理历史操作 ===
    
    /// 添加处理历史记录
    pub async fn add_processing_history(&self, history: &ProcessingHistory) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO processing_history 
            (id, file_path, output_path, rule_ids, file_size, masked_count, processing_time_ms, status, error_message, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&history.id)
        .bind(&history.file_path)
        .bind(&history.output_path)
        .bind(&history.rule_ids)
        .bind(history.file_size)
        .bind(history.masked_count)
        .bind(history.processing_time_ms)
        .bind(&history.status)
        .bind(&history.error_message)
        .bind(&history.created_at)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    /// 获取处理历史
    pub async fn get_processing_history(&self, limit: Option<i32>, offset: Option<i32>) -> Result<Vec<ProcessingHistory>> {
        let limit = limit.unwrap_or(50);
        let offset = offset.unwrap_or(0);
        
        let rows = sqlx::query(
            r#"
            SELECT id, file_path, output_path, rule_ids, file_size, masked_count, 
                   processing_time_ms, status, error_message, created_at
            FROM processing_history 
            ORDER BY created_at DESC 
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;
        
        let mut history = Vec::new();
        for row in rows {
            history.push(ProcessingHistory {
                id: row.get("id"),
                file_path: row.get("file_path"),
                output_path: row.get("output_path"),
                rule_ids: row.get("rule_ids"),
                file_size: row.get("file_size"),
                masked_count: row.get("masked_count"),
                processing_time_ms: row.get("processing_time_ms"),
                status: row.get("status"),
                error_message: row.get("error_message"),
                created_at: row.get("created_at"),
            });
        }
        
        Ok(history)
    }
    
    /// 获取统计信息
    pub async fn get_statistics(&self) -> Result<serde_json::Value> {
        // 总处理文件数
        let total_files: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM processing_history")
            .fetch_one(&self.pool)
            .await?;
            
        // 成功处理文件数
        let successful_files: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM processing_history WHERE status = 'success'")
            .fetch_one(&self.pool)
            .await?;
            
        // 总脱敏数量
        let total_masked: i64 = sqlx::query_scalar("SELECT COALESCE(SUM(masked_count), 0) FROM processing_history WHERE status = 'success'")
            .fetch_one(&self.pool)
            .await?;
            
        // 总处理时间
        let total_time: i64 = sqlx::query_scalar("SELECT COALESCE(SUM(processing_time_ms), 0) FROM processing_history WHERE status = 'success'")
            .fetch_one(&self.pool)
            .await?;
            
        // 最近7天的处理数量
        let recent_cutoff = Utc::now() - chrono::Duration::days(7);
        let recent_files: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM processing_history WHERE created_at > ?")
            .bind(recent_cutoff)
            .fetch_one(&self.pool)
            .await?;
        
        Ok(serde_json::json!({
            "total_files": total_files,
            "successful_files": successful_files,
            "failed_files": total_files - successful_files,
            "total_masked_items": total_masked,
            "total_processing_time_ms": total_time,
            "recent_files_7days": recent_files,
            "success_rate": if total_files > 0 { successful_files as f64 / total_files as f64 * 100.0 } else { 0.0 }
        }))
    }
}

/// 获取跨平台数据库路径
fn get_database_path() -> Result<PathBuf> {
    let app_data_dir = get_cross_platform_app_data_dir();
    Ok(app_data_dir.join("cheersai-vault.db"))
}

/// 获取跨平台应用数据目录
fn get_cross_platform_app_data_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        dirs_next::data_dir()
            .unwrap_or_else(|| PathBuf::from(std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string())))
            .join("CheersAI Vault")
    }
    
    #[cfg(target_os = "macos")]
    {
        dirs_next::data_dir()
            .unwrap_or_else(|| PathBuf::from("~/Library/Application Support"))
            .join("CheersAI Vault")
    }
    
    #[cfg(target_os = "linux")]
    {
        dirs_next::data_dir()
            .unwrap_or_else(|| PathBuf::from("~/.local/share"))
            .join("CheersAI Vault")
    }
    
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        PathBuf::from("./CheersAI Vault")
    }
}