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

/// 萌新首跑设置(DESIGN-ONBOARDING.md §6);全部字段可缺省,老文件零迁移。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    /// npm registry 覆盖;None 用 npm 默认源
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub npm_registry: Option<String>,
    /// node 发行版镜像;None 双源自动择优
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node_dist_mirror: Option<String>,
    /// 高级:改用系统 Node(默认 false,优先自带运行时)
    #[serde(default)]
    pub use_system_node: bool,
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
    /// O1 起启用:萌新首跑设置
    #[serde(default)]
    pub settings: Settings,
    /// O2 起启用:首跑向导断点状态
    #[serde(default)]
    pub onboarding: OnboardingState,
}

/// 首跑向导步骤:welcome → check → fix* → mode → install → home → launch → done。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OnboardingStep {
    #[default]
    Welcome,
    Check,
    Fix,
    Mode,
    Install,
    Home,
    Launch,
    Done,
}

/// 首跑断点状态;老文件缺省 = 未开始向导。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OnboardingState {
    #[serde(default)]
    pub step: OnboardingStep,
    #[serde(default)]
    pub completed: bool,
}

impl Default for OnboardingState {
    fn default() -> Self {
        Self {
            step: OnboardingStep::Welcome,
            completed: false,
        }
    }
}
