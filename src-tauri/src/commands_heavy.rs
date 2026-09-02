//! 长耗时命令:指纹实测、npm 安装、版本目录删除。
//! Tauri 同步命令跑在主线程上,会卡死界面;这里统一经 spawn_blocking
//! 把阻塞工作下放到后台线程池,命令本身改为 async,主线程保持响应。

use crate::commands::{peek_registry, with_registry};
use crate::launcher;
use crate::registry::{self, RegResult, VersionEntry, VersionKind};
use tauri::Emitter;

/// 把阻塞任务丢到后台线程池执行;join 失败(线程池故障)归一成字符串错误。
pub(crate) async fn run_blocking<T: Send + 'static>(
    task: impl FnOnce() -> RegResult<T> + Send + 'static,
) -> RegResult<T> {
    tauri::async_runtime::spawn_blocking(task)
        .await
        .map_err(|e| format!("后台任务执行失败: {e}"))?
}

/// 实测指纹:spawn 该版本 <bin> --version,把输出首行写回 fingerprint 字段。
#[tauri::command]
pub async fn fingerprint_version(id: String) -> RegResult<VersionEntry> {
    run_blocking(move || fingerprint_version_impl(id)).await
}

fn fingerprint_version_impl(id: String) -> RegResult<VersionEntry> {
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
pub async fn install_npm_version(
    app: tauri::AppHandle,
    version: String,
    id: Option<String>,
) -> RegResult<VersionEntry> {
    run_blocking(move || install_npm_version_impl(app, version, id)).await
}

fn install_npm_version_impl(
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
        tool: None,
    };
    with_registry(move |reg| {
        reg.upsert_version(entry.clone())?;
        Ok(entry)
    })
}

/// 删除版本:摘登记;npm kind 连 versions/<id> 目录一起删,dev/manual 只摘登记。
#[tauri::command]
pub async fn remove_version(id: String) -> RegResult<()> {
    run_blocking(move || remove_version_impl(id)).await
}

fn remove_version_impl(id: String) -> RegResult<()> {
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
