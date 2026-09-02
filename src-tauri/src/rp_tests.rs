//! rp 模块纯函数测试(不发网络):URL/信封/平台命名映射/文件名提取。

#[cfg(test)]
mod tests {
    use crate::rp::{envelope_items, filename_from_url, local_arch, local_os, query_url, urlencode};
    use serde_json::json;

    #[test]
    fn urlencode_escapes_unsafe_bytes() {
        assert_eq!(urlencode("t1"), "t1");
        assert_eq!(urlencode("a b"), "a%20b");
        assert_eq!(urlencode("m&m=1"), "m%26m%3D1");
        assert_eq!(urlencode("管理员"), "%E7%AE%A1%E7%90%86%E5%91%98");
    }

    #[test]
    fn query_url_joins_and_skips_empty() {
        let url = query_url(
            "http://host:38080/",
            "/v1/artifacts",
            &[("version_id", "v1"), ("os", ""), ("arch", "arm64")],
        );
        assert_eq!(url, "http://host:38080/v1/artifacts?version_id=v1&arch=arm64");
    }

    #[test]
    fn local_platform_maps_to_artifact_naming() {
        // macOS 本机名映射为平台 darwin/arm64(DESIGN-TOOLS.md §2.1)
        if std::env::consts::OS == "macos" {
            assert_eq!(local_os(), "darwin");
        }
        if std::env::consts::ARCH == "aarch64" {
            assert_eq!(local_arch(), "arm64");
        }
        if std::env::consts::ARCH == "x86_64" {
            assert_eq!(local_arch(), "amd64");
        }
    }

    #[test]
    fn envelope_items_handles_both_shapes() {
        let enveloped = json!({ "items": [1, 2], "total": 2 });
        assert_eq!(envelope_items(&enveloped).len(), 2);
        let bare = json!(["a", "b", "c"]);
        assert_eq!(envelope_items(&bare).len(), 3);
        assert!(envelope_items(&json!({ "error": "x" })).is_empty());
    }

    #[test]
    fn filename_from_url_strips_query_and_falls_back() {
        assert_eq!(
            filename_from_url("http://s3/bucket/tool-1.2.3-darwin-arm64.tar.gz?sig=ab", "x"),
            "tool-1.2.3-darwin-arm64.tar.gz"
        );
        assert_eq!(filename_from_url("http://s3/", "fallback.bin"), "fallback.bin");
    }
}
