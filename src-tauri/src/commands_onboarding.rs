//! 首跑向导命令:查询/步进/完成,以及运行时重装修复动作。

use crate::commands::registry_io_lock;
use crate::commands_heavy::run_blocking;
use crate::onboarding;
use crate::registry::{OnboardingState, RegResult};
use crate::runtime_install::RuntimeInfo;
use tauri::Emitter;

/// 当前首跑状态(前端据此决定是否进向导)。
#[tauri::command]
pub async fn onboarding_get() -> RegResult<OnboardingState> {
    run_blocking(move || Ok(onboarding::load())).await
}

/// 步进到指定步骤(非法跨步被拒绝)。
#[tauri::command]
pub async fn onboarding_advance(step: String) -> RegResult<OnboardingState> {
    run_blocking(move || {
        let to = onboarding::parse_step(&step)?;
        onboarding::advance(to)
    })
    .await
}

/// 向导完成。
#[tauri::command]
pub async fn onboarding_complete() -> RegResult<OnboardingState> {
    run_blocking(move || onboarding::complete()).await
}

/// 修复动作:清掉旧 runtime 后全新安装(向导与修复中心共用)。
/// 进度行经 runtime-install-log 事件推送。
#[tauri::command]
pub async fn repair_runtime(app: tauri::AppHandle) -> RegResult<RuntimeInfo> {
    run_blocking(move || {
        let _guard = registry_io_lock().lock().map_err(|e| e.to_string())?;
        let emitter = app;
        crate::runtime_install::reinstall_runtime(move |line| {
            let _ = emitter.emit("runtime-install-log", serde_json::json!({ "line": line }));
        })
    })
    .await
}
