//! 环境检查(设计文档 §2):首跑全量清单与日常快速预检。
//! 检查逻辑纯函数化:输入 EnvSnapshot + NetStatus,输出 CheckItem 清单;
//! 首跑向导与修复中心共用同一套判定。

use crate::envcheck_sys::{check_arch, check_disk, check_os};
use std::path::PathBuf;

/// 检查项级别:blocker 档住向导推进;warn 建议修;info 仅提示。
#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Blocker,
    Warn,
    Info,
}

/// 单项结论:Skip 表示本机无法判定,不当失败处理(如 musl 无 ldd)。
#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Pass,
    Fail,
    Skip,
}

/// 一条检查结果;id 与设计文档 §2 表一致。
#[derive(Clone, Debug, serde::Serialize)]
pub struct CheckItem {
    pub id: &'static str,
    pub level: Level,
    pub status: Status,
    pub detail: String,
}

impl CheckItem {
    pub(crate) fn new(
        id: &'static str,
        level: Level,
        status: Status,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            id,
            level,
            status,
            detail: detail.into(),
        }
    }
}

/// 系统快照:由 envcheck_probe 采集;测试可直接手工构造。
#[derive(Clone, Debug, serde::Serialize)]
pub struct EnvSnapshot {
    pub os: String,                // "macos" | "linux" | 其他
    pub os_version: String,        // "14.5";取不到为空串
    pub arch: String,              // "arm64" | "x86_64" | 其他
    pub glibc: Option<String>,     // 仅 linux;"2.39";取不到为 None
    pub disk_free_mb: Option<u64>, // base_dir 所在卷可用空间
    pub base_dir: PathBuf,
    pub base_dir_writable: bool,
    pub runtime_bin: Option<PathBuf>, // managed runtime 的 bin 目录
    pub node_version_ok: bool,        // runtime node --version 成功
    pub existing_dsh: Option<String>, // 检测到已有 dsh 的来源描述
}

impl Default for EnvSnapshot {
    fn default() -> Self {
        Self {
            os: "unknown".into(),
            os_version: String::new(),
            arch: "unknown".into(),
            glibc: None,
            disk_free_mb: None,
            base_dir: PathBuf::new(),
            base_dir_writable: false,
            runtime_bin: None,
            node_version_ok: false,
            existing_dsh: None,
        }
    }
}

/// 网络可达性:探测官方与镜像,双源任一可达即算过。
#[derive(Clone, Copy, Debug, Default)]
pub struct NetStatus {
    pub registry_ok: bool,
    pub node_dist_ok: bool,
}

/// 首跑全量检查,顺序即展示顺序。
pub fn run_checks(s: &EnvSnapshot, net: &NetStatus) -> Vec<CheckItem> {
    vec![
        check_os(s),
        check_arch(s),
        check_disk(s),
        check_base_dir(s),
        check_net_registry(net),
        check_net_node(net),
        check_runtime(s),
        check_existing_dsh(s),
    ]
}

/// 日常快速预检:只看本机 O(1) 项,网络探测不进日常路径。
pub fn run_fast_preflight(s: &EnvSnapshot) -> Vec<CheckItem> {
    vec![check_runtime(s), check_base_dir(s)]
}

/// 没有 fail 的 blocker 才算放行(Skip 不挡路,如 musl 无法判 glibc)。
pub fn blockers_cleared(items: &[CheckItem]) -> bool {
    !items
        .iter()
        .any(|i| i.level == Level::Blocker && i.status == Status::Fail)
}

fn check_base_dir(s: &EnvSnapshot) -> CheckItem {
    if s.base_dir_writable {
        CheckItem::new(
            "base_dir",
            Level::Blocker,
            Status::Pass,
            format!("{}", s.base_dir.display()),
        )
    } else {
        CheckItem::new(
            "base_dir",
            Level::Blocker,
            Status::Fail,
            format!("{} 不可写", s.base_dir.display()),
        )
    }
}

fn check_net_registry(net: &NetStatus) -> CheckItem {
    if net.registry_ok {
        CheckItem::new("net_registry", Level::Blocker, Status::Pass, "npm 源可达")
    } else {
        CheckItem::new(
            "net_registry",
            Level::Blocker,
            Status::Fail,
            "npm 源不可达,可切换镜像",
        )
    }
}

fn check_net_node(net: &NetStatus) -> CheckItem {
    if net.node_dist_ok {
        CheckItem::new(
            "net_node_dist",
            Level::Blocker,
            Status::Pass,
            "Node 发行版源可达",
        )
    } else {
        CheckItem::new(
            "net_node_dist",
            Level::Blocker,
            Status::Fail,
            "Node 发行版源不可达,可切换镜像",
        )
    }
}

fn check_runtime(s: &EnvSnapshot) -> CheckItem {
    match (&s.runtime_bin, s.node_version_ok) {
        (Some(bin), true) => CheckItem::new(
            "runtime",
            Level::Blocker,
            Status::Pass,
            format!("{}", bin.display()),
        ),
        (Some(_), false) => CheckItem::new(
            "runtime",
            Level::Blocker,
            Status::Fail,
            "运行时损坏,node --version 失败",
        ),
        (None, _) => CheckItem::new("runtime", Level::Blocker, Status::Fail, "尚未安装运行环境"),
    }
}

fn check_existing_dsh(s: &EnvSnapshot) -> CheckItem {
    match &s.existing_dsh {
        None => CheckItem::new(
            "existing_dsh",
            Level::Info,
            Status::Pass,
            "未检测到已有 dsh,全新环境",
        ),
        Some(src) => CheckItem::new(
            "existing_dsh",
            Level::Info,
            Status::Pass,
            format!("检测到已有 dsh:{src}"),
        ),
    }
}
