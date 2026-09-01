//! dsh 进程运行时(M3):profile 发现、启动、运行锁、SIGTERM 停止。
//! 设计依据 DESIGN.md §1/§3/§6:清游离 DSH_* 后显式设 DSH_HOME;
//! 命令形如 <bin> --profile <p> [--patch f] -- <args>;(home, profile) 单实例文件锁。

use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::thread::JoinHandle;
use std::time::Duration;

use fs4::FileExt;

use crate::launcher::launcher_env::prepare_cmd_env;
use crate::launcher::{expand_tilde, make_group_leader};
use crate::registry::RegResult;

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileInfo {
    pub name: String,
    pub bundle_count: usize,
}

/// 扫描 <home>/profiles/*/package.json 发现 profile;无 profiles 目录视为空。
pub fn list_profiles(home_path: &str) -> RegResult<Vec<ProfileInfo>> {
    let profiles_dir = expand_tilde(home_path).join("profiles");
    if !profiles_dir.is_dir() {
        return Ok(vec![]);
    }
    let mut out = Vec::new();
    let rd = fs::read_dir(&profiles_dir)
        .map_err(|e| format!("读取 {} 失败: {e}", profiles_dir.display()))?;
    for entry in rd {
        let entry = entry.map_err(|e| e.to_string())?;
        if !entry.file_type().map_err(|e| e.to_string())?.is_dir() {
            continue;
        }
        let pkg = entry.path().join("package.json");
        if !pkg.is_file() {
            continue;
        }
        out.push(ProfileInfo {
            name: entry.file_name().to_string_lossy().into_owned(),
            bundle_count: read_bundle_count(&pkg),
        });
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

fn read_bundle_count(pkg: &Path) -> usize {
    let value: Option<serde_json::Value> = fs::read_to_string(pkg)
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok());
    value
        .as_ref()
        .and_then(|v| v.pointer("/dsh/profile/bundles"))
        .and_then(|b| b.as_array())
        .map(|a| a.len())
        .unwrap_or(0)
}

/// 运行中的 dsh:锁句柄存活即持锁;SIGTERM 由 stop_dsh 发送。
pub struct RunningDsh {
    pub child: std::sync::Arc<std::sync::Mutex<Child>>,
    pub home_id: String,
    pub profile: String,
    pub version_id: String,
    pub started_at_ms: u64,
    _lock: File,
}

fn lock_path(home_path: &Path, profile: &str) -> PathBuf {
    home_path
        .join("profiles")
        .join(profile)
        .join(".dsh-launcher.lock")
}

/// (home, profile) 单实例锁:fs4 建议锁,进程死亡自动释放。
pub fn acquire_run_lock(home_path: &Path, profile: &str) -> RegResult<File> {
    let path = lock_path(home_path, profile);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("创建 {} 失败: {e}", parent.display()))?;
    }
    let file = File::create(&path).map_err(|e| format!("打开 {} 失败: {e}", path.display()))?;
    if file.try_lock().is_err() {
        return Err(format!("该 profile 已在运行(锁被占用): {}", path.display()));
    }
    Ok(file)
}

/// 构造启动命令(DESIGN.md §3)。<bin> --profile <p> [--patch f] -- <extra>。
pub fn build_start_command(
    bin: &str,
    home_path: &Path,
    profile: &str,
    patch: Option<&str>,
    extra: &[String],
    cwd: Option<&str>,
) -> RegResult<Command> {
    let tokens: Vec<&str> = bin.split_whitespace().collect();
    if tokens.is_empty() {
        return Err("bin 不能为空".into());
    }
    let mut args: Vec<String> = tokens[1..].iter().map(|s| s.to_string()).collect();
    args.push("--profile".into());
    args.push(profile.to_string());
    if let Some(p) = patch.map(str::trim).filter(|s| !s.is_empty()) {
        args.push("--patch".into());
        args.push(p.to_string());
    }
    if !extra.is_empty() {
        args.push("--".into());
        args.extend(extra.iter().cloned());
    }
    let mut c = Command::new(expand_tilde(tokens[0]));
    c.args(&args);
    prepare_cmd_env(&mut c);
    c.env("DSH_HOME", home_path);
    if let Some(cwd) = cwd.map(expand_tilde) {
        c.current_dir(cwd);
    }
    make_group_leader(&mut c);
    c.stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    Ok(c)
}

/// 启动 dsh 并持有运行锁。锁失败返回 Err(已在运行),不产生进程。
pub fn start_dsh(
    bin: &str,
    home_id: &str,
    home_path: &Path,
    profile: &str,
    version_id: &str,
    patch: Option<&str>,
    extra: &[String],
    cwd: Option<&str>,
) -> RegResult<RunningDsh> {
    let lock = acquire_run_lock(home_path, profile)?;
    let mut cmd = build_start_command(bin, home_path, profile, patch, extra, cwd)?;
    let child = cmd.spawn().map_err(|e| format!("启动 {bin:?} 失败: {e}"))?;
    Ok(RunningDsh {
        child: std::sync::Arc::new(std::sync::Mutex::new(child)),
        home_id: home_id.into(),
        profile: profile.into(),
        version_id: version_id.into(),
        started_at_ms: crate::registry::now_ms(),
        _lock: lock,
    })
}

/// 优雅停止:SIGTERM(dsh 约定 exit 0);收尸由监控线程负责。
pub fn stop_dsh(child: &std::sync::Arc<std::sync::Mutex<Child>>) -> RegResult<()> {
    let pid = child.lock().map_err(|e| e.to_string())?.id() as i32;
    let sent = unsafe { libc::kill(pid, libc::SIGTERM) };
    if sent != 0 {
        return Err(format!(
            "SIGTERM 发送失败(pid={pid}): {}",
            std::io::Error::last_os_error()
        ));
    }
    Ok(())
}

/// 子进程 stdout/stderr 转发到通道((is_stderr, line)),供事件流使用。
pub(crate) fn spawn_pipe_forwarders(
    child: &mut Child,
) -> (Receiver<(bool, String)>, JoinHandle<()>, JoinHandle<()>) {
    use std::io::{BufRead, BufReader};
    let out_pipe = child.stdout.take().expect("stdout 已 pipe");
    let err_pipe = child.stderr.take().expect("stderr 已 pipe");
    let (tx, rx) = mpsc::channel::<(bool, String)>();
    let t_out = {
        let tx = tx.clone();
        std::thread::spawn(move || {
            for line in BufReader::new(out_pipe).lines().map_while(Result::ok) {
                let _ = tx.send((false, line));
            }
        })
    };
    let t_err = std::thread::spawn(move || {
        for line in BufReader::new(err_pipe).lines().map_while(Result::ok) {
            let _ = tx.send((true, line));
        }
    });
    (rx, t_out, t_err)
}

/// 轮询等待退出(监控线程用):Some(status) 即已退出。
pub fn try_wait(
    child: &std::sync::Arc<std::sync::Mutex<Child>>,
) -> Option<std::process::ExitStatus> {
    child
        .lock()
        .ok()
        .and_then(|mut c| c.try_wait().ok())
        .flatten()
}

/// 监控线程轮询间隔。
pub fn poll_interval() -> Duration {
    Duration::from_millis(200)
}
