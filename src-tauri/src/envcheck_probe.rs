//! 系统快照采集与网络探测:全部 best-effort,失败返回"未知"而不报错。
//! 外部命令只用 curl / df / sw_vers / ldd,均为 macOS/Linux 自带。

use crate::envcheck::{EnvSnapshot, NetStatus};
use crate::registry;
use std::path::{Path, PathBuf};
use std::process::Command;

/// 采集本机快照;runtime 探测扫描 managed runtime 约定目录。
pub fn collect_snapshot() -> EnvSnapshot {
    let base_dir = registry::launcher_base_dir();
    let runtime_bin = find_runtime_bin(&base_dir);
    EnvSnapshot {
        os: os_name(),
        os_version: os_version(),
        arch: arch_name(),
        glibc: glibc_version(),
        disk_free_mb: disk_free_mb(&base_dir),
        base_dir_writable: probe_writable(&base_dir),
        node_version_ok: runtime_bin.as_ref().is_some_and(|b| node_version_ok(b)),
        runtime_bin,
        existing_dsh: detect_existing_dsh(),
        base_dir,
    }
}

/// 探测 npm 源与 Node 发行版源(官方/镜像任一可达即可)。
pub fn probe_net() -> NetStatus {
    NetStatus {
        registry_ok: probe_url("https://registry.npmjs.org")
            || probe_url("https://registry.npmmirror.com"),
        node_dist_ok: probe_url("https://nodejs.org/dist/index.json")
            || probe_url("https://cdn.npmmirror.com/binaries/node/index.json"),
    }
}

fn os_name() -> String {
    if cfg!(target_os = "macos") {
        "macos".into()
    } else if cfg!(target_os = "linux") {
        "linux".into()
    } else {
        "unknown".into()
    }
}

fn os_version() -> String {
    if cfg!(target_os = "macos") {
        run_stdout(Command::new("sw_vers").arg("-productVersion")).unwrap_or_default()
    } else {
        String::new()
    }
}

fn arch_name() -> String {
    match std::env::consts::ARCH {
        "aarch64" => "arm64".into(),
        other => other.into(),
    }
}

fn glibc_version() -> Option<String> {
    let line = run_stdout(Command::new("ldd").arg("--version"))?;
    let (_, after) = line.rsplit_once("ldd")?;
    let head = after.trim().split_whitespace().next()?;
    head.contains('.').then(|| head.to_string())
}

fn disk_free_mb(p: &Path) -> Option<u64> {
    let out = run_stdout(Command::new("df").arg("-k").arg(p))?;
    let last = out.lines().next_back()?;
    let avail_kb = last.split_whitespace().nth(3)?.parse::<u64>().ok()?;
    Some(avail_kb / 1024)
}

fn probe_url(url: &str) -> bool {
    Command::new("curl")
        .args(["-fsI", "--max-time", "2", url])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn probe_writable(dir: &Path) -> bool {
    if std::fs::create_dir_all(dir).is_err() {
        return false;
    }
    let probe = dir.join(".dsh-launcher-probe");
    let ok = std::fs::write(&probe, b"ok").is_ok();
    let _ = std::fs::remove_file(&probe);
    ok
}

/// managed runtime 约定:base/runtime/<node-v*>/bin,取版本号最大且含 node 的。
fn find_runtime_bin(base_dir: &Path) -> Option<PathBuf> {
    let root = base_dir.join("runtime");
    let mut entries: Vec<_> = std::fs::read_dir(&root).ok()?.flatten().collect();
    entries.sort_by_key(|e| e.file_name());
    entries.into_iter().rev().find_map(|e| {
        let bin = e.path().join("bin");
        bin.join("node").is_file().then_some(bin)
    })
}

fn node_version_ok(bin_dir: &Path) -> bool {
    Command::new(bin_dir.join("node"))
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// 检测已有 dsh:PATH 上的 dsh 或 ~/.dsh 目录,返回来源描述。
fn detect_existing_dsh() -> Option<String> {
    if let Ok(path) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path) {
            if dir.join("dsh").is_file() {
                return Some("PATH 中已安装 dsh".into());
            }
        }
    }
    let home_dsh = crate::launcher::home_dir().join(".dsh");
    home_dsh
        .is_dir()
        .then(|| format!("已有 home {}", home_dsh.display()))
}

fn run_stdout(cmd: &mut Command) -> Option<String> {
    let out = cmd.output().ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
}
