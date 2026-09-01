//! 指纹实测:spawn <bin> --version,stdout 首个非空行作为指纹。
//! Err 情形:bin/cwd 不存在、启动失败、超时、非零退出、无 stdout 输出。

use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;

use super::launcher_runtime::{
    expand_tilde, make_group_leader, strip_dsh_env, wait_with_output_lines, FINGERPRINT_TIMEOUT,
};
use crate::registry::RegResult;

pub(crate) fn build_version_command(bin: &str) -> RegResult<Command> {
    Ok(if bin.split_whitespace().count() > 1 {
        let mut c = Command::new("sh");
        c.arg("-c").arg(format!("{bin} --version"));
        make_group_leader(&mut c);
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
        make_group_leader(&mut c);
        c
    })
}

/// 运行 <bin> --version,stdout 首个非空行 trim 后作为指纹。
pub fn fingerprint(bin: &str, cwd: Option<&str>) -> RegResult<String> {
    fingerprint_with_timeout(bin, cwd, FINGERPRINT_TIMEOUT)
}

/// 同 fingerprint,超时可注入(测试与诊断用)。
pub fn fingerprint_with_timeout(
    bin: &str,
    cwd: Option<&str>,
    timeout: Duration,
) -> RegResult<String> {
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
    let (stdout_buf, stderr_buf, status) =
        wait_with_output_lines(&mut child, timeout, |_, _| {}, None)?;
    let status = status.ok_or_else(|| "采集指纹超时,进程已终止".to_string())?;
    if !status.success() {
        let tail = super::launcher_runtime::tail_lines(&stderr_buf, 5);
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
