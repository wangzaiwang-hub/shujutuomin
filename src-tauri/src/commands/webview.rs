use tauri::{AppHandle, Manager, Emitter, WebviewUrl, WebviewWindow, WebviewWindowBuilder};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::collections::HashMap;
use tokio::sync::{Mutex as TokioMutex, oneshot};
use base64::Engine as _;

const CLOUD_HOST: &str = "uat-desktop.cheersai.cloud";

// 全局导航锁，防止重复触发
static NAVIGATION_IN_PROGRESS: AtomicBool = AtomicBool::new(false);

// 编译时嵌入 64x64 缩略图
const SAFER_ICON_BYTES: &[u8] = include_bytes!("../../safer_small.png");

#[tauri::command]
pub async fn navigate_to_local(app: AppHandle, return_url: String) -> Result<(), String> {
    println!("🔙 Navigating back to local: {}", return_url);
    
    if let Some(main_window) = app.get_webview_window("main") {
        // 发送事件到前端，让前端处理路由
        let _ = app.emit("navigate-to-process", ());
        
        // 同时也尝试 JavaScript 导航作为备用
        let script = format!(r#"
            console.log('🔙 Returning to: {}');
            window.location.href = '{}';
        "#, return_url, return_url);
        
        main_window.eval(&script)
            .map_err(|e| format!("Failed to navigate: {}", e))?;
        
        println!("✅ Navigation command sent");
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

    // 创建新的 WebView 窗口，配置为全屏无边框模式
    let webview_window = WebviewWindowBuilder::new(
        &app,
        &window_label,
        WebviewUrl::External(options.url.parse().map_err(|e| format!("Invalid URL: {}", e))?)
    )
    .title(&title)
    .inner_size(width, height)
    .center()
    .resizable(true)
    .fullscreen(false)
    .decorations(true)  // 保留标题栏以便关闭窗口
    .maximized(true)    // 启动时最大化
    .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
    .accept_first_mouse(true)
    .build()
    .map_err(|e| format!("Failed to create webview window: {}", e))?;

    println!("Webview window created successfully: {}", window_label);

    // 注入浮动返回按钮
    let return_url = "http://localhost:1420/process";
    let icon_b64 = base64::engine::general_purpose::STANDARD.encode(SAFER_ICON_BYTES);
    let icon_data_url = format!("data:image/png;base64,{}", icon_b64);
    
    let inject_script = format!(r#"
        (function() {{
            window.__cheersai_nav_lock = window.__cheersai_nav_lock || false;

            if (document.readyState === 'loading') {{
                document.addEventListener('DOMContentLoaded', inject);
            }} else {{
                inject();
            }}

            function inject() {{
                var href = window.location.href;
                console.log('🔍 CheersAI FAB: Checking injection in new window...', href);

                // 只在外部页面注入
                if (href.indexOf('tauri://localhost') !== -1 ||
                    href.indexOf('localhost:1420') !== -1 ||
                    href.indexOf('127.0.0.1') !== -1) {{
                    console.log('❌ CheersAI FAB: Skipped - local page');
                    return;
                }}
                
                // 检查是否已经注入
                if (document.getElementById('__cheersai_fab')) {{
                    console.log('❌ CheersAI FAB: Already injected');
                    return;
                }}
                
                // 简化检查：只跳过明显的登录页面（URL 包含 login/signin）
                // 不再检查密码表单，因为很多页面都有登录表单但不是登录页
                var isLoginPage = href.indexOf('/login') !== -1 || href.indexOf('/signin') !== -1;
                if (isLoginPage) {{
                    console.log('❌ CheersAI FAB: Skipped - login page');
                    return;
                }}

                console.log('✅ CheersAI FAB: Injecting button in new window');

                var fab = document.createElement('div');
                fab.id = '__cheersai_fab';
                fab.style.cssText = 'position:fixed;top:40%;right:0px;z-index:2147483647;' +
                    'display:flex;align-items:center;cursor:pointer;user-select:none;' +
                    'transition:right 0.25s cubic-bezier(0.4,0,0.2,1),opacity 0.25s ease,box-shadow 0.25s ease;' +
                    'opacity:0.85;' +
                    'pointer-events:auto;';

                var circle = document.createElement('div');
                circle.style.cssText = 'width:56px;height:56px;min-width:56px;border-radius:50%;' +
                    'background:#fff;box-shadow:-2px 2px 12px rgba(0,0,0,0.15);' +
                    'display:flex;align-items:center;justify-content:center;overflow:hidden;' +
                    'transition:box-shadow 0.25s ease,transform 0.25s ease;';

                var img = document.createElement('img');
                img.src = '{icon_data_url}';
                img.style.cssText = 'width:40px;height:40px;border-radius:50%;pointer-events:none;';
                img.draggable = false;
                circle.appendChild(img);
                fab.appendChild(circle);

                var label = document.createElement('div');
                label.textContent = '\u6570\u636E\u5B89\u5168\u5C4B';
                label.style.cssText = 'margin-right:12px;padding:8px 16px;background:#fff;' +
                    'border-radius:24px;box-shadow:-2px 2px 12px rgba(0,0,0,0.1);' +
                    'font-size:14px;font-weight:500;color:#1f2937;white-space:nowrap;' +
                    'opacity:0;transform:translateX(8px);' +
                    'transition:opacity 0.2s ease,transform 0.2s ease;';
                fab.insertBefore(label, circle);

                var hideTimer = null;
                var isDragging = false;

                function snapToEdge() {{
                    if (window.__cheersai_nav_lock || isDragging) return;
                    fab.style.right = '0px';
                    fab.style.opacity = '0.85';
                    circle.style.boxShadow = '-2px 2px 12px rgba(0,0,0,0.15)';
                    circle.style.transform = 'scale(1)';
                    label.style.opacity = '0';
                    label.style.transform = 'translateX(8px)';
                }}

                function slideOut() {{
                    if (window.__cheersai_nav_lock || isDragging) return;
                    clearTimeout(hideTimer);
                    fab.style.right = '12px';
                    fab.style.opacity = '1';
                    circle.style.boxShadow = '0 6px 24px rgba(59,130,246,0.3)';
                    circle.style.transform = 'scale(1.08)';
                    label.style.opacity = '1';
                    label.style.transform = 'translateX(0)';
                }}

                function startHideTimer() {{
                    hideTimer = setTimeout(snapToEdge, 2000);
                }}

                fab.addEventListener('mouseenter', function() {{ slideOut(); }});
                fab.addEventListener('mouseleave', function() {{ startHideTimer(); }});

                fab.addEventListener('mousedown', function(e) {{
                    isDragging = false;
                    var startY = e.clientY;
                    var startTop = fab.getBoundingClientRect().top;
                    fab.style.transition = 'none';

                    function onMove(ev) {{
                        var dy = ev.clientY - startY;
                        if (Math.abs(dy) > 3) isDragging = true;
                        if (isDragging) {{
                            var newTop = Math.max(20, Math.min(window.innerHeight - 76, startTop + dy));
                            fab.style.top = newTop + 'px';
                        }}
                    }}
                    function onUp() {{
                        document.removeEventListener('mousemove', onMove);
                        document.removeEventListener('mouseup', onUp);
                        fab.style.transition = 'right 0.25s cubic-bezier(0.4,0,0.2,1),opacity 0.25s ease,box-shadow 0.25s ease';
                        if (isDragging) {{
                            startHideTimer();
                            setTimeout(function() {{ isDragging = false; }}, 50);
                        }}
                    }}
                    document.addEventListener('mousemove', onMove);
                    document.addEventListener('mouseup', onUp);
                }});

                setTimeout(function() {{
                    fab.style.right = '12px';
                    fab.style.opacity = '1';
                    label.style.opacity = '1';
                    label.style.transform = 'translateX(0)';
                }}, 300);
                setTimeout(snapToEdge, 2500);

                fab.addEventListener('click', function(e) {{
                    if (isDragging) return;
                    if (window.__cheersai_nav_lock) return;
                    window.__cheersai_nav_lock = true;
                    fab.style.transform = 'scale(0)';
                    fab.style.opacity = '0';
                    window.close();
                }});

                document.body.appendChild(fab);
                console.log('✅ CheersAI FAB: Button injected successfully in new window!');
                console.log('✅ CheersAI FAB: Button element:', fab);
                console.log('✅ CheersAI FAB: Button position:', fab.getBoundingClientRect());
            }}
        }})();
    "#, icon_data_url = icon_data_url);

    // 启动注入任务
    let window_clone = webview_window.clone();
    let script_clone = inject_script.clone();
    tokio::spawn(async move {
        let mut count = 0u32;
        loop {
            let delay = if count < 3 { 1500 } else { 3000 };
            tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
            count += 1;

            let url_str = match window_clone.url() {
                Ok(u) => u.to_string(),
                Err(_) => break,
            };

            if count <= 5 {
                println!("🔍 New window FAB inject check #{}: URL = {}", count, url_str);
            }

            // 只在外部页面注入
            if !url_str.contains("localhost") && !url_str.contains("tauri://") {
                if window_clone.eval(&script_clone).is_ok() {
                    if count <= 3 {
                        println!("✓ New window FAB inject #{}", count);
                    }
                } else if count <= 3 {
                    println!("⚠ New window FAB inject #{} failed", count);
                }
            }

            if count > 200 {
                println!("⚠ New window FAB inject loop timeout");
                break;
            }
        }
    });

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
    let main_window = app.get_webview_window("main")
        .ok_or("Main window not found".to_string())?;

    // 记录本地 origin 用于返回
    let return_origin = if let Ok(current_url) = main_window.url() {
        let origin = current_url.origin().unicode_serialization();
        println!("📍 Return origin: {}", origin);
        origin
    } else {
        "http://localhost:1420".to_string()
    };
    let return_url = format!("{}/#/process", return_origin);

    // 将图标转为 base64 data URL
    let icon_b64 = base64::engine::general_purpose::STANDARD.encode(SAFER_ICON_BYTES);
    let icon_data_url = format!("data:image/png;base64,{}", icon_b64);

    // 导航到目标外部 URL
    println!("🚀 Navigating to: {}", url);
    let parsed_url = url.parse().map_err(|e| format!("Invalid URL: {}", e))?;
    main_window.navigate(parsed_url)
        .map_err(|e| format!("Failed to navigate: {}", e))?;
    println!("✓ Navigation command sent");

    // ===== 注入悬浮圆形按钮脚本 =====
    // 核心改变：按钮点击后由 JS 直接 window.location.href 导航，不再用 hash 信号
    let inject_script = format!(r#"
        (function() {{
            window.__cheersai_nav_lock = window.__cheersai_nav_lock || false;

            if (document.readyState === 'loading') {{
                document.addEventListener('DOMContentLoaded', inject);
            }} else {{
                inject();
            }}

            function inject() {{
                var href = window.location.href;
                console.log('🔍 CheersAI FAB: Checking injection conditions...', href);

                // 只在目标外部页面注入
                if (href.indexOf('tauri://localhost') !== -1 ||
                    href.indexOf('localhost:1420') !== -1 ||
                    href.indexOf('127.0.0.1') !== -1) {{
                    console.log('❌ CheersAI FAB: Skipped - local page');
                    return;
                }}
                if (href.indexOf('{cloud_host}') === -1) {{
                    console.log('❌ CheersAI FAB: Skipped - not target domain, current:', href);
                    return;
                }}
                
                // 检查是否已经注入
                if (document.getElementById('__cheersai_fab')) {{
                    console.log('❌ CheersAI FAB: Already injected');
                    return;
                }}
                
                // 简化检查：只跳过明显的登录页面（URL 包含 login/signin）
                var isLoginPage = href.indexOf('/login') !== -1 || href.indexOf('/signin') !== -1;
                if (isLoginPage) {{
                    console.log('❌ CheersAI FAB: Skipped - login page');
                    return;
                }}

                console.log('✅ CheersAI FAB: All checks passed, injecting button...');

                // --- 悬浮圆形按钮（吸附右侧、可拖动、悬停显示文字） ---
                var fab = document.createElement('div');
                fab.id = '__cheersai_fab';
                fab.style.cssText = 'position:fixed;top:40%;right:0px;z-index:2147483647;' +
                    'display:flex;align-items:center;cursor:pointer;user-select:none;' +
                    'transition:right 0.25s cubic-bezier(0.4,0,0.2,1),opacity 0.25s ease,box-shadow 0.25s ease;' +
                    'opacity:0.85;' +
                    'pointer-events:auto;';

                // 圆形图标容器
                var circle = document.createElement('div');
                circle.style.cssText = 'width:56px;height:56px;min-width:56px;border-radius:50%;' +
                    'background:#fff;box-shadow:-2px 2px 12px rgba(0,0,0,0.15);' +
                    'display:flex;align-items:center;justify-content:center;overflow:hidden;' +
                    'transition:box-shadow 0.25s ease,transform 0.25s ease;';

                var img = document.createElement('img');
                img.src = '{icon_data_url}';
                img.style.cssText = 'width:40px;height:40px;border-radius:50%;pointer-events:none;';
                img.draggable = false;
                circle.appendChild(img);
                fab.appendChild(circle);

                // 文字标签（悬停时显示）
                var label = document.createElement('div');
                label.textContent = '\u6570\u636E\u5B89\u5168\u5C4B';
                label.style.cssText = 'margin-right:8px;padding:6px 14px;border-radius:20px;' +
                    'background:#fff;color:#1e40af;font-size:13px;font-weight:600;white-space:nowrap;' +
                    'box-shadow:0 2px 10px rgba(0,0,0,0.12);pointer-events:none;' +
                    'opacity:0;transform:translateX(8px);' +
                    'transition:opacity 0.2s ease,transform 0.2s ease;' +
                    'font-family:-apple-system,BlinkMacSystemFont,Segoe UI,Roboto,sans-serif;';
                fab.insertBefore(label, circle);

                // --- 吸附 / 滑出 ---
                var hideTimer = null;
                var isDragging = false;

                function snapToEdge() {{
                    if (window.__cheersai_nav_lock || isDragging) return;
                    fab.style.right = '0px';
                    fab.style.opacity = '0.85';
                    circle.style.boxShadow = '-2px 2px 12px rgba(0,0,0,0.15)';
                    circle.style.transform = 'scale(1)';
                    label.style.opacity = '0';
                    label.style.transform = 'translateX(8px)';
                }}
                function slideOut() {{
                    if (window.__cheersai_nav_lock || isDragging) return;
                    clearTimeout(hideTimer);
                    fab.style.right = '12px';
                    fab.style.opacity = '1';
                    circle.style.boxShadow = '0 6px 24px rgba(59,130,246,0.3)';
                    circle.style.transform = 'scale(1.08)';
                    label.style.opacity = '1';
                    label.style.transform = 'translateX(0)';
                }}
                function startHideTimer() {{
                    clearTimeout(hideTimer);
                    hideTimer = setTimeout(snapToEdge, 3000);
                }}

                fab.addEventListener('mouseenter', function() {{ slideOut(); }});
                fab.addEventListener('mouseleave', function() {{ startHideTimer(); }});

                // --- 上下拖动（吸附右侧边缘） ---
                var startY = 0, startTop = 0;

                fab.addEventListener('mousedown', function(e) {{
                    isDragging = false;
                    startY = e.clientY;
                    startTop = fab.getBoundingClientRect().top;
                    fab.style.transition = 'none'; // 拖动时关闭动画

                    function onMove(ev) {{
                        var dy = ev.clientY - startY;
                        if (Math.abs(dy) > 3) isDragging = true;
                        if (isDragging) {{
                            var newTop = Math.max(20, Math.min(window.innerHeight - 76, startTop + dy));
                            fab.style.top = newTop + 'px';
                        }}
                    }}
                    function onUp() {{
                        document.removeEventListener('mousemove', onMove);
                        document.removeEventListener('mouseup', onUp);
                        fab.style.transition = 'right 0.25s cubic-bezier(0.4,0,0.2,1),opacity 0.25s ease,box-shadow 0.25s ease';
                        if (isDragging) {{
                            startHideTimer();
                            setTimeout(function() {{ isDragging = false; }}, 50);
                        }}
                    }}
                    document.addEventListener('mousemove', onMove);
                    document.addEventListener('mouseup', onUp);
                }});

                // 初始化：先完整显示 5 秒，再吸附
                setTimeout(function() {{
                    fab.style.right = '12px';
                    fab.style.opacity = '1';
                    label.style.opacity = '1';
                    label.style.transform = 'translateX(0)';
                }}, 300);
                setTimeout(snapToEdge, 5000);

                // --- 点击：直接导航回本地页面 ---
                fab.addEventListener('click', function(e) {{
                    if (isDragging) return; // 拖动结束不触发点击
                    if (window.__cheersai_nav_lock) return;
                    window.__cheersai_nav_lock = true;

                    fab.style.transform = 'scale(0)';
                    fab.style.opacity = '0';
                    window.__TAURI_INTERNALS__.invoke('navigate_to_local', {{
                        returnUrl: '{return_url}'
                    }});
                }});

                document.body.appendChild(fab);
                console.log('✅ CheersAI FAB: Button injected successfully!');
                console.log('✅ CheersAI FAB: Button element:', fab);
                console.log('✅ CheersAI FAB: Button position:', fab.getBoundingClientRect());
                console.log('✅ CheersAI FAB: Button styles:', window.getComputedStyle(fab));
            }}
        }})();
    "#, icon_data_url = icon_data_url, return_url = return_url, cloud_host = CLOUD_HOST);

    // 注入任务：持续运行，只要还在外部页面就不断尝试注入
    // 这样退出登录再登录后，页面刷新也能重新注入按钮
    let window_for_inject = main_window.clone();
    let script_clone = inject_script.clone();
    tokio::spawn(async move {
        let mut count = 0u32;
        let mut found_target_domain = false;
        
        loop {
            // 前10次每秒检查一次，之后每2秒检查一次
            let delay = if count < 10 { 1000 } else { 2000 };
            tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
            count += 1;

            // 检查当前 URL
            let url_str = match window_for_inject.url() {
                Ok(u) => u.to_string(),
                Err(_) => continue,
            };
            
            if count <= 10 {
                println!("🔍 FAB inject check #{}: URL = {}", count, url_str);
            }
            
            // 检测是否在目标域名
            if url_str.contains(CLOUD_HOST) {
                found_target_domain = true;
                if count <= 10 {
                    println!("🎯 Target domain detected, injecting FAB...");
                }
                if window_for_inject.eval(&script_clone).is_ok() {
                    if count <= 5 {
                        println!("✓ FAB inject #{}", count);
                    }
                } else {
                    println!("⚠ FAB inject #{} failed", count);
                }
            } else if url_str.contains("localhost:1420") || url_str.contains("tauri://localhost") {
                // 只有在之前找到过目标域名后，才在返回本地页面时停止
                if found_target_domain {
                    println!("✓ FAB inject stopped: back on local page");
                    break;
                } else if count <= 10 {
                    println!("⏳ Waiting for navigation to target domain... (current: {})", url_str);
                }
            } else {
                if count <= 10 {
                    println!("⏳ Waiting for target domain... (current: {})", url_str);
                }
            }
            
            // 10 分钟超时
            if count > 300 {
                println!("⚠ FAB inject loop timeout");
                break;
            }
        }
    });

    Ok(())
}

// ===== Browser Fetch Infrastructure (uses WebView2/BoringSSL, bypasses reqwest TLS fingerprint) =====

pub struct BrowserFetchPending {
    pub channels: TokioMutex<HashMap<String, oneshot::Sender<BrowserFetchResult>>>,
}

impl Default for BrowserFetchPending {
    fn default() -> Self {
        Self {
            channels: TokioMutex::new(HashMap::new()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserFetchResult {
    pub ok: bool,
    pub status: u16,
    pub body: String,
}

#[tauri::command]
pub async fn on_browser_fetch_result(
    state: tauri::State<'_, BrowserFetchPending>,
    id: String,
    ok: bool,
    status: u16,
    body: String,
) -> Result<(), String> {
    let mut channels = state.channels.lock().await;
    if let Some(tx) = channels.remove(&id) {
        let _ = tx.send(BrowserFetchResult { ok, status, body });
    }
    Ok(())
}

/// 获取或创建隐藏的 FileBay API 代理窗口（同源 fetch，绕过 CORS）
async fn ensure_filebay_proxy_window(app: &AppHandle, base_url: &str) -> Result<WebviewWindow, String> {
    let label = "filebay-api-proxy";
    if let Some(win) = app.get_webview_window(label) {
        return Ok(win);
    }
    let parsed_url = base_url
        .parse::<tauri::Url>()
        .map_err(|e| format!("Invalid FileBay URL: {}", e))?;
    let win = WebviewWindowBuilder::new(app, label, WebviewUrl::External(parsed_url))
        .title("FileBay Proxy")
        .visible(false)
        .inner_size(1.0, 1.0)
        .skip_taskbar(true)
        .build()
        .map_err(|e| format!("Failed to create FileBay proxy window: {}", e))?;
    // 等待 WebView2 初始化
    tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;
    Ok(win)
}

/// 通过 WebView2 的 JavaScript fetch 发 HTTP 请求（使用浏览器的 BoringSSL TLS 栈）
/// proxy_base_url: 若提供，则在该域下的隐藏窗口中发请求（同源，避免 CORS）
pub async fn fetch_via_browser(
    app: &AppHandle,
    pending: &BrowserFetchPending,
    method: &str,
    url: &str,
    auth_token: &str,
    body: Option<&serde_json::Value>,
    proxy_base_url: Option<&str>,
) -> Result<BrowserFetchResult, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let (tx, rx) = oneshot::channel();

    {
        let mut channels = pending.channels.lock().await;
        channels.insert(id.clone(), tx);
    }

    let body_js = match body {
        Some(b) => {
            let body_str = b.to_string();
            let body_b64 = base64::engine::general_purpose::STANDARD.encode(body_str.as_bytes());
            format!(", body: atob('{}')" , body_b64)
        }
        None => String::new(),
    };

    let script = format!(
        r#"(async function() {{
    try {{
        const resp = await fetch('{}', {{
            method: '{}',
            headers: {{
                'Authorization': 'token {}',
                'Content-Type': 'application/json'
            }}{}
        }});
        const text = await resp.text();
        window.__TAURI_INTERNALS__.invoke('on_browser_fetch_result', {{
            id: '{}', ok: resp.ok, status: resp.status, body: text
        }});
    }} catch(err) {{
        window.__TAURI_INTERNALS__.invoke('on_browser_fetch_result', {{
            id: '{}', ok: false, status: 0, body: JSON.stringify({{error: err.message}})
        }});
    }}
}})();"#,
        url, method, auth_token, body_js, id, id
    );

    let window = if let Some(base_url) = proxy_base_url {
        ensure_filebay_proxy_window(app, base_url).await?
    } else {
        app.get_webview_window("main")
            .ok_or_else(|| "Main window not found".to_string())?
    };

    // 带重试的 eval（代理窗口刚创建时可能未就绪）
    let mut last_err = String::new();
    for attempt in 0..3u32 {
        match window.eval(&script) {
            Ok(_) => { last_err = String::new(); break; }
            Err(e) => {
                last_err = format!("Eval failed: {}", e);
                if attempt < 2 {
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                }
            }
        }
    }
    if !last_err.is_empty() {
        return Err(last_err);
    }

    tokio::time::timeout(tokio::time::Duration::from_secs(30), rx)
        .await
        .map_err(|_| "Browser fetch timeout after 30s".to_string())?
        .map_err(|_| "Channel was closed".to_string())
}
