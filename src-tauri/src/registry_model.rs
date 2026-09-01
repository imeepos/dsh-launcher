use serde::{Deserialize, Serialize};
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
