//! envcheck 纯逻辑单测:手工构造快照,不碰真实系统与网络。

#[cfg(test)]
mod tests {
    use crate::envcheck::{
        blockers_cleared, run_checks, run_fast_preflight, CheckItem, EnvSnapshot, Level, NetStatus,
        Status,
    };

    fn snap_good() -> EnvSnapshot {
        EnvSnapshot {
            os: "macos".into(),
            os_version: "14.5".into(),
            arch: "arm64".into(),
            disk_free_mb: Some(5000),
            base_dir_writable: true,
            runtime_bin: Some("/opt/runtime/bin".into()),
            node_version_ok: true,
            ..Default::default()
        }
    }

    fn net_good() -> NetStatus {
        NetStatus {
            registry_ok: true,
            node_dist_ok: true,
        }
    }

    fn status_of(items: &[CheckItem], id: &str) -> Status {
        items
            .iter()
            .find(|i| i.id == id)
            .map(|i| i.status)
            .unwrap_or(Status::Skip)
    }

    #[test]
    fn fresh_machine_blocks_on_runtime_and_net() {
        let items = run_checks(&EnvSnapshot::default(), &NetStatus::default());
        assert_eq!(items.len(), 8);
        assert!(!blockers_cleared(&items));
        assert_eq!(status_of(&items, "runtime"), Status::Fail);
    }

    #[test]
    fn healthy_machine_clears_all_blockers() {
        let items = run_checks(&snap_good(), &net_good());
        assert!(blockers_cleared(&items));
        let info = items.iter().find(|i| i.id == "existing_dsh").unwrap();
        assert_eq!(info.level, Level::Info);
    }

    #[test]
    fn musl_unknown_glibc_skips_without_blocking() {
        let mut s = snap_good();
        s.os = "linux".into();
        s.glibc = None;
        let items = run_checks(&s, &net_good());
        assert_eq!(status_of(&items, "sys_os"), Status::Skip);
        assert!(blockers_cleared(&items));
    }

    #[test]
    fn old_glibc_blocks() {
        let mut s = snap_good();
        s.os = "linux".into();
        s.glibc = Some("2.16".into());
        let items = run_checks(&s, &net_good());
        assert!(!blockers_cleared(&items));
    }

    #[test]
    fn old_macos_blocks() {
        let mut s = snap_good();
        s.os_version = "10.15".into();
        let items = run_checks(&s, &net_good());
        assert!(!blockers_cleared(&items));
    }

    #[test]
    fn low_disk_blocks() {
        let mut s = snap_good();
        s.disk_free_mb = Some(500);
        let items = run_checks(&s, &net_good());
        assert!(!blockers_cleared(&items));
    }

    #[test]
    fn fast_preflight_flags_missing_runtime() {
        let items = run_fast_preflight(&EnvSnapshot::default());
        assert_eq!(items.len(), 2);
        assert_eq!(status_of(&items, "runtime"), Status::Fail);
    }

    #[test]
    fn existing_dsh_reported_as_info() {
        let mut s = snap_good();
        s.existing_dsh = Some("已有 home ~/.dsh".into());
        let items = run_checks(&s, &net_good());
        let item = items.iter().find(|i| i.id == "existing_dsh").unwrap();
        assert_eq!(item.level, Level::Info);
        assert!(item.detail.contains("已有"));
    }
}
