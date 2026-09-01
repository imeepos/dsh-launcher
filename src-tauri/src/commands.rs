//! Tauri commands(M0+M1)。
//! M0:list_versions / add_manual_version / fingerprint_version
//! M1:install_npm_version / add_dev_version / remove_version
//!
//! 所有注册表变更经 with_registry 串行化(进程内互斥 + 原子落盘)。
//! 长耗时命令(指纹/npm 安装/删除)在 commands_heavy,经 spawn_blocking 后台执行。

use crate::launcher;
use crate::registry::{self, RegResult, Registry, VersionEntry, VersionKind};
use std::sync::{Mutex, OnceLock};

pub(crate) fn registry_io_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

pub(crate) fn with_registry<T>(f: impl FnOnce(&mut Registry) -> RegResult<T>) -> RegResult<T> {
    let _guard = registry_io_lock().lock().map_err(|e| e.to_string())?;
    let path = registry::default_registry_path();
    let mut reg = registry::load(&path)?;
    let out = f(&mut reg)?;
    registry::save(&path, &reg)?;
    Ok(out)
}

pub(crate) fn peek_registry() -> RegResult<Registry> {
    let _guard = registry_io_lock().lock().map_err(|e| e.to_string())?;
    registry::load(&registry::default_registry_path())
}

/// 列出全部版本(含指纹字段)。
#[tauri::command]
pub fn list_versions() -> RegResult<Vec<VersionEntry>> {
    Ok(peek_registry()?.versions)
}

/// 手动登记既有 dsh 可执行文件:用户指定 bin(必填)与 cwd(可选)。
#[tauri::command]
pub fn add_manual_version(
    bin: String,
    cwd: Option<String>,
    id: Option<String>,
) -> RegResult<VersionEntry> {
    let bin = bin.trim().to_string();
    if bin.is_empty() {
        return Err("bin 不能为空".into());
    }
    with_registry(move |reg| {
        let base = match id.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
            Some(x) => x.to_string(),
            None => {
                let name = std::path::Path::new(&bin)
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| "manual".into());
                format!("manual-{name}")
            }
        };
        let entry = VersionEntry {
            id: reg.fresh_id(&base),
            kind: VersionKind::Manual,
            spec: None,
            bin: launcher::expand_tilde(&bin).to_string_lossy().into_owned(),
            cwd: cwd
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(|c| launcher::expand_tilde(c).to_string_lossy().into_owned()),
            fingerprint: None,
            added_at_ms: Some(registry::now_ms()),
        };
        reg.upsert_version(entry.clone())?;
        Ok(entry)
    })
}

/// 登记一个 dev repo checkout:启动命令 pnpm dsh,cwd=repoPath。
#[tauri::command]
pub fn add_dev_version(repo_path: String, id: Option<String>) -> RegResult<VersionEntry> {
    let repo = launcher::validate_dev_repo(&repo_path)?;
    with_registry(move |reg| {
        let base = match id.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
            Some(x) => x.to_string(),
            None => {
                let name = repo
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| "repo".into());
                format!("dev-{}", registry::sanitize_id_fragment(&name))
            }
        };
        let entry = VersionEntry {
            id: reg.fresh_id(&base),
            kind: VersionKind::Dev,
            spec: None,
            bin: launcher::DEV_BIN.into(),
            cwd: Some(repo.to_string_lossy().into_owned()),
            fingerprint: None,
            added_at_ms: Some(registry::now_ms()),
        };
        reg.upsert_version(entry.clone())?;
        Ok(entry)
    })
}
