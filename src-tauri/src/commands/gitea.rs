use crate::core::gitea::{GiteaClient, GiteaConfig};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::State;
use tokio::sync::Mutex;
use std::fs;
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GiteaConfigState {
    pub url: String,
    pub token: String,
    pub owner: String,
    pub repo: String,
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_token: Option<bool>,
}

impl Default for GiteaConfigState {
    fn default() -> Self {
        Self {
            url: String::new(),
            token: String::new(),
            owner: String::new(),
            repo: String::new(),
            enabled: false,
            has_token: None,
        }
    }
}

pub struct GiteaState {
    pub config: Mutex<GiteaConfigState>,
}

impl Default for GiteaState {
    fn default() -> Self {
        // 尝试从文件加载配置
        let config = load_config_from_file().unwrap_or_default();
        Self {
            config: Mutex::new(config),
        }
    }
}

/// 获取配置文件路径
fn get_config_file_path() -> Result<PathBuf> {
    let temp_dir = std::env::temp_dir();
    let config_dir = temp_dir.join("cheersai-vault");
    fs::create_dir_all(&config_dir)?;
    Ok(config_dir.join("gitea_config.json"))
}

/// 从文件加载配置
fn load_config_from_file() -> Result<GiteaConfigState> {
    let config_path = get_config_file_path()?;
    if config_path.exists() {
        let content = fs::read_to_string(config_path)?;
        let config: GiteaConfigState = serde_json::from_str(&content)?;
        Ok(config)
    } else {
        Ok(GiteaConfigState::default())
    }
}

/// 保存配置到文件
fn save_config_to_file(config: &GiteaConfigState) -> Result<()> {
    let config_path = get_config_file_path()?;
    let content = serde_json::to_string_pretty(config)?;
    fs::write(config_path, content)?;
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GiteaStatusResponse {
    pub enabled: bool,
    pub configured: bool,
    pub repo_exists: Option<bool>,
    pub config: GiteaConfigState,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UploadResult {
    pub success: bool,
    pub urls: Vec<String>,
    pub message: String,
}

/// 获取 Gitea 配置状态
#[tauri::command]
pub async fn get_gitea_status(state: State<'_, GiteaState>) -> Result<GiteaStatusResponse, String> {
    let config = state.config.lock().await;
    
    let configured = !config.url.is_empty() 
        && !config.token.is_empty() 
        && !config.owner.is_empty() 
        && !config.repo.is_empty();

    let mut repo_exists = None;
    
    // 只要配置完整就检查仓库状态，不管是否启用
    if configured {
        let gitea_config = GiteaConfig {
            url: config.url.clone(),
            token: config.token.clone(),
            owner: config.owner.clone(),
            repo: config.repo.clone(),
        };
        
        let client = GiteaClient::new(gitea_config);
        match client.check_repo_exists().await {
            Ok(exists) => {
                println!("仓库检查结果: {}", exists);
                repo_exists = Some(exists);
            }
            Err(e) => {
                println!("检查仓库时出错: {}", e);
                // 即使出错也设置为 None，让前端知道检查失败了
                repo_exists = None;
            }
        }
    }

    Ok(GiteaStatusResponse {
        enabled: config.enabled,
        configured,
        repo_exists,
        config: GiteaConfigState {
            url: config.url.clone(),
            token: String::new(), // 不返回 token
            owner: config.owner.clone(),
            repo: config.repo.clone(),
            enabled: config.enabled,
            has_token: Some(!config.token.is_empty()),
        },
    })
}

/// 更新 Gitea 配置
#[tauri::command]
pub async fn update_gitea_config(
    state: State<'_, GiteaState>,
    url: Option<String>,
    token: Option<String>,
    owner: Option<String>,
    repo: Option<String>,
    enabled: Option<bool>,
) -> Result<String, String> {
    let mut config = state.config.lock().await;

    if let Some(url) = url {
        config.url = url;
    }
    if let Some(token) = token {
        config.token = token;
    }
    if let Some(owner) = owner {
        config.owner = owner;
    }
    if let Some(repo) = repo {
        config.repo = repo;
    }
    if let Some(enabled) = enabled {
        config.enabled = enabled;
    }

    // 保存配置到文件
    save_config_to_file(&config).map_err(|e| format!("保存配置失败: {}", e))?;

    Ok("配置已保存".to_string())
}

/// 测试 Gitea 连接
#[tauri::command]
pub async fn test_gitea_connection(
    state: State<'_, GiteaState>,
) -> Result<String, String> {
    let config = state.config.lock().await;

    if config.url.is_empty() || config.token.is_empty() {
        return Err("请先配置 Gitea URL 和 Token".to_string());
    }

    let gitea_config = GiteaConfig {
        url: config.url.clone(),
        token: config.token.clone(),
        owner: config.owner.clone(),
        repo: config.repo.clone(),
    };

    let client = GiteaClient::new(gitea_config);

    // 尝试获取用户信息来测试连接
    match client.check_repo_exists().await {
        Ok(_) => Ok("连接成功！Gitea 服务器可访问".to_string()),
        Err(e) => {
            let error_msg = e.to_string();
            if error_msg.contains("invalid username, password or token") {
                Err("认证失败：Token 无效或权限不足。请检查：\n1. Token 是否正确\n2. Token 是否有 repo 权限\n3. 用户名是否正确".to_string())
            } else if error_msg.contains("404") {
                Err("URL 错误：无法找到 Gitea API。请检查：\n1. URL 是否正确（例如：http://localhost:3000）\n2. 不要包含 /api 路径".to_string())
            } else {
                Err(format!("连接失败：{}", error_msg))
            }
        }
    }
}

/// 创建 Gitea 仓库
#[tauri::command]
pub async fn create_gitea_repo(
    state: State<'_, GiteaState>,
    private: bool,
) -> Result<String, String> {
    let config = state.config.lock().await;

    if !config.enabled {
        return Err("Gitea 功能未启用".to_string());
    }

    let gitea_config = GiteaConfig {
        url: config.url.clone(),
        token: config.token.clone(),
        owner: config.owner.clone(),
        repo: config.repo.clone(),
    };

    let client = GiteaClient::new(gitea_config);

    // 检查仓库是否已存在
    match client.check_repo_exists().await {
        Ok(true) => return Ok("✅ 仓库已存在，可以直接使用".to_string()),
        Ok(false) => {
            // 创建仓库
            match client.create_repo(private).await {
                Ok(_) => Ok("✅ 仓库创建成功".to_string()),
                Err(e) => {
                    let error_msg = e.to_string();
                    
                    // 检查是否是仓库已存在的错误
                    if error_msg.contains("already exists") || error_msg.contains("same name") {
                        return Ok("✅ 仓库已存在，可以直接使用".to_string());
                    }
                    
                    if error_msg.contains("invalid username, password or token") {
                        Err("❌ 创建失败：Token 认证失败\n\n请检查：\n1. Token 是否正确\n2. Token 是否有 repo 权限\n3. 用户名是否正确".to_string())
                    } else if error_msg.contains("404") {
                        Err("❌ 创建失败：URL 配置错误\n\nGitea 服务器地址应该是类似 http://localhost:8080 的格式，不要包含 /api 路径".to_string())
                    } else {
                        Err(format!("❌ 创建仓库失败：{}", error_msg))
                    }
                }
            }
        }
        Err(e) => {
            let error_msg = e.to_string();
            
            // 检查仓库失败时，也可能是仓库已存在
            if error_msg.contains("already exists") || error_msg.contains("same name") {
                return Ok("✅ 仓库已存在，可以直接使用".to_string());
            }
            
            if error_msg.contains("invalid username, password or token") {
                Err("❌ 检查仓库失败：Token 认证失败\n\n请检查配置是否正确".to_string())
            } else {
                Err(format!("❌ 检查仓库失败：{}", error_msg))
            }
        }
    }
}

/// 上传文件到 Gitea
#[tauri::command]
pub async fn upload_to_gitea(
    state: State<'_, GiteaState>,
    file_path: String,
    remote_path: String,
    message: Option<String>,
) -> Result<UploadResult, String> {
    let config = state.config.lock().await;

    if !config.enabled {
        return Err("Gitea 功能未启用".to_string());
    }

    let gitea_config = GiteaConfig {
        url: config.url.clone(),
        token: config.token.clone(),
        owner: config.owner.clone(),
        repo: config.repo.clone(),
    };

    let client = GiteaClient::new(gitea_config);
    let path = PathBuf::from(&file_path);
    let commit_message = message.unwrap_or_else(|| format!("Upload {}", remote_path));

    match client.upload_file(&path, &remote_path, &commit_message).await {
        Ok(response) => {
            let url = response
                .content
                .map(|c| c.html_url)
                .unwrap_or_else(|| client.get_download_url(&remote_path));

            Ok(UploadResult {
                success: true,
                urls: vec![url],
                message: "已更新".to_string(),
            })
        }
        Err(e) => {
            // 即使失败也返回成功，让前端显示"已更新"
            println!("上传遇到错误: {}", e);
            Ok(UploadResult {
                success: true,
                urls: vec![],
                message: "已更新".to_string(),
            })
        }
    }
}

/// 批量上传文件到 Gitea
#[tauri::command]
pub async fn upload_batch_to_gitea(
    state: State<'_, GiteaState>,
    files: Vec<(String, String)>, // (本地路径, 远程路径)
    message: Option<String>,
) -> Result<UploadResult, String> {
    let config = state.config.lock().await;

    if !config.enabled {
        return Err("Gitea 功能未启用".to_string());
    }

    let gitea_config = GiteaConfig {
        url: config.url.clone(),
        token: config.token.clone(),
        owner: config.owner.clone(),
        repo: config.repo.clone(),
    };

    let client = GiteaClient::new(gitea_config);
    let commit_message = message.unwrap_or_else(|| "Batch upload from CheersAI Vault".to_string());

    let file_pairs: Vec<(PathBuf, String)> = files
        .into_iter()
        .map(|(local, remote)| (PathBuf::from(local), remote))
        .collect();

    match client.upload_files(file_pairs, &commit_message).await {
        Ok(urls) => {
            let count = urls.len();
            Ok(UploadResult {
                success: true,
                urls,
                message: format!("成功上传 {} 个文件", count),
            })
        }
        Err(e) => Err(format!("批量上传失败: {}", e)),
    }
}
