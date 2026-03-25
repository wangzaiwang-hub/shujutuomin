use tauri::{AppHandle, Manager, Emitter, WebviewUrl, WebviewWindowBuilder};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use base64::Engine as _;

// 全局导航锁，防止重复触发
static NAVIGATION_IN_PROGRESS: AtomicBool = AtomicBool::new(false);

// 编译时嵌入 64x64 缩略图
const SAFER_ICON_BYTES: &[u8] = include_bytes!("../../safer_small.png");

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
    let return_url = format!("{}/process", return_origin);

    // 将图标转为 base64 data URL
    let icon_b64 = base64::engine::general_purpose::STANDARD.encode(SAFER_ICON_BYTES);
    let icon_data_url = format!("data:image/png;base64,{}", icon_b64);

    // 导航到目标外部 URL
    let parsed_url = url.parse().map_err(|e| format!("Invalid URL: {}", e))?;
    main_window.navigate(parsed_url)
        .map_err(|e| format!("Failed to navigate: {}", e))?;

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

                // 只在目标外部页面注入
                if (href.indexOf('tauri://localhost') !== -1 ||
                    href.indexOf('localhost:1420') !== -1 ||
                    href.indexOf('127.0.0.1') !== -1) return;
                if (href.indexOf('7smile.dlithink.com') === -1) return;
                if (href.indexOf('/login') !== -1 || href.indexOf('/signin') !== -1 ||
                    href.indexOf('/register') !== -1 || href.indexOf('/signup') !== -1 ||
                    href.indexOf('/auth') !== -1) return;
                if (document.querySelector('input[type="password"]') &&
                    document.querySelector('form')) return;
                if (document.getElementById('__cheersai_fab')) return;

                // --- 悬浮圆形按钮（吸附右侧、可拖动、悬停显示文字） ---
                var fab = document.createElement('div');
                fab.id = '__cheersai_fab';
                fab.style.cssText = 'position:fixed;top:40%;right:-24px;z-index:2147483647;' +
                    'display:flex;align-items:center;cursor:pointer;user-select:none;' +
                    'transition:right 0.25s cubic-bezier(0.4,0,0.2,1),opacity 0.25s ease,box-shadow 0.25s ease;' +
                    'opacity:0.7;';

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
                    fab.style.right = '-24px';
                    fab.style.opacity = '0.7';
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
                    hideTimer = setTimeout(snapToEdge, 2000);
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

                // 初始化：先完整显示 2 秒，再吸附
                setTimeout(function() {{
                    fab.style.right = '12px';
                    fab.style.opacity = '1';
                    label.style.opacity = '1';
                    label.style.transform = 'translateX(0)';
                }}, 300);
                setTimeout(snapToEdge, 2500);

                // --- 点击：直接导航回本地页面 ---
                fab.addEventListener('click', function(e) {{
                    if (isDragging) return; // 拖动结束不触发点击
                    if (window.__cheersai_nav_lock) return;
                    window.__cheersai_nav_lock = true;

                    fab.style.transform = 'scale(0)';
                    fab.style.opacity = '0';
                    window.location.replace('{return_url}');
                }});

                document.body.appendChild(fab);
                console.log('CheersAI: FAB injected');
            }}
        }})();
    "#, icon_data_url = icon_data_url, return_url = return_url);

    // 注入任务：持续运行，只要还在外部页面就不断尝试注入
    // 这样退出登录再登录后，页面刷新也能重新注入按钮
    let window_for_inject = main_window.clone();
    let script_clone = inject_script.clone();
    tokio::spawn(async move {
        let mut count = 0u32;
        loop {
            let delay = if count < 3 { 1500 } else { 3000 };
            tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
            count += 1;

            // 检查当前 URL，如果已离开外部页面则停止注入
            let url_str = match window_for_inject.url() {
                Ok(u) => u.to_string(),
                Err(_) => continue,
            };
            if url_str.contains("localhost:1420") || url_str.contains("tauri://localhost") {
                println!("✓ FAB inject stopped: back on local page");
                break;
            }
            // 只在目标域名上注入
            if url_str.contains("7smile.dlithink.com") {
                if window_for_inject.eval(&script_clone).is_ok() {
                    if count <= 3 {
                        println!("✓ FAB inject #{}", count);
                    }
                }
            }
            // 10 分钟超时
            if count > 200 {
                println!("⚠ FAB inject loop timeout");
                break;
            }
        }
    });

    // ===== 兜底任务：如果 JS 导航失败，Rust 强制导航 =====
    let window_clone = main_window.clone();
    let return_url_clone = return_url.clone();
    tokio::spawn(async move {
        let mut ticks = 0u32;

        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            ticks += 1;

            let url_str = match window_clone.url() {
                Ok(u) => u.to_string(),
                Err(_) => continue,
            };

            // 已经成功回到本地页面
            if url_str.contains("localhost:1420") || url_str.contains("tauri://localhost") {
                println!("✓ Back on local page: {}", url_str);
                break;
            }

            // 还在外部页面，继续等待
            if url_str.starts_with("https://7smile.dlithink.com")
                || url_str.starts_with("http://7smile.dlithink.com") {
                // 正常等待用户点击按钮
                if ticks > 600 { // 5 分钟超时
                    println!("⚠ Monitoring timeout");
                    break;
                }
                continue;
            }

            // URL 既不是外部也不是本地（可能 JS 导航卡在中间状态）
            // 等 2 秒看是否自动恢复
            println!("⚠ Unexpected URL: {}, retrying navigate...", url_str);
            tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;

            // 再检查一次
            let url_now = match window_clone.url() {
                Ok(u) => u.to_string(),
                Err(_) => continue,
            };

            if url_now.contains("localhost:1420") || url_now.contains("tauri://localhost") {
                println!("✓ Recovered, now on local: {}", url_now);
                break;
            }

            // 仍然卡住，Rust 强制导航
            println!("🔄 Force navigating to {}", return_url_clone);
            if let Ok(local_url) = return_url_clone.parse() {
                let _ = window_clone.navigate(local_url);
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

            if let Ok(final_url) = window_clone.url() {
                println!("📍 After force nav: {}", final_url);
            }
            break;
        }
    });

    Ok(())
}
