//! Tauri commands(M3):profile 发现 / 启动 / 停止 / 运行清单。
//! 进程表 ProcessMap 以 "homeId/profile" 为键;退出监控线程负责
//! 收尸、广播 process-exit、写 history 并释放运行锁。

use crate::commands::{peek_registry, with_registry};
use crate::launcher::expand_tilde;
use crate::launcher_process as proc;
use crate::registry::{now_ms, HistoryEntry, RegResult};
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, State};

#[derive(Default)]
pub struct ProcessMap(pub Mutex<HashMap<String, proc::RunningDsh>>);

fn process_key(home_id: &str, profile: &str) -> String {
    format!("{home_id}/{profile}")
}

/// 发现某 home 下的 profiles(读 <home>/profiles/*/package.json)。
#[tauri::command]
pub fn list_profiles(home_path: String) -> RegResult<Vec<proc::ProfileInfo>> {
    proc::list_profiles(&home_path)
}

/// 当前运行中的进程键("homeId/profile")。
#[tauri::command]
pub fn list_running(state: State<ProcessMap>) -> RegResult<Vec<String>> {
    let map = state.0.lock().map_err(|e| e.to_string())?;
    Ok(map.keys().cloned().collect())
}

/// 启动一个 profile:版本解析(显式指定 > home 绑定),加锁,起进程,开日志流。
#[tauri::command]
pub fn start_profile(
    app: AppHandle,
    state: State<ProcessMap>,
    home_id: String,
    profile: String,
    version_id: Option<String>,
    patch: Option<String>,
    args: Option<Vec<String>>,
    cwd: Option<String>,
) -> RegResult<()> {
    let reg = peek_registry()?;
    let home = reg
        .find_home(&home_id)
        .cloned()
        .ok_or_else(|| format!("home {home_id:?} 不存在"))?;
    let vid = version_id
        .or_else(|| home.bound_version_id.clone())
        .ok_or_else(|| format!("home {home_id:?} 未绑定版本,请先绑定或显式指定 versionId"))?;
    let version = reg
        .find_version(&vid)
        .cloned()
        .ok_or_else(|| format!("版本 {vid:?} 不存在"))?;
    let key = process_key(&home_id, &profile);
    if state
        .0
        .lock()
        .map_err(|e| e.to_string())?
        .contains_key(&key)
    {
        return Err(format!("{key} 已在运行"));
    }
    let running = proc::start_dsh(
        &version.bin,
        &home_id,
        &expand_tilde(&home.path),
        &profile,
        &vid,
        patch.as_deref(),
        args.as_deref().unwrap_or(&[]),
        cwd.as_deref(),
    )?;
    let child = running.child.clone();
    let (rx, t_out, t_err) = {
        let mut guard = child.lock().map_err(|e| e.to_string())?;
        proc::spawn_pipe_forwarders(&mut guard)
    };
    let _ = (t_out, t_err);
    state
        .0
        .lock()
        .map_err(|e| e.to_string())?
        .insert(key.clone(), running);
    spawn_exit_watcher(app, child, rx, home_id, profile, vid, key, now_ms());
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn spawn_exit_watcher(
    app: AppHandle,
    child: std::sync::Arc<std::sync::Mutex<std::process::Child>>,
    rx: std::sync::mpsc::Receiver<(bool, String)>,
    home_id: String,
    profile: String,
    version_id: String,
    key: String,
    started_at_ms: u64,
) {
    std::thread::spawn(move || {
        let mut log_rx = rx;
        let mut exit_code: Option<i32> = None;
        loop {
            match log_rx.recv_timeout(proc::poll_interval()) {
                Ok((is_err, line)) => {
                    let _ = app.emit(
                        "process-log",
                        serde_json::json!({ "homeId": home_id, "profile": profile, "line": line, "isErr": is_err }),
                    );
                    continue;
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
            }
            if let Some(status) = proc::try_wait(&child) {
                exit_code = status.code();
                break;
            }
        }
        let code = child
            .lock()
            .ok()
            .and_then(|mut c| c.wait().ok())
            .and_then(|s| s.code())
            .or(exit_code);
        let _ = app.emit(
            "process-exit",
            serde_json::json!({ "homeId": home_id, "profile": profile, "exitCode": code }),
        );
        if let Ok(map) = app.state::<ProcessMap>().0.lock() {
            let mut map = map;
            map.remove(&key);
        }
        let _ = with_registry(|reg| {
            reg.history.push(HistoryEntry {
                started_at: started_at_ms,
                home_id: home_id.clone(),
                profile: profile.clone(),
                version_id: version_id.clone(),
                exit_code: code,
            });
            let path = crate::registry::default_registry_path();
            crate::registry::save(&path, reg)
        });
    });
}

/// 停止:SIGTERM(dsh 约定 exit 0);收尾由监控线程完成。
#[tauri::command]
pub fn stop_profile(state: State<ProcessMap>, home_id: String, profile: String) -> RegResult<()> {
    let key = process_key(&home_id, &profile);
    let map = state.0.lock().map_err(|e| e.to_string())?;
    let entry = map.get(&key).ok_or_else(|| format!("未在运行: {key}"))?;
    proc::stop_dsh(&entry.child)
}
