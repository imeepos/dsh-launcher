//! 进程环境与 npm 解析:把 managed runtime 前置进 PATH,让 npm/dsh 都用自带
//! Node(新装系统上没有系统 Node);settings.useSystemNode 可切回系统 Node。

use super::launcher_runtime::{expand_tilde, strip_dsh_env};
use crate::registry::RegResult;
use crate::runtime_install;
use std::path::PathBuf;
use std::process::Command;

/// managed runtime 的 bin 目录;未安装返回 None。
pub(crate) fn runtime_bin_dir() -> Option<PathBuf> {
    runtime_install::read_runtime().map(|r| r.bin)
}

/// 给命令前置 managed runtime 的 PATH(未安装则不动)。
/// dsh/npm 的 shebang 都是 env node,新装系统上不注入就找不到 node。
pub(crate) fn inject_runtime_path(cmd: &mut Command) {
    let Some(bin) = runtime_bin_dir() else { return };
    let mut paths = vec![bin];
    if let Some(cur) = std::env::var_os("PATH") {
        paths.extend(std::env::split_paths(&cur));
    }
    if let Ok(joined) = std::env::join_paths(&paths) {
        cmd.env("PATH", joined);
    }
}

/// spawn dsh/npm 的统一环境准备:清游离 DSH_*(DESIGN.md §1)+ 前置 runtime PATH。
pub(crate) fn prepare_cmd_env(cmd: &mut Command) {
    strip_dsh_env(cmd);
    inject_runtime_path(cmd);
}

pub(crate) fn resolve_npm() -> RegResult<PathBuf> {
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
    let managed = runtime_bin_dir()
        .map(|b| b.join("npm"))
        .filter(|p| p.is_file());
    let which_fn = |n: &str| which(n);
    pick_npm(None, managed, use_system_node_pref(), which_fn)
        .or_else(|| npm_candidates().into_iter().find(|c| c.is_file()))
        .ok_or_else(|| "找不到 npm:不在 PATH。可设 DSH_LAUNCHER_NPM 环境变量指定 npm 路径".into())
}

/// npm 解析顺序:显式覆盖 > managed runtime(除非 useSystemNode)> 系统 PATH。
fn pick_npm(
    override_path: Option<PathBuf>,
    managed: Option<PathBuf>,
    use_system: bool,
    which: impl Fn(&str) -> Option<PathBuf>,
) -> Option<PathBuf> {
    if let Some(p) = override_path {
        return Some(p);
    }
    if !use_system && managed.is_some() {
        return managed;
    }
    which("npm")
}

/// settings.useSystemNode;读注册表失败按 false 处理(优先自带运行时)。
fn use_system_node_pref() -> bool {
    crate::commands::peek_registry()
        .ok()
        .map(|r| r.settings.use_system_node)
        .unwrap_or(false)
}

fn npm_candidates() -> [PathBuf; 3] {
    use super::launcher_runtime::home_dir;
    [
        home_dir().join(".vite-plus/bin/npm"),
        PathBuf::from("/opt/homebrew/bin/npm"),
        PathBuf::from("/usr/local/bin/npm"),
    ]
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
fn is_executable(p: &std::path::Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    p.is_file()
        && p.metadata()
            .map(|m| m.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::pick_npm;
    use std::path::PathBuf;

    fn p(s: &str) -> Option<PathBuf> {
        Some(PathBuf::from(s))
    }

    #[test]
    fn override_wins_over_everything() {
        let got = pick_npm(p("/a"), p("/m"), false, |_: &str| p("/w"));
        assert_eq!(got, p("/a"));
    }

    #[test]
    fn managed_beats_system_path() {
        let got = pick_npm(None, p("/m"), false, |_: &str| p("/w"));
        assert_eq!(got, p("/m"));
    }

    #[test]
    fn use_system_skips_managed() {
        let got = pick_npm(None, p("/m"), true, |_: &str| p("/w"));
        assert_eq!(got, p("/w"));
    }

    #[test]
    fn falls_back_to_system_path() {
        let got = pick_npm(None, None, false, |_: &str| p("/w"));
        assert_eq!(got, p("/w"));
    }

    #[test]
    fn none_when_nothing_available() {
        let got = pick_npm(None, None, false, |_: &str| None);
        assert_eq!(got, None);
    }
}
