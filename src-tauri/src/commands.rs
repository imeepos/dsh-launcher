//! Tauri commands(M0):list_versions / add_manual_version / fingerprint_version。
//!
//! 所有注册表变更经 with_registry 串行化(进程内互斥 + 原子落盘),
//! 避免并发命令交叉读写 registry.json。

use crate::launcher;
use crate::registry::{self, RegResult, Registry, VersionEntry, VersionKind};
use std::sync::{Mutex, OnceLock};

fn registry_io_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn with_registry<T>(f: impl FnOnce(&mut Registry) -> RegResult<T>) -> RegResult<T> {
    let _guard = registry_io_lock().lock().map_err(|e| e.to_string())?;
    let path = registry::default_registry_path();
    let mut reg = registry::load(&path)?;
    let out = f(&mut reg)?;
    registry::save(&path, &reg)?;
    Ok(out)
}

fn peek_registry() -> RegResult<Registry> {
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

/// 实测指纹:spawn 该版本 <bin> --version,把输出首行写回 fingerprint 字段。
#[tauri::command]
pub fn fingerprint_version(id: String) -> RegResult<VersionEntry> {
    let entry = peek_registry()?
        .find_version(&id)
        .cloned()
        .ok_or_else(|| format!("版本 {id:?} 不存在"))?;
    let fp = launcher::fingerprint(&entry.bin, entry.cwd.as_deref())?;
    with_registry(|reg| {
        let slot = reg
            .find_version_mut(&id)
            .ok_or_else(|| format!("版本 {id:?} 已不存在"))?;
        slot.fingerprint = Some(fp.clone());
        Ok(())
    })?;
    let mut updated = entry;
    updated.fingerprint = Some(fp);
    Ok(updated)
}
