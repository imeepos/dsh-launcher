//! npm 安装逻辑:npm install --prefix versions/<id>,每版本独立依赖树。

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

use super::launcher_runtime::{
    expand_tilde, home_dir, make_group_leader, strip_dsh_env, tail_lines, wait_with_output_lines,
    INSTALL_TIMEOUT,
};
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

/// npm 安装一个版本。重试语义:自动清掉已存在的 versions/<id> 目录,从头安装。
pub fn install_npm(spec: &str, id: &str, on_line: impl FnMut(&str)) -> RegResult<PathBuf> {
    validate_id(id)?;
    let dir = versions_dir().join(id);
    let npm = resolve_npm()?;
    install_npm_into(&dir, &npm, spec, INSTALL_TIMEOUT, on_line)
}

/// 安装到指定目录,npm 路径与超时可注入(测试用假 npm)。
/// 成功返回 node_modules/.bin/dsh 路径。
pub fn install_npm_into(
    dir: &Path,
    npm: &Path,
    spec: &str,
    timeout: Duration,
    mut on_line: impl FnMut(&str),
) -> RegResult<PathBuf> {
    if dir.exists() {
        on_line(&format!("清理旧目录 {}", dir.display()));
        fs::remove_dir_all(dir).map_err(|e| format!("清理旧目录 {} 失败: {e}", dir.display()))?;
    }
    fs::create_dir_all(dir).map_err(|e| format!("创建 {} 失败: {e}", dir.display()))?;
    let mut cmd = Command::new(npm);
    cmd.args(["install", "--prefix"]).arg(dir).arg(spec);
    strip_dsh_env(&mut cmd);
    make_group_leader(&mut cmd);
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
    let relay = |is_err: bool, l: &str| {
        if is_err {
            on_line(&format!("[stderr] {l}"));
        } else {
            on_line(l);
        }
    };
    let (_stdout, stderr_buf, status) = wait_with_output_lines(
        &mut child,
        timeout,
        relay,
        Some((Duration::from_secs(5), "…仍在安装中")),
    )?;
    let status = status.ok_or_else(|| {
        format!(
            "npm install 超时({}s),进程已终止\n{}",
            timeout.as_secs(),
            tail_lines(&stderr_buf, 15)
        )
    })?;
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

/// 删除 npm 版本的独立依赖树目录 versions/<id>。
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
