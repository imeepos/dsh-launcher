use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use super::registry_model::{RegResult, Registry};

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

/// 从任意字符串(如目录名)生成 id 安全片段:非法字符折叠为 '-',小写化。
pub fn sanitize_id_fragment(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        if c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_' | '+') {
            out.push(c);
        } else if !out.ends_with('-') && !out.is_empty() {
            out.push('-');
        }
    }
    let t = out.trim_matches('-').to_lowercase();
    if t.is_empty() {
        "x".into()
    } else {
        t
    }
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
        fs::create_dir_all(parent).map_err(|e| format!("创建 {} 失败: {e}", parent.display()))?;
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
