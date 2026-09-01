//! 环境检查命令:给前端首跑向导(env_check)与修复中心(env_check_fast)用。
//! 探测含网络请求,统一走 spawn_blocking 不卡主线程。

use crate::commands_heavy::run_blocking;
use crate::envcheck::{self, CheckItem, EnvSnapshot};
use crate::envcheck_probe;
use crate::registry::RegResult;

/// 首跑全量检查(含网络探测,数秒级)。
#[tauri::command]
pub async fn env_check() -> RegResult<Vec<CheckItem>> {
    run_blocking(move || {
        let snap = envcheck_probe::collect_snapshot();
        let net = envcheck_probe::probe_net();
        Ok(envcheck::run_checks(&snap, &net))
    })
    .await
}

/// 日常快速预检(纯本地 O(1) 项)。
#[tauri::command]
pub async fn env_check_fast() -> RegResult<Vec<CheckItem>> {
    run_blocking(move || {
        let snap: EnvSnapshot = envcheck_probe::collect_snapshot();
        Ok(envcheck::run_fast_preflight(&snap))
    })
    .await
}

/// 只采集快照不探网络,供向导分步展示。
#[tauri::command]
pub async fn env_snapshot() -> RegResult<EnvSnapshot> {
    run_blocking(move || Ok(envcheck_probe::collect_snapshot())).await
}
