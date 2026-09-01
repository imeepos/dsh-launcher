//! 子进程运行时:管道读取、流式输出收集与超时控制。

use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::Command;
use std::sync::mpsc::{self, Receiver};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

pub(crate) const FINGERPRINT_TIMEOUT: Duration = Duration::from_secs(120);
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

/// 子进程以自身为进程组长启动,超时可整组击杀(否则 sh 的孙进程会存活并占住管道)。
pub(crate) fn make_group_leader(cmd: &mut Command) {
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        cmd.process_group(0);
    }
    #[cfg(not(unix))]
    {
        let _ = cmd;
    }
}

/// 杀掉子进程所在的整个进程组(需配合 make_group_leader),失败时退回单杀。
pub(crate) fn kill_process_group(child: &mut std::process::Child) {
    #[cfg(unix)]
    {
        let pid = child.id() as i32;
        let killed = unsafe { libc::kill(-pid, libc::SIGKILL) };
        if killed != 0 {
            let _ = child.kill();
        }
    }
    #[cfg(not(unix))]
    {
        let _ = child.kill();
    }
}

type LineRx = Receiver<(bool, String)>;

fn spawn_pipe_readers(child: &mut std::process::Child) -> (LineRx, JoinHandle<()>, JoinHandle<()>) {
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

fn push_line(
    stdout_buf: &mut String,
    stderr_buf: &mut String,
    is_err: bool,
    line: &str,
    on_line: &mut dyn FnMut(bool, &str),
) {
    if is_err {
        stderr_buf.push_str(line);
        stderr_buf.push('\n');
    } else {
        stdout_buf.push_str(line);
        stdout_buf.push('\n');
    }
    on_line(is_err, line);
}

type Collected = (String, String, Option<std::process::ExitStatus>, bool);

fn collect_until_exit(
    rx: &LineRx,
    child: &mut std::process::Child,
    deadline: Instant,
    mut on_line: impl FnMut(bool, &str),
    keepalive: Option<(Duration, &str)>,
) -> Collected {
    let mut stdout_buf = String::new();
    let mut stderr_buf = String::new();
    let mut status: Option<std::process::ExitStatus> = None;
    let mut timed_out = false;
    let mut last_keepalive = Instant::now();
    loop {
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok((is_err, line)) => {
                push_line(
                    &mut stdout_buf,
                    &mut stderr_buf,
                    is_err,
                    &line,
                    &mut on_line,
                );
                continue;
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
        if let Ok(Some(s)) = child.try_wait() {
            status = Some(s);
            while let Ok((is_err, line)) = rx.try_recv() {
                push_line(
                    &mut stdout_buf,
                    &mut stderr_buf,
                    is_err,
                    &line,
                    &mut on_line,
                );
            }
            break;
        }
        if let Some((interval, msg)) = keepalive {
            if last_keepalive.elapsed() >= interval {
                last_keepalive = Instant::now();
                on_line(false, msg);
            }
        }
        if Instant::now() > deadline {
            kill_process_group(child);
            timed_out = true;
            break;
        }
    }
    (stdout_buf, stderr_buf, status, timed_out)
}

/// 流式执行子进程:stdout/stderr 逐行喂给 on_line(is_err, line)。
/// 返回 (stdout, stderr, Option<status>);status 为 None 表示超时被 kill。
/// 正常退出路径:管道关闭 → Disconnected → break,补 wait 采集退出码。
pub(crate) fn wait_with_output_lines(
    child: &mut std::process::Child,
    timeout: Duration,
    on_line: impl FnMut(bool, &str),
    keepalive: Option<(Duration, &str)>,
) -> crate::registry::RegResult<(String, String, Option<std::process::ExitStatus>)> {
    let (rx, t_out, t_err) = spawn_pipe_readers(child);
    let deadline = Instant::now() + timeout;
    let (stdout_buf, stderr_buf, status, timed_out) =
        collect_until_exit(&rx, child, deadline, on_line, keepalive);
    let _ = t_out.join();
    let _ = t_err.join();
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
