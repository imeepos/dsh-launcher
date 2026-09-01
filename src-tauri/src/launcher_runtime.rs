use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use crate::registry::RegResult;

const FINGERPRINT_TIMEOUT: Duration = Duration::from_secs(120);
pub(crate) const INSTALL_TIMEOUT: Duration = Duration::from_secs(600);

/// dev 版本的启动命令(DESIGN.md:登记 repo checkout,启动命令 pnpm dsh)
pub const DEV_BIN: &str = "pnpm dsh";

pub fn home_dir() -> PathBuf {
    PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| ".".into()))
}

/// 支持 "~" 与 "~/..." 展开;其余原样。
pub fn expand_tilde(p: &str) -> PathBuf {
    let t = p.trim();
    if t == "~" {
        return home_dir();
    }
    if let Some(rest) = t.strip_prefix("~/") {
        return home_dir().join(rest);
    }
    PathBuf::from(t)
}

pub(crate) fn strip_dsh_env(cmd: &mut Command) {
    for (k, _) in std::env::vars() {
        if k.starts_with("DSH_") {
            cmd.env_remove(k);
        }
    }
}

fn build_version_command(bin: &str) -> RegResult<Command> {
    Ok(if bin.split_whitespace().count() > 1 {
        let mut c = Command::new("sh");
        c.arg("-c").arg(format!("{bin} --version"));
        c
    } else {
        let path = expand_tilde(bin);
        // 显式路径必须存在;裸命令名(如 "dsh")交由 PATH 解析
        if bin.contains('/') && !path.is_file() {
            return Err(format!("bin 不存在: {}", path.display()));
        }
        let target = if bin.contains('/') {
            path
        } else {
            PathBuf::from(bin)
        };
        let mut c = Command::new(target);
        c.arg("--version");
        c
    })
}
/// 运行 <bin> --version,stdout 首个非空行 trim 后作为指纹。
/// Err 情形:bin/cwd 不存在、启动失败、超时、非零退出、无 stdout 输出。
pub fn fingerprint(bin: &str, cwd: Option<&str>) -> RegResult<String> {
    let mut cmd = build_version_command(bin)?;
    if let Some(cwd) = cwd {
        let dir = expand_tilde(cwd);
        if !dir.is_dir() {
            return Err(format!("cwd 不存在: {}", dir.display()));
        }
        cmd.current_dir(dir);
    }
    strip_dsh_env(&mut cmd);
    cmd.stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = cmd.spawn().map_err(|e| format!("无法启动 {bin:?}: {e}"))?;
    let (stdout_buf, stderr_buf, status) = wait_with_output_lines(&mut child, FINGERPRINT_TIMEOUT)?;
    let status = status.ok_or_else(|| "采集指纹超时,进程已终止".to_string())?;
    if !status.success() {
        let tail = tail_lines(&stderr_buf, 5);
        return Err(format!("{bin} --version 退出码 {status};stderr: {tail}"));
    }
    let fp = stdout_buf
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .unwrap_or("")
        .to_string();
    if fp.is_empty() {
        return Err(format!(
            "{bin} --version 无输出;stderr: {}",
            stderr_buf.trim()
        ));
    }
    Ok(fp)
}
/// 流式执行子进程:stdout/stderr 逐行进缓冲;返回 (stdout, stderr, Option<status>)。
/// status 为 None 表示超时被 kill。
pub(crate) fn wait_with_output_lines(
    child: &mut std::process::Child,
    timeout: Duration,
) -> RegResult<(String, String, Option<std::process::ExitStatus>)> {
    let out_pipe = child.stdout.take().expect("stdout 已 pipe");
    let err_pipe = child.stderr.take().expect("stderr 已 pipe");
    let (tx, rx) = mpsc::channel::<(bool, String)>(); // (is_stderr, line)
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
    let deadline = Instant::now() + timeout;
    let mut status: Option<std::process::ExitStatus> = None;
    let mut timed_out = false;
    loop {
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok((false, line)) => {
                stdout_buf.push_str(&line);
                stdout_buf.push('\n');
                continue;
            }
            Ok((true, line)) => {
                stderr_buf.push_str(&line);
                stderr_buf.push('\n');
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
                } else {
                    stdout_buf.push_str(&line);
                    stdout_buf.push('\n');
                }
            }
            break;
        }
        if Instant::now() > deadline {
            let _ = child.kill();
            timed_out = true;
            break;
        }
    }
    let _ = t_out.join();
    let _ = t_err.join();
    // 正常退出路径:管道关闭 → Disconnected → break,status 还没采集,这里补 wait。
    let status = if timed_out {
        let _ = child.wait();
        None
    } else {
        match status {
            Some(s) => Some(s),
            None => child.wait().ok(),
        }
    };
    Ok((stdout_buf, stderr_buf, status))
}

pub(crate) fn tail_lines(buf: &str, n: usize) -> String {
    buf.lines()
        .rev()
        .take(n)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>()
        .join(" | ")
}
