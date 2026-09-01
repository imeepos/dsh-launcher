//! Tauri commands(M0+M1)。
//! M0:list_versions / add_manual_version / fingerprint_version
//! M1:install_npm_version / add_dev_version / remove_version
//!
//! 所有注册表变更经 with_registry 串行化(进程内互斥 + 原子落盘)。

use crate::launcher;
use crate::registry::{self, RegResult, Registry, VersionEntry, VersionKind};
use std::sync::{Mutex, OnceLock};
use tauri::Emitter;

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

/// npm 安装一个 DSH 版本(DESIGN.md:npm install --prefix versions/<id>)。
/// 安装进度通过 install-progress 事件逐行推给前端;失败时注册表不落任何记录,
/// 前端可直接重试(重试会自动清理半成品目录)。
#[tauri::command]
pub fn install_npm_version(
    app: tauri::AppHandle,
    version: String,
    id: Option<String>,
) -> RegResult<VersionEntry> {
    let version = version.trim().trim_start_matches('v').to_string();
    if version.is_empty() {
        return Err("版本号不能为空".into());
    }
    let spec = format!("@deepseek-ai/dsh@{version}");
    let id = match id.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        Some(x) => x.to_string(),
        None => format!("v{version}"),
    };
    registry::validate_id(&id)?;
    if peek_registry()?.find_version(&id).is_some() {
        return Err(format!("版本 {id} 已存在,请换一个 id 或先删除"));
    }

    let emitter = app;
    let emit_id = id.clone();
    let bin_path = launcher::install_npm(&spec, &id, move |line| {
        let _ = emitter.emit(
            "install-progress",
            serde_json::json!({ "id": emit_id, "line": line }),
        );
    })?;

    let entry = VersionEntry {
        id,
        kind: VersionKind::Npm,
        spec: Some(spec),
        bin: bin_path.to_string_lossy().into_owned(),
        cwd: None,
        fingerprint: None,
        added_at_ms: Some(registry::now_ms()),
    };
    with_registry(move |reg| {
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

/// 删除版本:摘登记;npm kind 连 versions/<id> 目录一起删,dev/manual 只摘登记。
#[tauri::command]
pub fn remove_version(id: String) -> RegResult<()> {
    let entry = with_registry(|reg| {
        let e = reg
            .find_version(&id)
            .cloned()
            .ok_or_else(|| format!("版本 {id:?} 不存在"))?;
        reg.remove_version(&id);
        Ok(e)
    })?;
    if entry.kind == VersionKind::Npm {
        launcher::remove_version_dir(&id)?;
    }
    Ok(())
}
