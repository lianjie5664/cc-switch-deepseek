//! DeepSeek 本地代理服务管理
//!
//! 负责启动/停止 Node.js 代理子进程，
//! 实现 OpenAI Responses API ↔ DeepSeek Chat Completions API 协议转换。

use crate::settings;
use crate::store::AppState;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// DeepSeek 代理状态
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeepSeekProxyStatus {
    pub running: bool,
    pub port: u16,
    pub pid: Option<u32>,
}

fn resolve_proxy_js_path() -> Option<PathBuf> {
    // 1. 环境变量覆盖（开发模式）
    if let Ok(path) = std::env::var("DEEPSEEK_PROXY_PATH") {
        let p = PathBuf::from(&path);
        if p.join("index.js").exists() {
            return Some(p.join("index.js"));
        }
        if p.exists() {
            return Some(p);
        }
    }

    // 2. 开发模式：相对路径查找（源码目录下的 cswitch-deepseek-main）
    if let Ok(cwd) = std::env::current_dir() {
        // cswitch-deepseek-main 与 cc-switch-main 同级
        let candidate = cwd
            .parent()
            .map(|p| p.join("ccswitch-deepseek-main").join("index.js"));
        if let Some(ref p) = candidate {
            if p.exists() {
                return candidate;
            }
        }
    }

    // 3. 开发模式：在 cc-switch-main 目录内查找
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            let candidate = exe_dir.join("ccswitch-deepseek-main").join("index.js");
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }

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
    state: tauri::State<'_, AppState>,
) -> Result<DeepSeekProxyStatus, String> {
    let config = settings::get_deepseek_proxy_config().unwrap_or_default();

    if config.api_key.trim().is_empty() {
        return Err("DeepSeek API Key 未配置，请先在设置中填写 API Key".to_string());
    }

    let js_path = resolve_proxy_js_path().ok_or_else(|| {
        "找不到 DeepSeek 代理脚本 (index.js)，请确认 cswitch-deepseek-main 目录存在".to_string()
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
