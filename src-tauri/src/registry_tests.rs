#[cfg(test)]
mod tests {
    use crate::registry::{
        load, sanitize_id_fragment, save, validate_id, HistoryEntry, HomeEntry, Registry,
        VersionEntry, VersionKind,
    };
    use std::fs;
    use std::path::PathBuf;

    fn temp_registry_path(tag: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("dsh-launcher-test-{}-{tag}", std::process::id()));
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
            tool: None,
        }
    }

    #[test]
    fn tool_field_backward_compatible_and_fallback() {
        // 旧文件无 tool 字段:反序列化为 None,展示回落 dsh(DESIGN-TOOLS.md §1)
        let legacy = r#"{"id":"v1","kind":"manual","bin":"/usr/local/bin/releasectl"}"#;
        let entry: VersionEntry = serde_json::from_str(legacy).expect("legacy json ok");
        assert!(entry.tool.is_none());
        assert_eq!(entry.effective_tool(), "dsh");

        let mut reg = Registry::default();
        let mut with_tool = sample("t1", VersionKind::Manual);
        with_tool.tool = Some("releasectl".into());
        reg.upsert_version(with_tool).expect("upsert ok");
        assert_eq!(reg.versions[0].effective_tool(), "releasectl");

        // 空/空白 tool 也回落 dsh
        let mut blank = sample("t2", VersionKind::Manual);
        blank.tool = Some("  ".into());
        assert_eq!(blank.effective_tool(), "dsh");
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
        assert_eq!(
            loaded.homes[0].bound_version_id.as_deref(),
            Some("v0.1.1-rc.2")
        );
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
        reg.upsert_version(sample("dev-x", VersionKind::Dev))
            .unwrap();
        assert_eq!(reg.fresh_id("dev-x"), "dev-x-2");
    }

    #[test]
    fn sanitize_id_fragment_folds_illegal_chars() {
        assert_eq!(sanitize_id_fragment("deepseek-harness"), "deepseek-harness");
        assert_eq!(sanitize_id_fragment("My Repo/v2"), "my-repo-v2");
        assert_eq!(sanitize_id_fragment("   "), "x");
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

    #[test]
    fn old_registry_without_settings_loads_with_defaults() {
        let path = temp_registry_path("no-settings");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, r#"{"versions":[],"homes":[],"history":[]}"#).unwrap();
        let reg = load(&path).expect("老格式应可加载");
        assert!(!reg.settings.use_system_node);
        assert!(reg.settings.npm_registry.is_none());
        assert!(reg.settings.node_dist_mirror.is_none());
    }
}
