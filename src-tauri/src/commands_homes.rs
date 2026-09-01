//! Tauri commands(M2):home 登记/新建/克隆/绑定/删除。
//! 串行锁与原子落盘复用 commands::with_registry。

use crate::commands::{peek_registry, with_registry};
use crate::homes::{
    clone_dir, default_homes_root, ensure_path_free, register_home, validate_existing_home_path,
};
use crate::registry::{sanitize_id_fragment, HomeEntry, RegResult};
use std::path::PathBuf;

/// 列出全部 home。
#[tauri::command]
pub fn list_homes() -> RegResult<Vec<HomeEntry>> {
    Ok(peek_registry()?.homes)
}

/// 登记既有目录为 home(目录必须已存在)。
#[tauri::command]
pub fn add_home(path: String, id: Option<String>) -> RegResult<HomeEntry> {
    let dir = validate_existing_home_path(&path)?;
    with_registry(move |reg| register_home(reg, id, dir, None))
}

/// 新建空目录并登记为 home;path 缺省时落在 ~/.dsh-launcher/homes/<id>。
#[tauri::command]
pub fn create_home(path: Option<String>, id: Option<String>) -> RegResult<HomeEntry> {
    with_registry(move |reg| {
        let base = id
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .unwrap_or_else(|| reg.fresh_home_id("home"));
        let dir: PathBuf = match path.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
            Some(p) => crate::launcher::expand_tilde(p),
            None => default_homes_root().join(&base),
        };
        if dir.exists() && !dir.is_dir() {
            return Err(format!("路径已被文件占用: {}", dir.display()));
        }
        std::fs::create_dir_all(&dir).map_err(|e| format!("创建 {} 失败: {e}", dir.display()))?;
        ensure_path_free(reg, &dir, None)?;
        let entry = HomeEntry {
            id: reg.fresh_home_id(&sanitize_id_fragment(&base)),
            path: dir.to_string_lossy().into_owned(),
            bound_version_id: None,
            last_good_version_id: None,
        };
        reg.upsert_home(entry.clone())?;
        Ok(entry)
    })
}

/// 克隆 home:递归拷贝目录并登记为新 home,继承版本绑定。
#[tauri::command]
pub fn clone_home(
    source_id: String,
    new_path: Option<String>,
    new_id: Option<String>,
) -> RegResult<HomeEntry> {
    with_registry(move |reg| {
        let source = reg
            .find_home(&source_id)
            .cloned()
            .ok_or_else(|| format!("home {source_id:?} 不存在"))?;
        let src = crate::launcher::expand_tilde(&source.path);
        let base = new_id
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .unwrap_or_else(|| sanitize_id_fragment(&format!("{}-clone", source.id)));
        let dst: PathBuf = match new_path.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
            Some(p) => crate::launcher::expand_tilde(p),
            None => default_homes_root().join(reg.fresh_home_id(&base)),
        };
        clone_dir(&src, &dst)?;
        ensure_path_free(reg, &dst, None)?;
        let entry = HomeEntry {
            id: reg.fresh_home_id(&base),
            path: dst.to_string_lossy().into_owned(),
            bound_version_id: source.bound_version_id.clone(),
            last_good_version_id: None,
        };
        reg.upsert_home(entry.clone())?;
        Ok(entry)
    })
}

/// 绑定/解绑 home 的运行版本(version_id 传 null 解绑)。
#[tauri::command]
pub fn bind_home_version(home_id: String, version_id: Option<String>) -> RegResult<HomeEntry> {
    with_registry(move |reg| {
        if let Some(v) = version_id.as_deref() {
            reg.find_version(v)
                .ok_or_else(|| format!("版本 {v:?} 不存在"))?;
        }
        let slot = reg
            .find_home_mut(&home_id)
            .ok_or_else(|| format!("home {home_id:?} 不存在"))?;
        slot.bound_version_id = version_id;
        Ok(slot.clone())
    })
}

/// 删除 home:只摘登记,目录原样保留(DESIGN.md:home 只登记路径不搬家)。
#[tauri::command]
pub fn remove_home(home_id: String) -> RegResult<()> {
    with_registry(move |reg| {
        if !reg.remove_home(&home_id) {
            return Err(format!("home {home_id:?} 不存在"));
        }
        Ok(())
    })
}
