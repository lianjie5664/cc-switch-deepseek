//! DeepSeek 本地代理服务管理
//!
//! 负责启动/停止 Node.js 代理子进程，
//! 实现 OpenAI Responses API ↔ DeepSeek Chat Completions API 协议转换。

use crate::settings;
use crate::store::AppState;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::Manager;

/// DeepSeek 代理状态
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeepSeekProxyStatus {
    pub running: bool,
    pub port: u16,
    pub pid: Option<u32>,
}

fn resolve_proxy_js_path(app_handle: Option<&tauri::AppHandle>) -> Option<PathBuf> {
    let check = |dir: &str, p: &std::path::Path| -> bool {
        let exists = p.exists();
        log::info!("DeepSeek proxy path check [{dir}]: {:?} -> {}", p, exists);
        exists
    };

    // 1. 环境变量覆盖
    if let Ok(path) = std::env::var("DEEPSEEK_PROXY_PATH") {
        let p = PathBuf::from(&path);
        let candidate = p.join("index.js");
        if check("env", &candidate) {
            return Some(candidate);
        }
        if check("env(raw)", &p) {
            return Some(p);
        }
    }

    // 2. exe 同级目录（最可靠：便携版/开发版都适用）
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            let candidate = exe_dir.join("ccswitch-deepseek-main").join("index.js");
            if check("exe_dir", &candidate) {
                return Some(candidate);
            }
        }
    }

    // 3. Tauri 资源目录（正式打包/MSI 安装版）
    if let Some(app) = app_handle {
        if let Ok(resource_dir) = app.path().resource_dir() {
            let candidate = resource_dir.join("resources").join("deepseek-proxy").join("index.js");
            if check("resource_dir", &candidate) {
                return Some(candidate);
            }
        }
    }

    // 4. 开发模式：cwd parent（源码目录布局）
    if let Ok(cwd) = std::env::current_dir() {
        let candidate = cwd
            .parent()
            .map(|p| p.join("ccswitch-deepseek-main").join("index.js"));
        if let Some(ref p) = candidate {
            if check("cwd_parent", p) {
                return candidate;
            }
        }
    }

    log::warn!("DeepSeek proxy index.js not found in any location");
    None
}

/// 获取 DeepSeek 代理状态
#[tauri::command]
pub async fn get_deepseek_proxy_status(
    state: tauri::State<'_, AppState>,
) -> Result<DeepSeekProxyStatus, String> {
    let config = settings::get_deepseek_proxy_config().unwrap_or_default();
    let guard = state.deepseek_proxy_process.lock().await;
    let running = guard.is_some();

    Ok(DeepSeekProxyStatus {
        running,
        port: config.port,
        pid: if running {
            guard.as_ref().and_then(|c| c.id())
        } else {
            None
        },
    })
}

/// 启动 DeepSeek 代理
#[tauri::command]
pub async fn start_deepseek_proxy(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<DeepSeekProxyStatus, String> {
    let config = settings::get_deepseek_proxy_config().unwrap_or_default();

    if config.api_key.trim().is_empty() {
        return Err("DeepSeek API Key 未配置，请先在设置中填写 API Key".to_string());
    }

    let js_path = resolve_proxy_js_path(Some(&app)).ok_or_else(|| {
        let exe = std::env::current_exe()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| "unknown".to_string());
        format!(
            "找不到 DeepSeek 代理脚本 (index.js)\n\
             exe 位置: {}\n\
             请确认 resources/deepseek-proxy 目录存在",
            exe
        )
    })?;

    let js_dir = js_path.parent().unwrap_or(&js_path);

    // 检查是否已在运行
    {
        let mut guard = state.deepseek_proxy_process.lock().await;
        if let Some(ref mut child) = *guard {
            match child.try_wait() {
                Ok(Some(_)) => {
                    // 进程已退出，清理
                    *guard = None;
                }
                Ok(None) => {
                    // 仍在运行
                    return Ok(DeepSeekProxyStatus {
                        running: true,
                        port: config.port,
                        pid: child.id(),
                    });
                }
                Err(e) => {
                    log::warn!("检查 DeepSeek 代理进程状态失败: {e}");
                    *guard = None;
                }
            }
        }
    }

    log::info!(
        "启动 DeepSeek 代理: node {:?} port={} model={}",
        js_path,
        config.port,
        config.model
    );

    let child = tokio::process::Command::new("node")
        .arg(&js_path)
        .current_dir(js_dir)
        .env("api_key", &config.api_key)
        .env("port", config.port.to_string())
        .env("model", &config.model)
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| format!("启动 DeepSeek 代理进程失败: {e}"))?;

    let pid = child.id();
    let mut guard = state.deepseek_proxy_process.lock().await;
    *guard = Some(child);

    log::info!("DeepSeek 代理已启动, pid={:?} port={}", pid, config.port);

    Ok(DeepSeekProxyStatus {
        running: true,
        port: config.port,
        pid,
    })
}

/// 停止 DeepSeek 代理
#[tauri::command]
pub async fn stop_deepseek_proxy(
    state: tauri::State<'_, AppState>,
) -> Result<DeepSeekProxyStatus, String> {
    let config = settings::get_deepseek_proxy_config().unwrap_or_default();

    let mut guard = state.deepseek_proxy_process.lock().await;
    if let Some(mut child) = guard.take() {
        log::info!("停止 DeepSeek 代理...");
        child
            .kill()
            .await
            .map_err(|e| format!("停止 DeepSeek 代理失败: {e}"))?;
        log::info!("DeepSeek 代理已停止");
    }

    Ok(DeepSeekProxyStatus {
        running: false,
        port: config.port,
        pid: None,
    })
}
