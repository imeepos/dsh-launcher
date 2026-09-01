//! 系统类检查项(sys_os / sys_arch / disk_space):从 envcheck 拆出以守文件规模。

use crate::envcheck::{CheckItem, EnvSnapshot, Level, Status};

pub(crate) fn check_os(s: &EnvSnapshot) -> CheckItem {
    match s.os.as_str() {
        "macos" => check_macos_version(s),
        "linux" => check_linux_glibc(s),
        _ => CheckItem::new("sys_os", Level::Blocker, Status::Fail, "无法识别的操作系统"),
    }
}

fn check_macos_version(s: &EnvSnapshot) -> CheckItem {
    let major: Option<u32> = s.os_version.split('.').next().and_then(|m| m.parse().ok());
    match major {
        Some(v) if v >= 12 => CheckItem::new(
            "sys_os",
            Level::Blocker,
            Status::Pass,
            format!("macOS {}", s.os_version),
        ),
        Some(v) => CheckItem::new(
            "sys_os",
            Level::Blocker,
            Status::Fail,
            format!("macOS {v} 过老,需要 12 及以上"),
        ),
        None => CheckItem::new(
            "sys_os",
            Level::Blocker,
            Status::Skip,
            "无法读取 macOS 版本",
        ),
    }
}

fn check_linux_glibc(s: &EnvSnapshot) -> CheckItem {
    let parsed = s.glibc.as_deref().and_then(parse_glibc);
    match parsed {
        Some(v) if v >= 217 => CheckItem::new(
            "sys_os",
            Level::Blocker,
            Status::Pass,
            format!("glibc {}", s.glibc.clone().unwrap_or_default()),
        ),
        Some(_) => CheckItem::new(
            "sys_os",
            Level::Blocker,
            Status::Fail,
            format!(
                "glibc {} 过老,需要 2.17+",
                s.glibc.clone().unwrap_or_default()
            ),
        ),
        None => CheckItem::new(
            "sys_os",
            Level::Blocker,
            Status::Skip,
            "无法读取 glibc 版本(musl?)",
        ),
    }
}

/// "2.39" -> 239,便于整数比较。
fn parse_glibc(v: &str) -> Option<u32> {
    let (a, b) = v.trim().split_once('.')?;
    Some(a.parse::<u32>().ok()? * 100 + b.parse::<u32>().ok()?)
}

pub(crate) fn check_arch(s: &EnvSnapshot) -> CheckItem {
    if matches!(s.arch.as_str(), "arm64" | "x86_64") {
        CheckItem::new("sys_arch", Level::Blocker, Status::Pass, s.arch.clone())
    } else {
        CheckItem::new(
            "sys_arch",
            Level::Blocker,
            Status::Fail,
            format!("不支持的架构 {}", s.arch),
        )
    }
}

pub(crate) fn check_disk(s: &EnvSnapshot) -> CheckItem {
    match s.disk_free_mb {
        Some(mb) if mb >= 1024 => CheckItem::new(
            "disk_space",
            Level::Blocker,
            Status::Pass,
            format!("可用 {mb} MB"),
        ),
        Some(mb) => CheckItem::new(
            "disk_space",
            Level::Blocker,
            Status::Fail,
            format!("可用 {mb} MB,至少需要 1 GB"),
        ),
        None => CheckItem::new(
            "disk_space",
            Level::Blocker,
            Status::Skip,
            "无法读取磁盘剩余空间",
        ),
    }
}
