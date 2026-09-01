//! runtime_install 纯函数单测:不触网络与磁盘。

#[cfg(test)]
mod tests {
    use crate::runtime_install_core::{
        expected_sha256, pick_lts_version, shasums_url, tarball_name, tarball_url,
    };

    const INDEX: &str = r#"
      [
        {"version":"v24.0.0","lts":false},
        {"version":"v22.14.0","lts":"Jod"},
        {"version":"v20.19.0","lts":"Iron"}
      ]
    "#;

    #[test]
    fn picks_newest_lts_not_absolute_newest() {
        assert_eq!(pick_lts_version(INDEX).as_deref(), Some("v22.14.0"));
    }

    #[test]
    fn no_lts_yields_none() {
        assert_eq!(
            pick_lts_version(r#"[{"version":"v24.0.0","lts":false}]"#),
            None
        );
    }

    #[test]
    fn malformed_index_yields_none() {
        assert_eq!(pick_lts_version("not json"), None);
    }

    #[test]
    fn url_builders_compose() {
        let base = "https://nodejs.org/dist";
        assert_eq!(
            shasums_url(base, "v22.14.0"),
            "https://nodejs.org/dist/v22.14.0/SHASUMS256.txt",
        );
        assert_eq!(
            tarball_url(
                base,
                "v22.14.0",
                &tarball_name("v22.14.0", "darwin", "arm64")
            ),
            "https://nodejs.org/dist/v22.14.0/node-v22.14.0-darwin-arm64.tar.xz",
        );
    }

    #[test]
    fn shasums_parsing_matches_two_space_format() {
        let text = concat!(
            "4e845cb7..  node-v22.14.0-darwin-arm64.tar.xz\n",
            "69b09dba..  node-v22.14.0-linux-x64.tar.xz\n",
        );
        assert_eq!(
            expected_sha256(text, "node-v22.14.0-darwin-arm64.tar.xz").as_deref(),
            Some("4e845cb7..")
        );
        assert_eq!(expected_sha256(text, "missing.tar.xz"), None);
    }
}
