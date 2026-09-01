use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};
use super::launcher_runtime::{expand_tilde, home_dir, strip_dsh_env, tail_lines, INSTALL_TIMEOUT};
use crate::registry::{validate_id, versions_dir, RegResult};
pub fn resolve_npm() -> RegResult<PathBuf> {
    if let Ok(p) = std::env::var("DSH_LAUNCHER_NPM") {
        let p = expand_tilde(&p);
        if p.is_file() {
            return Ok(p);
        }
        return Err(format!(
            "DSH_LAUNCHER_NPM 指向的文件不存在: {}",
            p.display()
        ));
    }
    if let Some(p) = which("npm") {
        return Ok(p);
    }
    let candidates = [
        home_dir().join(".vite-plus/bin/npm"),
        PathBuf::from("/opt/homebrew/bin/npm"),
        PathBuf::from("/usr/local/bin/npm"),
    ];
    for c in &candidates {
        if c.is_file() {
            return Ok(c.clone());
        }
    }
    Err("找不到 npm:不在 PATH。可设 DSH_LAUNCHER_NPM 环境变量指定 npm 路径".into())
}
fn which(name: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        let candidate = dir.join(name);
        if is_executable(&candidate) {
            return Some(candidate);
        }
    }
    None
}
#[cfg(unix)]
fn is_executable(p: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    p.is_file()
        && p.metadata()
            .map(|m| m.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
}
pub fn install_npm(spec: &str, id: &str, mut on_line: impl FnMut(&str)) -> RegResult<PathBuf> {
    validate_id(id)?;
    let dir = versions_dir().join(id);
    if dir.exists() {
        on_line(&format!("清理旧目录 {}", dir.display()));
        fs::remove_dir_all(&dir).map_err(|e| format!("清理旧目录 {} 失败: {e}", dir.display()))?;
    }
    fs::create_dir_all(&dir).map_err(|e| format!("创建 {} 失败: {e}", dir.display()))?;
    let npm = resolve_npm()?;
    let mut cmd = Command::new(&npm);
    cmd.args(["install", "--prefix"]).arg(&dir).arg(spec);
    strip_dsh_env(&mut cmd);
    cmd.stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    on_line(&format!(
        "> {} install --prefix {} {}",
        npm.display(),
        dir.display(),
        spec
    ));
    let mut child = cmd
        .spawn()
        .map_err(|e| format!("无法启动 npm ({}): {e}", npm.display()))?;
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
    let mut stdout_buf = String::new();
    let mut stderr_buf = String::new();
    let deadline = Instant::now() + INSTALL_TIMEOUT;
    let mut last_keepalive = Instant::now();
    let mut status: Option<std::process::ExitStatus> = None;
    let mut timed_out = false;
    loop {
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok((is_err, line)) => {
                if is_err {
                    stderr_buf.push_str(&line);
                    stderr_buf.push('\n');
                    on_line(&format!("[stderr] {line}"));
                } else {
                    stdout_buf.push_str(&line);
                    stdout_buf.push('\n');
                    on_line(&line);
                }
                continue;
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
        if let Ok(Some(s)) = child.try_wait() {
            status = Some(s);
            while let Ok((is_err, line)) = rx.try_recv() {
                if is_err {
                    stderr_buf.push_str(&line);
                    stderr_buf.push('\n');
                    on_line(&format!("[stderr] {line}"));
                } else {
                    stdout_buf.push_str(&line);
                    stdout_buf.push('\n');
                    on_line(&line);
                }
            }
            break;
        }
        if last_keepalive.elapsed() >= Duration::from_secs(5) {
            last_keepalive = Instant::now();
            on_line("…仍在安装中");
        }
        if Instant::now() > deadline {
            let _ = child.kill();
            timed_out = true;
            break;
        }
    }
    let _ = t_out.join();
    let _ = t_err.join();
    if timed_out {
        let _ = child.wait();
    } else if status.is_none() {
        status = child.wait().ok();
    }
    let status = match status {
        Some(s) => s,
        None => {
            return Err(format!(
                "npm install 超时({}s),进程已终止\n{}",
                INSTALL_TIMEOUT.as_secs(),
                tail_lines(&stderr_buf, 15)
            ));
        }
    };
    if !status.success() {
        return Err(format!(
            "npm install 失败(退出码 {status})\n{}",
            tail_lines(&stderr_buf, 15)
        ));
    }
    let bin = dir.join("node_modules").join(".bin").join("dsh");
    if !bin.is_file() {
        return Err(format!(
            "npm install 结束但未找到 {}\nstderr: {}",
            bin.display(),
            tail_lines(&stderr_buf, 5)
        ));
    }
    on_line(&format!("安装完成: {}", bin.display()));
    Ok(bin)
}
pub fn remove_version_dir(id: &str) -> RegResult<()> {
    validate_id(id)?;
    let dir = versions_dir().join(id);
    if dir.exists() {
        fs::remove_dir_all(&dir).map_err(|e| format!("删除 {} 失败: {e}", dir.display()))?;
    }
    Ok(())
}
pub fn validate_dev_repo(repo_path: &str) -> RegResult<PathBuf> {
    let dir = expand_tilde(repo_path);
    if !dir.is_dir() {
        return Err(format!("repo 目录不存在: {}", dir.display()));
    }
    if !dir.join("package.json").is_file() {
        return Err(format!(
            "{} 下没有 package.json,不是 Node 项目根目录",
            dir.display()
        ));
    }
    Ok(dir)
}
