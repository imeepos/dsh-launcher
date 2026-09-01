//! Node 发行版双源 URL 构建与清单解析:纯函数,便于单测。

/// 双源:官方与国内镜像(SHASUMS256 两源一致,已实测验证)。
pub const SOURCES: [(&str, &str); 2] = [
    ("official", "https://nodejs.org/dist"),
    ("npmmirror", "https://cdn.npmmirror.com/binaries/node"),
];

/// 安装器兜底版本:版本索引双源都不可达时使用(已验证双源存在)。
pub const FALLBACK_VERSION: &str = "v22.14.0";

pub fn tarball_name(version: &str, os: &str, arch: &str) -> String {
    format!("node-{version}-{os}-{arch}.tar.xz")
}

pub fn shasums_url(source_base: &str, version: &str) -> String {
    format!("{source_base}/{version}/SHASUMS256.txt")
}

pub fn tarball_url(source_base: &str, version: &str, name: &str) -> String {
    format!("{source_base}/{version}/{name}")
}

/// 从 index.json 选最新 LTS。索引按新→旧排列,取第一个 lts 为字符串的条目。
pub fn pick_lts_version(index_json: &str) -> Option<String> {
    let entries = serde_json::from_str::<serde_json::Value>(index_json).ok()?;
    entries
        .as_array()?
        .iter()
        .find(|e| matches!(e.get("lts"), Some(serde_json::Value::String(_))))
        .and_then(|e| e.get("version")?.as_str().map(String::from))
}

/// 从 SHASUMS256.txt 取目标文件的期望哈希(小写化)。
pub fn expected_sha256(shasums: &str, filename: &str) -> Option<String> {
    shasums.lines().find_map(|line| {
        let (hash, name) = line.trim().split_once(char::is_whitespace)?;
        (name.trim_start() == filename).then(|| hash.to_lowercase())
    })
}
