#[cfg(test)]
mod tests {
    use crate::launcher::{
        expand_tilde, fingerprint, home_dir, install_npm, remove_version_dir, validate_dev_repo,
        DEV_BIN,
    };
    use crate::registry::sanitize_id_fragment;
    use std::path::PathBuf;

    #[test]
    fn expand_tilde_handles_home_forms() {
        let home = home_dir();
        assert_eq!(expand_tilde("~"), home);
        assert_eq!(expand_tilde("~/foo/bar"), home.join("foo/bar"));
        assert_eq!(expand_tilde("/abs/path"), PathBuf::from("/abs/path"));
        assert_eq!(expand_tilde("  ~/x  "), home.join("x"));
    }

    #[test]
    fn fingerprint_direct_exec() {
        // echo 直接 exec:输出恰为参数本身
        assert_eq!(fingerprint("echo", None).unwrap(), "--version");
    }

    #[test]
    fn fingerprint_shell_for_spaced_bin() {
        // 含空白的 bin 走 sh -c(dev kind 的 "pnpm dsh" 同理)
        assert_eq!(
            fingerprint("printf hello-from-shell", None).unwrap(),
            "hello-from-shell"
        );
    }

    #[test]
    fn fingerprint_missing_bin_is_error() {
        assert!(fingerprint("/nonexistent/dsh-binary-xyz", None).is_err());
    }

    #[test]
    fn fingerprint_nonzero_exit_is_error() {
        assert!(fingerprint("sh -c 'echo boom 1>&2; exit 3'", None).is_err());
    }

    #[test]
    fn fingerprint_empty_stdout_is_error() {
        assert!(fingerprint("sh -c 'exit 0'", None).is_err());
    }

    #[test]
    fn install_npm_rejects_bad_id_fast() {
        // 非法 id 直接报错,不会真的去跑 npm
        let mut called = false;
        let r = install_npm("@deepseek-ai/dsh@0.1.1-rc.2", "../evil", |_| {
            called = true;
        });
        assert!(r.is_err());
        assert!(!called);
    }

    #[test]
    fn remove_version_dir_rejects_traversal() {
        assert!(remove_version_dir("../../etc").is_err());
        assert!(remove_version_dir("").is_err());
    }

    #[test]
    fn validate_dev_repo_checks_layout() {
        assert!(validate_dev_repo("/nonexistent-repo-xyz").is_err());
        // /tmp 存在但没有 package.json
        assert!(validate_dev_repo("/tmp").is_err());
    }

    #[test]
    fn dev_bin_is_pnpm_dsh() {
        assert_eq!(DEV_BIN, "pnpm dsh");
        assert_eq!(sanitize_id_fragment("deepseek-harness"), "deepseek-harness");
    }
}
