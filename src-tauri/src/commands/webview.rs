use tauri::{AppHandle, Manager, Emitter, WebviewUrl, WebviewWindowBuilder};
use serde::{Deserialize, Serialize};

#[tauri::command]
pub async fn navigate_to_local(app: AppHandle) -> Result<(), String> {
    if let Some(main_window) = app.get_webview_window("main") {
        let local_url = "tauri://localhost/process".parse()
            .map_err(|e| format!("Invalid URL: {}", e))?;
        main_window.navigate(local_url)
            .map_err(|e| format!("Failed to navigate: {}", e))?;
        Ok(())
    } else {
        Err("Main window not found".to_string())
    }
}

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
    _return_url: String,
) -> Result<(), String> {
    // 获取主窗口
    let main_window = app.get_webview_window("main")
        .ok_or("Main window not found".to_string())?;

    // 记录当前 URL origin，用于返回（dev: http://localhost:1420, prod: tauri://localhost）
    let return_origin = if let Ok(current_url) = main_window.url() {
        let origin = current_url.origin().unicode_serialization();
        println!("📍 Saved return origin: {}", origin);
        origin
    } else {
        "http://localhost:1420".to_string()
    };

    // 导航到目标外部 URL
    let parsed_url = url.parse().map_err(|e| format!("Invalid URL: {}", e))?;
    main_window.navigate(parsed_url)
        .map_err(|e| format!("Failed to navigate: {}", e))?;

    // 注入返回按钮脚本（按钮点击后修改 URL hash 作为信号）
    let inject_script = r#"
        (function() {
            if (document.readyState === 'loading') {
                document.addEventListener('DOMContentLoaded', injectButton);
            } else {
                injectButton();
            }
            
            function injectButton() {
                if (!window.location.href.includes('7smile.dlithink.com')) return;
                if (document.getElementById('cheersai-return-button')) return;
                
                const buttonContainer = document.createElement('div');
                buttonContainer.id = 'cheersai-return-button';
                buttonContainer.style.cssText = `
                    position: fixed;
                    top: 80px;
                    left: 18px;
                    z-index: 2147483647;
                    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
                `;
                
                const button = document.createElement('button');
                button.style.cssText = `
                    display: flex;
                    align-items: center;
                    gap: 12px;
                    padding: 10px 16px;
                    background: #4285f4;
                    color: white;
                    border: none;
                    border-radius: 12px;
                    font-size: 15px;
                    font-weight: 500;
                    cursor: pointer;
                    box-shadow: 0 4px 12px rgba(66, 133, 244, 0.3);
                    transition: all 0.15s ease;
                    width: 215px;
                `;
                
                button.innerHTML = `
                    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <path d="M15 18l-6-6 6-6"/>
                    </svg>
                    <span>数据脱敏</span>
                `;
                
                button.addEventListener('mouseenter', function() {
                    this.style.background = '#5a95f5';
                    this.style.boxShadow = '0 6px 16px rgba(66, 133, 244, 0.4)';
                    this.style.transform = 'translateY(-1px)';
                });
                
                button.addEventListener('mouseleave', function() {
                    this.style.background = '#4285f4';
                    this.style.boxShadow = '0 4px 12px rgba(66, 133, 244, 0.3)';
                    this.style.transform = 'translateY(0)';
                });
                
                button.addEventListener('click', function() {
                    console.log('Return button clicked');
                    this.innerHTML = '<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M15 18l-6-6 6-6"/></svg><span>正在返回...</span>';
                    this.style.opacity = '0.7';
                    this.disabled = true;
                    window.location.hash = '#__cheersai_navigate_back__';
                });
                
                buttonContainer.appendChild(button);
                document.body.appendChild(buttonContainer);
                console.log('CheersAI: 返回按钮已添加');
            }
        })();
    "#;

    // 多次注入确保成功
    let window_for_inject = main_window.clone();
    tokio::spawn(async move {
        for i in 0..5 {
            let delay = if i == 0 { 1500 } else { 2000 };
            tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
            if window_for_inject.eval(inject_script).is_ok() {
                println!("✓ Button inject attempt {} succeeded", i + 1);
            }
        }
    });

    // 启动检查任务 - 检测 URL hash 变化
    let window_clone = main_window.clone();
    tokio::spawn(async move {
        let mut check_count = 0;
        
        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
            check_count += 1;
            
            if let Ok(current_url) = window_clone.url() {
                let url_str = current_url.to_string();
                
                if url_str.contains("#__cheersai_navigate_back__") {
                    println!("🔄 Navigate back hash detected!");
                    
                    let return_url = format!("{}/process", return_origin);
                    println!("🔄 Return URL: {}", return_url);
                    
                    // 第1步: 先导航到 about:blank 清除外部页面安全上下文
                    if let Ok(blank_url) = "about:blank".parse() {
                        let _ = window_clone.navigate(blank_url);
                        println!("✓ Step 1: Navigated to about:blank");
                    }
                    
                    // 等待 about:blank 加载完成
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                    
                    // 第2步: 从 about:blank 导航到本地页面
                    if let Ok(local_url) = return_url.parse() {
                        let _ = window_clone.navigate(local_url);
                        println!("✓ Step 2: Navigated to {}", return_url);
                    }
                    
                    // 检查结果
                    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
                    if let Ok(final_url) = window_clone.url() {
                        println!("📍 Final URL: {}", final_url);
                    }
                    
                    break;
                }
                
                if !url_str.starts_with("https://7smile.dlithink.com") 
                    && !url_str.starts_with("http://7smile.dlithink.com") 
                    && !url_str.starts_with("about:blank") {
                    println!("✓ No longer on external page: {}", url_str);
                    break;
                }
            }
            
            if check_count > 600 {
                println!("⚠ Navigation check timeout");
                break;
            }
        }
    });

    Ok(())
}
