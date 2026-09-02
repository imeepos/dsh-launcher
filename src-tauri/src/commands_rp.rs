//! Tauri commands:release-platform 连接配置、目录浏览、制品安装(DESIGN-TOOLS.md §2)。
//! password 模式的 JWT 只驻内存 RpSession,不落盘;列表数据原样透传 Value。

use crate::commands::{peek_registry, with_registry};
use crate::commands_heavy::run_blocking;
use crate::registry::{
    launcher_base_dir, now_ms, sanitize_id_fragment, RegResult, RpAuth, RpAuthMode, RpSettings,
    VersionEntry, VersionKind,
};
use crate::rp::{self, RpClient};
use serde_json::{json, Value};
use std::path::Path;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager};

/// password 模式会话 JWT;连接成功后写入,进程生命周期内复用。
#[derive(Default)]
pub struct RpSession(pub Mutex<Option<String>>);

pub(crate) fn default_rp_settings() -> RpSettings {
    RpSettings {
        base_url: rp::DEFAULT_BASE_URL.into(),
        auth: None,
    }
}

fn rp_settings() -> RegResult<RpSettings> {
    Ok(peek_registry()
        ?.settings
        .rp
        .clone()
        .unwrap_or_else(default_rp_settings))
}

fn jwt_from(app: &AppHandle) -> RegResult<Option<String>> {
    Ok(app
        .state::<RpSession>()
        .0
        .lock()
        .map_err(|e| e.to_string())?
        .clone())
}

#[tauri::command]
pub fn rp_get_config() -> RegResult<RpSettings> {
    rp_settings()
}

/// 保存连接配置;base_url 去尾斜杠,auth 原样覆盖(传 null 清除)。
#[tauri::command]
pub fn rp_set_config(base_url: String, auth: Option<RpAuth>) -> RegResult<RpSettings> {
    let base_url = base_url.trim().trim_end_matches('/').to_string();
    if base_url.is_empty() {
        return Err("base_url 不能为空".into());
    }
    let cfg = RpSettings { base_url, auth };
    with_registry(|reg| {
        reg.settings.rp = Some(cfg.clone());
        Ok(())
    })?;
    Ok(cfg)
}

/// 连接测试:password 模式先换 JWT 存会话,再拉 /v1/products 探活。
#[tauri::command]
pub async fn rp_connect(app: AppHandle) -> RegResult<Value> {
    run_blocking(move || rp_connect_impl(&app)).await
}

fn rp_connect_impl(app: &AppHandle) -> RegResult<Value> {
    let settings = rp_settings()?;
    let jwt = match settings.auth.as_ref().map(|a| a.mode) {
        Some(RpAuthMode::Password) => {
            let a = settings.auth.as_ref().expect("mode 存在则 auth 存在");
            let issuer = a
                .issuer_url
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .ok_or("password 模式需要 issuerUrl")?;
            let user = a
                .username
                .as_deref()
                .filter(|s| !s.is_empty())
                .ok_or("password 模式需要 username")?;
            let pass = a
                .password
                .as_deref()
                .filter(|s| !s.is_empty())
                .ok_or("password 模式需要 password")?;
            Some(rp::password_grant(issuer, user, pass)?)
        }
        _ => None,
    };
    if let Some(j) = &jwt {
        *app.state::<RpSession>()
            .0
            .lock()
            .map_err(|e| e.to_string())? = Some(j.clone());
    }
    let client = RpClient::new(&settings, jwt.as_deref());
    let products = client.get_json("/v1/products", &[])?;
    let count = rp::envelope_items(&products).len();
    Ok(json!({ "ok": true, "baseUrl": client.base_url(), "products": count }))
}

#[tauri::command]
pub async fn rp_list_products(app: AppHandle) -> RegResult<Vec<Value>> {
    run_blocking(move || {
        let client = RpClient::new(&rp_settings()?, jwt_from(&app)?.as_deref());
        Ok(rp::envelope_items(&client.get_json("/v1/products", &[])?))
    })
    .await
}

/// 发布流(浏览主入口,时间倒序);channel/status/limit 可选。
#[tauri::command]
pub async fn rp_list_releases(
    app: AppHandle,
    channel: Option<String>,
    status: Option<String>,
    limit: Option<u32>,
) -> RegResult<Vec<Value>> {
    run_blocking(move || {
        let client = RpClient::new(&rp_settings()?, jwt_from(&app)?.as_deref());
        let limit_s = limit.map_or_else(|| "50".to_string(), |l| l.to_string());
        let query = vec![
            ("channel", channel.as_deref().unwrap_or("")),
            ("status", status.as_deref().unwrap_or("")),
            ("limit", limit_s.as_str()),
        ];
        Ok(rp::envelope_items(&client.get_json("/v1/releases", &query)?))
    })
    .await
}

/// 制品列表;os/arch 缺省用本机映射值,传空串表示不过滤。
#[tauri::command]
pub async fn rp_list_artifacts(
    app: AppHandle,
    version_id: String,
    os: Option<String>,
    arch: Option<String>,
) -> RegResult<Vec<Value>> {
    run_blocking(move || rp_list_artifacts_impl(&app, version_id, os, arch)).await
}

fn rp_list_artifacts_impl(
    app: &AppHandle,
    version_id: String,
    os: Option<String>,
    arch: Option<String>,
) -> RegResult<Vec<Value>> {
    let client = RpClient::new(&rp_settings()?, jwt_from(app)?.as_deref());
    let os_f = os
        .as_deref()
        .filter(|s| !s.is_empty())
        .map_or_else(|| rp::local_os().to_string(), String::from);
    let arch_f = arch
        .as_deref()
        .filter(|s| !s.is_empty())
        .map_or_else(|| rp::local_arch().to_string(), String::from);
    let query = vec![
        ("version_id", version_id.as_str()),
        ("os", os_f.as_str()),
        ("arch", arch_f.as_str()),
    ];
    Ok(rp::envelope_items(&client.get_json("/v1/artifacts", &query)?))
}

/// 安装制品:download-url → 流式下载(.part + sha256 校验)→ chmod +x → 登记版本。
/// 进度经 install-progress 事件推送,id 形如 rp-<artifactId>。
#[tauri::command]
pub async fn rp_install_artifact(
    app: AppHandle,
    artifact_id: String,
    tool: Option<String>,
    semver: Option<String>,
) -> RegResult<VersionEntry> {
    run_blocking(move || rp_install_impl(app, artifact_id, tool, semver)).await
}

fn emit_progress(app: &AppHandle, job: &str, line: String) {
    let _ = app.emit("install-progress", json!({ "id": job, "line": line }));
}

fn rp_install_impl(
    app: AppHandle,
    artifact_id: String,
    tool: Option<String>,
    semver: Option<String>,
) -> RegResult<VersionEntry> {
    let artifact_id = artifact_id.trim().to_string();
    if artifact_id.is_empty() {
        return Err("artifactId 不能为空".into());
    }
    let client = RpClient::new(&rp_settings()?, jwt_from(&app)?.as_deref());
    let artifact = client.get_json(&format!("/v1/artifacts/{artifact_id}"), &[])?;
    let dl = client.get_json(&format!("/v1/artifacts/{artifact_id}/download-url"), &[])?;
    let url = dl
        .get("download_url")
        .and_then(|v| v.as_str())
        .ok_or("download-url 响应缺少 download_url")?
        .to_string();

    let tool_name = sanitize_id_fragment(
        tool.as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .unwrap_or("tool"),
    );
    let dir = launcher_base_dir().join("tools").join(&tool_name);
    std::fs::create_dir_all(&dir).map_err(|e| format!("创建 {} 失败: {e}", dir.display()))?;
    let filename = rp::filename_from_url(&url, &format!("{artifact_id}.bin"));
    let dest = dir.join(&filename);
    let part = dir.join(format!("{filename}.part"));

    let job = format!("rp-{artifact_id}");
    emit_progress(&app, &job, format!("开始下载 {artifact_id}…"));
    let mut progress = |line: String| emit_progress(&app, &job, line);
    let (bytes, sha_hex) = rp::download_to_file(&url, &part, &mut progress)?;
    verify_sha256(&artifact, &sha_hex, bytes, &part)?;
    std::fs::rename(&part, &dest).map_err(|e| format!("落盘 {} 失败: {e}", dest.display()))?;
    make_executable(&dest)?;
    emit_progress(&app, &job, format!("已安装到 {}", dest.display()));

    let spec = semver
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from);
    let id_base = match &spec {
        Some(v) => format!("{tool_name}-v{v}"),
        None => format!("{tool_name}-{artifact_id}"),
    };
    with_registry(move |reg| {
        let entry = VersionEntry {
            id: reg.fresh_id(&id_base),
            kind: VersionKind::Manual,
            spec: spec.clone(),
            bin: dest.to_string_lossy().into_owned(),
            cwd: None,
            fingerprint: None,
            added_at_ms: Some(now_ms()),
            tool: Some(tool_name.clone()),
        };
        reg.upsert_version(entry.clone())?;
        Ok(entry)
    })
}

fn verify_sha256(artifact: &Value, actual: &str, bytes: u64, part: &Path) -> RegResult<()> {
    let expect = artifact
        .get("sha256")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    if expect.is_empty() {
        return Ok(());
    }
    if !actual.eq_ignore_ascii_case(&expect) {
        let _ = std::fs::remove_file(part);
        return Err(format!(
            "sha256 不匹配(实际 {bytes} bytes): 期望 {expect}, 实得 {actual}"
        ));
    }
    Ok(())
}

fn make_executable(path: &Path) -> RegResult<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perm = std::fs::metadata(path)
            .map_err(|e| format!("stat {} 失败: {e}", path.display()))?
            .permissions();
        perm.set_mode(perm.mode() | 0o755);
        std::fs::set_permissions(path, perm).map_err(|e| e.to_string())?;
    }
    Ok(())
}
