use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct SandboxFile {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub modified: String,
}

// 简单的PIN存储（实际应用中应该使用更安全的方式）
use std::sync::Mutex;
use std::sync::LazyLock;

static STORED_PIN: LazyLock<Mutex<Option<String>>> = LazyLock::new(|| Mutex::new(None));

#[tauri::command]
pub async fn verify_pin(pin: String) -> Result<bool, String> {
    let stored_pin = STORED_PIN.lock().map_err(|_| "Failed to access PIN storage")?;
    match stored_pin.as_ref() {
        Some(stored) => Ok(stored == &pin),
        None => Ok(false), // 没有设置PIN时返回false
    }
}

#[tauri::command]
pub async fn set_pin(pin: String) -> Result<(), String> {
    if pin.len() < 4 {
        return Err("PIN must be at least 4 characters".to_string());
    }
    
    let mut stored_pin = STORED_PIN.lock().map_err(|_| "Failed to access PIN storage")?;
    *stored_pin = Some(pin);
    Ok(())
}

#[tauri::command]
pub async fn list_sandbox_files() -> Result<Vec<SandboxFile>, String> {
    // 这里应该基于用户设置的输出目录来列出文件
    // 暂时返回空列表，实际实现需要从前端传递输出目录路径
    Ok(vec![])
}

#[tauri::command]
pub async fn list_files_in_directory(directory: String) -> Result<Vec<SandboxFile>, String> {
    let dir_path = Path::new(&directory);
    
    if !dir_path.exists() {
        return Ok(vec![]);
    }
    
    if !dir_path.is_dir() {
        return Err("Path is not a directory".to_string());
    }
    
    let mut files = Vec::new();
    
    match std::fs::read_dir(dir_path) {
        Ok(entries) => {
            for entry in entries {
                match entry {
                    Ok(entry) => {
                        let path = entry.path();
                        if path.is_file() {
                            let name = path.file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("unknown")
                                .to_string();
                            
                            let size = match entry.metadata() {
                                Ok(metadata) => metadata.len(),
                                Err(_) => 0,
                            };
                            
                            let modified = match entry.metadata()
                                .and_then(|m| m.modified()) {
                                Ok(time) => {
                                    match time.duration_since(std::time::UNIX_EPOCH) {
                                        Ok(duration) => {
                                            let timestamp = duration.as_secs();
                                            format_timestamp(timestamp)
                                        },
                                        Err(_) => "Unknown".to_string(),
                                    }
                                },
                                Err(_) => "Unknown".to_string(),
                            };
                            
                            files.push(SandboxFile {
                                name,
                                path: path.to_string_lossy().to_string(),
                                size,
                                modified,
                            });
                        }
                    },
                    Err(_) => continue,
                }
            }
        },
        Err(e) => return Err(format!("Failed to read directory: {}", e)),
    }
    
    // 按名称排序
    files.sort_by(|a, b| a.name.cmp(&b.name));
    
    Ok(files)
}

#[tauri::command]
pub async fn export_sandbox(_file_name: String, _dest_path: String, _passphrase: String) -> Result<(), String> {
    // TODO: 实现文件导出功能
    Err("Export functionality not implemented yet".to_string())
}

#[tauri::command]
pub async fn import_sandbox(_src_path: String, _passphrase: String) -> Result<SandboxFile, String> {
    // TODO: 实现文件导入功能
    Err("Import functionality not implemented yet".to_string())
}

/// 格式化时间戳为可读格式
fn format_timestamp(timestamp: u64) -> String {
    use std::time::UNIX_EPOCH;
    
    let system_time = UNIX_EPOCH + std::time::Duration::from_secs(timestamp);
    
    // 简单的时间格式化（实际应用中可能需要更复杂的格式化）
    match system_time.elapsed() {
        Ok(elapsed) => {
            let days = elapsed.as_secs() / 86400;
            if days > 0 {
                format!("{} 天前", days)
            } else {
                let hours = elapsed.as_secs() / 3600;
                if hours > 0 {
                    format!("{} 小时前", hours)
                } else {
                    let minutes = elapsed.as_secs() / 60;
                    if minutes > 0 {
                        format!("{} 分钟前", minutes)
                    } else {
                        "刚刚".to_string()
                    }
                }
            }
        },
        Err(_) => {
            // 时间戳在未来，直接显示日期
            "未来时间".to_string()
        }
    }
}
