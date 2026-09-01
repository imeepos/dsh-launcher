//! managed Node 运行时安装:双源下载 → sha256 校验 → 解压 → 落 runtime.json。
//! 全程用户态无 sudo;任一步骤失败自动换源重试一次(设计文档 §1)。

use crate::registry::{self, RegResult};
use crate::runtime_install_core as core;
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::process::Command;

/// 已安运行时的记录,落在 base/runtime.json,是运行时的唯一事实源。
#[derive(Clone, Debug, Serialize, serde::Deserialize)]
pub struct RuntimeInfo {
    pub node_version: String,
    pub bin: PathBuf,
    pub installed_at_ms: u64,
    pub sha256: String,
    pub source: String,
}

pub fn runtime_json_path() -> PathBuf {
    registry::launcher_base_dir().join("runtime.json")
}

/// 读取已安装运行时;未安装或文件损坏返回 None。
pub fn read_runtime() -> Option<RuntimeInfo> {
    read_runtime_from(&registry::launcher_base_dir())
}

/// 从指定 base 目录读取 runtime.json。
pub fn read_runtime_from(base: &Path) -> Option<RuntimeInfo> {
    let text = std::fs::read_to_string(base.join("runtime.json")).ok()?;
    serde_json::from_str(&text).ok()
}

/// 安装入口:装到 launcher base 目录(生产路径)。
pub fn install_runtime(on_line: impl FnMut(&str)) -> RegResult<RuntimeInfo> {
    install_runtime_into(&registry::launcher_base_dir(), on_line)
}

/// 重装:清掉 runtime 目录与 runtime.json 后全新安装(修复中心用)。
pub fn reinstall_runtime(mut on_line: impl FnMut(&str)) -> RegResult<RuntimeInfo> {
    let base = registry::launcher_base_dir();
    let rt = base.join("runtime");
    if rt.exists() {
        on_line("清理旧运行时目录");
        std::fs::remove_dir_all(&rt).map_err(|e| format!("清理运行时失败:{e}"))?;
    }
    let _ = std::fs::remove_file(base.join("runtime.json"));
    install_runtime_into(&base, on_line)
}

/// 安装到指定目录(冒烟测试可注入临时目录)。
pub fn install_runtime_into(base: &Path, mut on_line: impl FnMut(&str)) -> RegResult<RuntimeInfo> {
    let version = resolve_version(&mut on_line);
    on_line(&format!("选定 Node 版本 {version}"));
    let name = core::tarball_name(&version, os_dir(), arch_dir());
    let mut last_err = "无可用源".to_string();
    for (src_name, src_base) in core::SOURCES {
        on_line(&format!("尝试源 {src_name}"));
        match try_source(src_name, src_base, &version, &name, &base, &mut on_line) {
            Ok(info) => return Ok(info),
            Err(e) => {
                on_line(&format!("源 {src_name} 失败:{e}"));
                last_err = e;
            }
        }
    }
    Err(format!("运行时安装失败:{last_err}"))
}

fn try_source(
    src_name: &str,
    src_base: &str,
    version: &str,
    name: &str,
    base: &Path,
    on_line: &mut impl FnMut(&str),
) -> RegResult<RuntimeInfo> {
    let tmp = std::env::temp_dir().join(format!("dsh-node-{}-{name}", std::process::id()));
    let result = install_from(src_name, src_base, version, name, base, &tmp, on_line);
    let _ = std::fs::remove_file(&tmp);
    result
}

fn install_from(
    src_name: &str,
    src_base: &str,
    version: &str,
    name: &str,
    base: &Path,
    tmp: &Path,
    on_line: &mut impl FnMut(&str),
) -> RegResult<RuntimeInfo> {
    let shasums =
        fetch_text(&core::shasums_url(src_base, version)).ok_or("获取 SHASUMS256 失败")?;
    download(&core::tarball_url(src_base, version, name), tmp, on_line)?;
    let sha = sha256_of(tmp)?;
    let expected = core::expected_sha256(&shasums, name).ok_or("清单中找不到该文件")?;
    if sha != expected {
        return Err(format!("sha256 不匹配:期望 {expected} 实得 {sha}"));
    }
    on_line("sha256 校验通过");
    extract(tmp, base)?;
    let bin = base
        .join("runtime")
        .join(format!("node-{version}-{}-{}", os_dir(), arch_dir()))
        .join("bin");
    if !bin.join("node").is_file() {
        return Err("解压后未找到 node 可执行文件".into());
    }
    let info = RuntimeInfo {
        node_version: version.to_string(),
        bin,
        installed_at_ms: registry::now_ms(),
        sha256: sha,
        source: src_name.to_string(),
    };
    write_runtime_json(base, &info)?;
    Ok(info)
}

/// 版本号:双源拉 index.json 选最新 LTS;都不可达用兜底版本。
fn resolve_version(on_line: &mut impl FnMut(&str)) -> String {
    for (_, src_base) in core::SOURCES {
        let url = format!("{src_base}/index.json");
        if let Some(text) = fetch_text(&url) {
            if let Some(v) = core::pick_lts_version(&text) {
                return v;
            }
        }
    }
    on_line("版本索引不可达,使用内置兜底版本");
    core::FALLBACK_VERSION.to_string()
}

fn fetch_text(url: &str) -> Option<String> {
    let out = Command::new("curl")
        .args(["-fsSL", "--max-time", "20", url])
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).into_owned())
}

fn download(url: &str, dest: &Path, on_line: &mut impl FnMut(&str)) -> RegResult<()> {
    on_line(&format!("下载 {url}"));
    let status = Command::new("curl")
        .args(["-fSL", "--retry", "2", "--max-time", "900", "-o"])
        .arg(dest)
        .arg(url)
        .status()
        .map_err(|e| format!("启动 curl 失败:{e}"))?;
    status
        .success()
        .then(|| on_line("下载完成"))
        .ok_or_else(|| "下载失败(curl 非零退出)".to_string())
}

/// sha256:优先 sha256sum(Linux),回退 shasum -a 256(macOS)。
fn sha256_of(path: &Path) -> RegResult<String> {
    let out = Command::new("sha256sum")
        .arg(path)
        .output()
        .or_else(|_| {
            Command::new("shasum")
                .args(["-a", "256"])
                .arg(path)
                .output()
        })
        .map_err(|e| format!("找不到 sha256 工具:{e}"))?;
    if !out.status.success() {
        return Err("计算 sha256 失败".into());
    }
    String::from_utf8_lossy(&out.stdout)
        .split_whitespace()
        .next()
        .map(str::to_lowercase)
        .ok_or_else(|| "sha256 输出为空".into())
}

fn extract(tarball: &Path, base: &Path) -> RegResult<()> {
    let runtime_dir = base.join("runtime");
    std::fs::create_dir_all(&runtime_dir).map_err(|e| format!("创建 runtime 目录失败:{e}"))?;
    let status = Command::new("tar")
        .args(["-xJf"])
        .arg(tarball)
        .arg("-C")
        .arg(&runtime_dir)
        .status()
        .map_err(|e| format!("启动 tar 失败:{e}"))?;
    status
        .success()
        .then_some(())
        .ok_or_else(|| "解压失败(tar 非零退出)".to_string())
}

fn write_runtime_json(base: &Path, info: &RuntimeInfo) -> RegResult<()> {
    std::fs::create_dir_all(base).map_err(|e| format!("创建 base 目录失败:{e}"))?;
    let text = serde_json::to_string_pretty(info).map_err(|e| format!("序列化失败:{e}"))?;
    std::fs::write(base.join("runtime.json"), text).map_err(|e| format!("写 runtime.json 失败:{e}"))
}

fn os_dir() -> &'static str {
    if cfg!(target_os = "macos") {
        "darwin"
    } else {
        "linux"
    }
}

fn arch_dir() -> &'static str {
    match std::env::consts::ARCH {
        "aarch64" => "arm64",
        _ => "x64",
    }
}
