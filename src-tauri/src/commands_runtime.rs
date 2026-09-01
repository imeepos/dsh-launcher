//! 运行时命令:安装(install_runtime)与查询(runtime_info)。
//! 安装走注册表串行锁,与 npm 版本安装互斥;进度经事件推送。

use crate::commands::registry_io_lock;
use crate::commands_heavy::run_blocking;
use crate::registry::RegResult;
use crate::runtime_install::{self, RuntimeInfo};
use tauri::Emitter;

/// 安装 managed Node 运行时(幂等,已装则重装覆盖)。
/// 进度行经 runtime-install-log 事件推送给前端。
#[tauri::command]
pub async fn install_runtime(app: tauri::AppHandle) -> RegResult<RuntimeInfo> {
    run_blocking(move || {
        let _guard = registry_io_lock().lock().map_err(|e| e.to_string())?;
        let emitter = app;
        runtime_install::install_runtime(move |line| {
            let _ = emitter.emit("runtime-install-log", serde_json::json!({ "line": line }));
        })
    })
    .await
}

/// 当前运行时信息;未安装返回 None。
#[tauri::command]
pub async fn runtime_info() -> RegResult<Option<RuntimeInfo>> {
    run_blocking(move || Ok(runtime_install::read_runtime())).await
}

/// 查 npm 上 @deepseek-ai/dsh 的最新版本号(向导快速模式用)。
#[tauri::command]
pub async fn npm_latest_version() -> RegResult<String> {
    run_blocking(move || {
        let npm = crate::launcher::launcher_env::resolve_npm()?;
        let out = std::process::Command::new(npm)
            .args(["view", "@deepseek-ai/dsh", "version"])
            .output()
            .map_err(|e| format!("启动 npm 失败:{e}"))?;
        let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if !out.status.success() || text.is_empty() {
            return Err("查询最新版本失败".into());
        }
        Ok(text)
    })
    .await
}
