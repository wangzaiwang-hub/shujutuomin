use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct WebviewOptions {
    pub url: String,
    pub title: Option<String>,
    pub width: Option<f64>,
    pub height: Option<f64>,
}

#[tauri::command]
pub async fn open_webview_window(
    app: AppHandle,
    options: WebviewOptions,
) -> Result<String, String> {
    let window_label = format!("webview_{}", chrono::Utc::now().timestamp_millis());
    let title = options.title.unwrap_or_else(|| "CheersAI Cloud".to_string());
    let width = options.width.unwrap_or(1200.0);
    let height = options.height.unwrap_or(800.0);

    println!("Opening webview window: {} -> {}", window_label, options.url);

    // 创建新的 WebView 窗口，配置更多浏览器特性
    let _webview_window = WebviewWindowBuilder::new(
        &app,
        &window_label,
        WebviewUrl::External(options.url.parse().map_err(|e| format!("Invalid URL: {}", e))?)
    )
    .title(&title)
    .inner_size(width, height)
    .center()
    .resizable(true)
    .fullscreen(false)
    .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
    .accept_first_mouse(true)
    .build()
    .map_err(|e| format!("Failed to create webview window: {}", e))?;

    println!("Webview window created successfully: {}", window_label);

    Ok(window_label)
}

#[tauri::command]
pub async fn navigate_webview(
    app: AppHandle,
    label: String,
    url: String,
) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(&label) {
        let parsed_url = url.parse().map_err(|e| format!("Invalid URL: {}", e))?;
        window.navigate(parsed_url)
            .map_err(|e| format!("Failed to navigate: {}", e))?;
        Ok(())
    } else {
        Err(format!("Window not found: {}", label))
    }
}

#[tauri::command]
pub async fn webview_reload(
    app: AppHandle,
    label: String,
) -> Result<(), String> {
    if let Some(webview) = app.get_webview_window(&label) {
        // Reload by navigating to current URL
        if let Ok(current_url) = webview.url() {
            webview.navigate(current_url)
                .map_err(|e| format!("Failed to reload: {}", e))?;
        }
        Ok(())
    } else {
        Err(format!("Window not found: {}", label))
    }
}

#[tauri::command]
pub async fn close_webview_window(
    app: AppHandle,
    label: String,
) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(&label) {
        window.close().map_err(|e| format!("Failed to close window: {}", e))?;
        Ok(())
    } else {
        Err(format!("Window not found: {}", label))
    }
}

#[tauri::command]
pub async fn get_webview_url(
    app: AppHandle,
    label: String,
) -> Result<String, String> {
    if let Some(window) = app.get_webview_window(&label) {
        window.url()
            .map(|u| u.to_string())
            .map_err(|e| format!("Failed to get URL: {}", e))
    } else {
        Err(format!("Window not found: {}", label))
    }
}

#[tauri::command]
pub async fn webview_eval_script(
    app: AppHandle,
    label: String,
    script: String,
) -> Result<(), String> {
    if let Some(webview) = app.get_webview_window(&label) {
        webview.eval(&script)
            .map_err(|e| format!("Failed to evaluate script: {}", e))?;
        Ok(())
    } else {
        Err(format!("Window not found: {}", label))
    }
}

#[tauri::command]
pub async fn navigate_main_window_with_button(
    app: AppHandle,
    url: String,
    return_url: String,
) -> Result<(), String> {
    // 获取主窗口
    if let Some(main_window) = app.get_webview_window("main") {
        // 导航到目标 URL
        let parsed_url = url.parse().map_err(|e| format!("Invalid URL: {}", e))?;
        main_window.navigate(parsed_url)
            .map_err(|e| format!("Failed to navigate: {}", e))?;
        
        // 等待页面加载后注入返回按钮脚本
        let inject_script = format!(r#"
            (function() {{
                // 等待 DOM 加载完成
                if (document.readyState === 'loading') {{
                    document.addEventListener('DOMContentLoaded', injectButton);
                }} else {{
                    injectButton();
                }}
                
                function injectButton() {{
                    // 避免重复注入
                    if (document.getElementById('cheersai-return-button')) {{
                        return;
                    }}
                    
                    // 创建按钮容器
                    const buttonContainer = document.createElement('div');
                    buttonContainer.id = 'cheersai-return-button';
                    buttonContainer.style.cssText = `
                        position: fixed;
                        top: 20px;
                        right: 20px;
                        z-index: 2147483647;
                        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
                    `;
                    
                    // 创建按钮
                    const button = document.createElement('button');
                    button.style.cssText = `
                        display: flex;
                        align-items: center;
                        gap: 8px;
                        padding: 12px 20px;
                        background: rgba(255, 255, 255, 0.95);
                        color: #374151;
                        border: 1px solid #d1d5db;
                        border-radius: 8px;
                        font-size: 14px;
                        font-weight: 500;
                        cursor: pointer;
                        box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06);
                        transition: all 0.2s ease;
                        backdrop-filter: blur(8px);
                    `;
                    
                    button.innerHTML = `
                        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                            <path d="M15 18l-6-6 6-6"/>
                        </svg>
                        <span>返回脱敏</span>
                    `;
                    
                    // 悬停效果
                    button.addEventListener('mouseenter', function() {{
                        this.style.background = 'rgba(255, 255, 255, 1)';
                        this.style.boxShadow = '0 10px 15px -3px rgba(0, 0, 0, 0.1), 0 4px 6px -2px rgba(0, 0, 0, 0.05)';
                        this.style.transform = 'translateY(-1px)';
                    }});
                    
                    button.addEventListener('mouseleave', function() {{
                        this.style.background = 'rgba(255, 255, 255, 0.95)';
                        this.style.boxShadow = '0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06)';
                        this.style.transform = 'translateY(0)';
                    }});
                    
                    // 点击返回
                    button.addEventListener('click', function() {{
                        window.location.href = '{}';
                    }});
                    
                    buttonContainer.appendChild(button);
                    document.body.appendChild(buttonContainer);
                    
                    console.log('CheersAI: 返回按钮已添加');
                }}
            }})();
        "#, return_url);
        
        // 延迟注入，等待页面开始加载
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        main_window.eval(&inject_script)
            .map_err(|e| format!("Failed to inject button script: {}", e))?;
        
        Ok(())
    } else {
        Err("Main window not found".to_string())
    }
}
