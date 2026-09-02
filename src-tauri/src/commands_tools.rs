//! 通用工具运行(DESIGN-TOOLS.md §1):任意登记版本以任意参数启动。
//! 复用 DSH 进程表与退出监控:键 __tool__/<versionId>,日志仍走 process-log 事件,
//! 停止复用 stop_profile(SIGTERM)语义。

use crate::commands::peek_registry;
use crate::commands_processes::{spawn_exit_watcher, ProcessMap};
use crate::launcher_process as proc;
use crate::registry::{now_ms, RegResult};
use tauri::{AppHandle, State};

/// 启动一个通用工具:版本 bin + 用户参数 + 可选 cwd。
/// 返回运行键 "__tool__/<versionId>";单实例由 run/tool-<id>.lock 保证。
#[tauri::command]
pub fn start_tool(
    app: AppHandle,
    state: State<ProcessMap>,
    version_id: String,
    args: Option<Vec<String>>,
    cwd: Option<String>,
) -> RegResult<String> {
    let version = peek_registry()?
        .find_version(&version_id)
        .cloned()
        .ok_or_else(|| format!("版本 {version_id:?} 不存在"))?;
    let key = format!("{}/{}", proc::TOOL_HOME_ID, version_id);
    if state
        .0
        .lock()
        .map_err(|e| e.to_string())?
        .contains_key(&key)
    {
        return Err(format!("{key} 已在运行"));
    }
    let running = proc::start_tool(
        &version.bin,
        &version_id,
        args.as_deref().unwrap_or(&[]),
        cwd.as_deref(),
    )?;
    let child = running.child.clone();
    let (rx, _t_out, _t_err) = {
        let mut guard = child.lock().map_err(|e| e.to_string())?;
        proc::spawn_pipe_forwarders(&mut guard)
    };
    state
        .0
        .lock()
        .map_err(|e| e.to_string())?
        .insert(key.clone(), running);
    spawn_exit_watcher(
        app,
        child,
        rx,
        proc::TOOL_HOME_ID.into(),
        version_id.clone(),
        version_id,
        key.clone(),
        now_ms(),
    );
    Ok(key)
}
