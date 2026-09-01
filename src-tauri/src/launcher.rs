//! 进程级操作。设计依据 DESIGN.md §1/§3:
//! - 启动任何 dsh 前,先清掉环境里游离的 DSH_*(launcher 独占 DSH_HOME 来源);
//! - bin 含空白(如 dev 的 "pnpm dsh")时经 sh -c 执行,否则直接 exec;
//! - npm 安装:每版本独立目录 versions/<id>。

use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use crate::registry::{validate_id, versions_dir, RegResult};

const FINGERPRINT_TIMEOUT: Duration = Duration::from_secs(120);
const INSTALL_TIMEOUT: Duration = Duration::from_secs(600);

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

fn strip_dsh_env(cmd: &mut Command) {
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
        let target = if bin.contains('/') { path } else { PathBuf::from(bin) };
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

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("无法启动 {bin:?}: {e}"))?;
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
fn wait_with_output_lines(
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

fn tail_lines(buf: &str, n: usize) -> String {
    buf.lines()
        .rev()
        .take(n)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>()
        .join(" | ")
}
/// 在 PATH 与常见位置解析 npm。可用 DSH_LAUNCHER_NPM 显式覆盖。
pub fn resolve_npm() -> RegResult<PathBuf> {
    if let Ok(p) = std::env::var("DSH_LAUNCHER_NPM") {
        let p = expand_tilde(&p);
        if p.is_file() {
            return Ok(p);
        }
        return Err(format!("DSH_LAUNCHER_NPM 指向的文件不存在: {}", p.display()));
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
/// npm 安装一个版本(DESIGN.md:npm install --prefix versions/<id>,每版本独立依赖树)。
/// 进度逐行回调;成功返回 bin 路径 versions/<id>/node_modules/.bin/dsh。
/// 重试语义:调用前自动清掉已存在的 versions/<id> 目录,从头安装。
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
        // 正常退出路径:管道关闭 → Disconnected → break,status 还没采集,这里补 wait。
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
/// 删除 npm 版本的独立依赖树目录 versions/<id>(DESIGN.md:删除 npm 版本连目录一起删)。
pub fn remove_version_dir(id: &str) -> RegResult<()> {
    validate_id(id)?;
    let dir = versions_dir().join(id);
    if dir.exists() {
        fs::remove_dir_all(&dir).map_err(|e| format!("删除 {} 失败: {e}", dir.display()))?;
    }
    Ok(())
}

/// 校验 dev repo:目录存在且根上有 package.json。
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::sanitize_id_fragment;

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
        // 含空白的 bin 走 sh -c(dev kind 的 "pnpm dsh" 同理)
        assert_eq!(
            fingerprint("printf hello-from-shell", None).unwrap(),
            "hello-from-shell"
        );
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

    #[test]
    fn install_npm_rejects_bad_id_fast() {
        // 非法 id 直接报错,不会真的去跑 npm
        let mut called = false;
        let r = install_npm("@deepseek-ai/dsh@0.1.1-rc.2", "../evil", |_| {
            called = true;
        });
        assert!(r.is_err());
        assert!(!called);
    }

    #[test]
    fn remove_version_dir_rejects_traversal() {
        assert!(remove_version_dir("../../etc").is_err());
        assert!(remove_version_dir("").is_err());
    }

    #[test]
    fn validate_dev_repo_checks_layout() {
        assert!(validate_dev_repo("/nonexistent-repo-xyz").is_err());
        // /tmp 存在但没有 package.json
        assert!(validate_dev_repo("/tmp").is_err());
    }

    #[test]
    fn dev_bin_is_pnpm_dsh() {
        assert_eq!(DEV_BIN, "pnpm dsh");
        assert_eq!(sanitize_id_fragment("deepseek-harness"), "deepseek-harness");
    }
}
