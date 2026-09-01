//! 进程级操作。设计依据 DESIGN.md §1/§3:
//! - 启动任何 dsh 前,先清掉环境里游离的 DSH_*(launcher 独占 DSH_HOME 来源);
//! - bin 含空白(如 dev 的 "pnpm dsh")时经 sh -c 执行,否则直接 exec。

use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use crate::registry::RegResult;

const FINGERPRINT_TIMEOUT: Duration = Duration::from_secs(120);

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

fn strip_dsh_env(cmd: &mut Command) {
    for (k, _) in std::env::vars() {
        if k.starts_with("DSH_") {
            cmd.env_remove(k);
        }
    }
}

/// 运行 <bin> --version,stdout 首个非空行 trim 后作为指纹。
/// Err 情形:bin/cwd 不存在、启动失败、超时、非零退出、无 stdout 输出。
pub fn fingerprint(bin: &str, cwd: Option<&str>) -> RegResult<String> {
    let mut cmd = if bin.split_whitespace().count() > 1 {
        let mut c = Command::new("sh");
        c.arg("-c").arg(format!("{bin} --version"));
        c
    } else {
        let path = expand_tilde(bin);
        // 显式路径必须存在;裸命令名(如 "dsh")交由 PATH 解析
        if bin.contains('/') && !path.is_file() {
            return Err(format!("bin 不存在: {}", path.display()));
        }
        let target = if bin.contains('/') { path } else { PathBuf::from(bin) };
        let mut c = Command::new(target);
        c.arg("--version");
        c
    };
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

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("无法启动 {bin:?}: {e}"))?;
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
    let deadline = Instant::now() + FINGERPRINT_TIMEOUT;
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
        if let Ok(Some(_)) = child.try_wait() {
            // 进程已退出:排空管道残留后收尾
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
            let _ = child.wait();
            return Err(format!(
                "采集指纹超时({}s),进程已终止",
                FINGERPRINT_TIMEOUT.as_secs()
            ));
        }
    }
    let _ = t_out.join();
    let _ = t_err.join();
    let status = child.wait().map_err(|e| format!("等待进程失败: {e}"))?;
    if !status.success() {
        let tail: String = stderr_buf
            .lines()
            .rev()
            .take(5)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .join(" | ");
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expand_tilde_handles_home_forms() {
        let home = home_dir();
        assert_eq!(expand_tilde("~"), home);
        assert_eq!(expand_tilde("~/foo/bar"), home.join("foo/bar"));
        assert_eq!(expand_tilde("/abs/path"), PathBuf::from("/abs/path"));
        assert_eq!(expand_tilde("  ~/x  "), home.join("x"));
    }

    #[test]
    fn fingerprint_direct_exec() {
        // echo 直接 exec:输出恰为参数本身
        assert_eq!(fingerprint("echo", None).unwrap(), "--version");
    }

    #[test]
    fn fingerprint_shell_for_spaced_bin() {
        // 含空白的 bin 走 sh -c
        assert_eq!(fingerprint("printf hello-from-shell", None).unwrap(), "hello-from-shell");
    }

    #[test]
    fn fingerprint_missing_bin_is_error() {
        assert!(fingerprint("/nonexistent/dsh-binary-xyz", None).is_err());
    }

    #[test]
    fn fingerprint_nonzero_exit_is_error() {
        assert!(fingerprint("sh -c 'echo boom 1>&2; exit 3'", None).is_err());
    }

    #[test]
    fn fingerprint_empty_stdout_is_error() {
        assert!(fingerprint("sh -c 'exit 0'", None).is_err());
    }
}
