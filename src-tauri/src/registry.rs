//! ~/.dsh-launcher/registry.json 的数据模型与读写(DESIGN.md「数据模型」一节)。
//!
//! 写盘一律走原子替换:先写同目录临时文件并 fsync,再 rename 覆盖目标,
//! 崩溃/断电不会留下半截 registry.json。

use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub type RegResult<T> = Result<T, String>;

/// 版本来源。DESIGN.md schema 注释为 npm | dev;
/// manual 是 M0「手动指定 bin/cwd」引入的第三类,删除语义同 dev(只摘登记,不动文件)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VersionKind {
    /// npm 安装:~/.dsh-launcher/versions/<id> 下独立依赖树
    Npm,
    /// 登记 repo checkout:启动命令 pnpm dsh,cwd=repoPath
    Dev,
    /// 手动登记既有可执行文件
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionEntry {
    pub id: String,
    pub kind: VersionKind,
    /// npm 形如 "@deepseek-ai/dsh@0.1.1-rc.2";dev/manual 可空
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spec: Option<String>,
    /// 启动命令:npm 为 bin 绝对路径;dev 为 "pnpm dsh";manual 为用户指定
    pub bin: String,
    /// dev kind 为 repoPath,其余可空
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    /// spawn 该版本 <bin> --version 实测写回(DESIGN.md「fingerprint」字段)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fingerprint: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub added_at_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HomeEntry {
    pub id: String,
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bound_version_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_good_version_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEntry {
    /// epoch 毫秒
    pub started_at: u64,
    pub home_id: String,
    pub profile: String,
    pub version_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Registry {
    #[serde(default)]
    pub versions: Vec<VersionEntry>,
    /// M2 起启用;M0/M1 先保证 schema 完整落盘
    #[serde(default)]
    pub homes: Vec<HomeEntry>,
    /// M3 起启用
    #[serde(default)]
    pub history: Vec<HistoryEntry>,
}

impl Registry {
    pub fn find_version(&self, id: &str) -> Option<&VersionEntry> {
        self.versions.iter().find(|v| v.id == id)
    }

    pub fn find_version_mut(&mut self, id: &str) -> Option<&mut VersionEntry> {
        self.versions.iter_mut().find(|v| v.id == id)
    }

    /// 同 id 视为整体替换(指纹写回场景);新 id 走合法性校验。
    pub fn upsert_version(&mut self, entry: VersionEntry) -> RegResult<()> {
        if let Some(slot) = self.find_version_mut(&entry.id) {
            *slot = entry;
            return Ok(());
        }
        validate_id(&entry.id)?;
        self.versions.push(entry);
        Ok(())
    }

    pub fn remove_version(&mut self, id: &str) -> bool {
        let before = self.versions.len();
        self.versions.retain(|v| v.id != id);
        self.versions.len() != before
    }

    /// 基于 base 生成未占用的 id:base、base-2、base-3…
    pub fn fresh_id(&self, base: &str) -> String {
        if self.find_version(base).is_none() {
            return base.to_string();
        }
        for n in 2.. {
            let candidate = format!("{base}-{n}");
            if self.find_version(&candidate).is_none() {
                return candidate;
            }
        }
        unreachable!()
    }
}

/// id 兼作 versions/<id> 目录名,收紧字符集防目录穿越。
pub fn validate_id(id: &str) -> RegResult<()> {
    if id.is_empty() {
        return Err("id 不能为空".into());
    }
    if id.starts_with('.') {
        return Err(format!("id 不能以 . 开头: {id:?}"));
    }
    if !id
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_' | '+'))
    {
        return Err(format!("id 只允许字母数字与 . _ - +,得到 {id:?}"));
    }
    Ok(())
}

pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

pub fn launcher_base_dir() -> PathBuf {
    PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| ".".into())).join(".dsh-launcher")
}

pub fn default_registry_path() -> PathBuf {
    launcher_base_dir().join("registry.json")
}

pub fn versions_dir() -> PathBuf {
    launcher_base_dir().join("versions")
}

/// 读注册表;文件不存在视为空注册表(首次运行)。
pub fn load(path: &Path) -> RegResult<Registry> {
    match fs::read_to_string(path) {
        Ok(text) => serde_json::from_str(&text)
            .map_err(|e| format!("registry.json 解析失败 ({}): {e}", path.display())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Registry::default()),
        Err(e) => Err(format!("registry.json 读取失败 ({}): {e}", path.display())),
    }
}

/// 原子写:临时文件 + fsync + rename。
pub fn save(path: &Path, reg: &Registry) -> RegResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("创建 {} 失败: {e}", parent.display()))?;
    }
    let json = serde_json::to_string_pretty(reg).map_err(|e| format!("序列化失败: {e}"))?;
    let tmp = path.with_extension("json.tmp");
    {
        let mut f = fs::File::create(&tmp)
            .map_err(|e| format!("创建临时文件 {} 失败: {e}", tmp.display()))?;
        f.write_all(json.as_bytes())
            .and_then(|_| f.write_all(b"\n"))
            .and_then(|_| f.sync_all())
            .map_err(|e| format!("写临时文件 {} 失败: {e}", tmp.display()))?;
    }
    fs::rename(&tmp, path).map_err(|e| format!("原子替换 {} 失败: {e}", path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_registry_path(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join(format!("dsh-launcher-test-{}-{tag}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        dir.join("registry.json")
    }

    fn sample(id: &str, kind: VersionKind) -> VersionEntry {
        VersionEntry {
            id: id.into(),
            kind,
            spec: (kind == VersionKind::Npm).then(|| "@deepseek-ai/dsh@0.1.1-rc.2".to_string()),
            bin: "~/.dsh-launcher/versions/x/node_modules/.bin/dsh".into(),
            cwd: (kind == VersionKind::Dev).then(|| "/tmp/repo".to_string()),
            fingerprint: None,
            added_at_ms: Some(1234567890),
        }
    }

    #[test]
    fn load_missing_file_returns_default() {
        let p = temp_registry_path("missing");
        let reg = load(&p).expect("missing file should load as default");
        assert!(reg.versions.is_empty() && reg.homes.is_empty() && reg.history.is_empty());
    }

    #[test]
    fn save_load_roundtrip() {
        let p = temp_registry_path("roundtrip");
        let mut reg = Registry::default();
        reg.versions.push(sample("v0.1.1-rc.2", VersionKind::Npm));
        reg.versions.push(sample("dev-repo", VersionKind::Dev));
        reg.homes.push(HomeEntry {
            id: "main".into(),
            path: "~/.dsh".into(),
            bound_version_id: Some("v0.1.1-rc.2".into()),
            last_good_version_id: None,
        });
        reg.history.push(HistoryEntry {
            started_at: 42,
            home_id: "main".into(),
            profile: "default".into(),
            version_id: "v0.1.1-rc.2".into(),
            exit_code: Some(0),
        });
        save(&p, &reg).expect("save ok");
        let loaded = load(&p).expect("load ok");
        assert_eq!(loaded.versions.len(), 2);
        assert_eq!(loaded.versions[0].id, "v0.1.1-rc.2");
        assert_eq!(loaded.versions[0].kind, VersionKind::Npm);
        assert_eq!(
            loaded.versions[0].spec.as_deref(),
            Some("@deepseek-ai/dsh@0.1.1-rc.2")
        );
        assert_eq!(loaded.versions[1].kind, VersionKind::Dev);
        assert_eq!(loaded.versions[1].cwd.as_deref(), Some("/tmp/repo"));
        assert_eq!(loaded.homes.len(), 1);
        assert_eq!(loaded.homes[0].bound_version_id.as_deref(), Some("v0.1.1-rc.2"));
        assert_eq!(loaded.history.len(), 1);
        assert_eq!(loaded.history[0].exit_code, Some(0));
    }

    #[test]
    fn save_is_atomic_no_tmp_left_and_creates_dirs() {
        let p = temp_registry_path("atomic");
        save(&p, &Registry::default()).expect("save ok");
        assert!(p.is_file(), "registry.json 应存在");
        let parent = p.parent().unwrap();
        let leftovers: Vec<String> = std::fs::read_dir(parent)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .filter(|n| n.ends_with(".tmp"))
            .collect();
        assert!(leftovers.is_empty(), "不应残留临时文件: {leftovers:?}");
    }

    #[test]
    fn load_corrupt_file_is_error() {
        let p = temp_registry_path("corrupt");
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(&p, "{ not json").unwrap();
        assert!(load(&p).is_err(), "坏 JSON 必须报错而不是静默清空注册表");
    }

    #[test]
    fn upsert_inserts_then_replaces_same_id() {
        let mut reg = Registry::default();
        reg.upsert_version(sample("v1", VersionKind::Npm)).unwrap();
        assert_eq!(reg.versions.len(), 1);
        let mut fp = sample("v1", VersionKind::Npm);
        fp.fingerprint = Some("0.1.1-rc.2".into());
        reg.upsert_version(fp).unwrap();
        assert_eq!(reg.versions.len(), 1, "同 id 应替换而非新增");
        assert_eq!(reg.versions[0].fingerprint.as_deref(), Some("0.1.1-rc.2"));
    }

    #[test]
    fn remove_version_only_removes_target() {
        let mut reg = Registry::default();
        reg.upsert_version(sample("a", VersionKind::Npm)).unwrap();
        reg.upsert_version(sample("b", VersionKind::Dev)).unwrap();
        assert!(reg.remove_version("a"));
        assert!(!reg.remove_version("a"), "二次删除应返回 false");
        assert_eq!(reg.versions.len(), 1);
        assert_eq!(reg.versions[0].id, "b");
    }

    #[test]
    fn fresh_id_avoids_collision() {
        let mut reg = Registry::default();
        assert_eq!(reg.fresh_id("dev-x"), "dev-x");
        reg.upsert_version(sample("dev-x", VersionKind::Dev)).unwrap();
        assert_eq!(reg.fresh_id("dev-x"), "dev-x-2");
    }

    #[test]
    fn validate_id_rejects_traversal_and_blank() {
        assert!(validate_id("").is_err());
        assert!(validate_id(".hidden").is_err());
        assert!(validate_id("../etc").is_err());
        assert!(validate_id("a/b").is_err());
        assert!(validate_id("v0.1.1-rc.2").is_ok());
        assert!(validate_id("dev-deepseek-harness").is_ok());
    }
}
