//! 子进程超时与流式路径的测试:假 npm/假慢进程,不碰网络。

#[cfg(test)]
mod tests {
    use crate::launcher::{fingerprint_with_timeout, install_npm_into};
    use std::fs;
    use std::io::Write;
    use std::path::{Path, PathBuf};
    use std::time::{Duration, Instant};

    fn write_exec(dir: &Path, name: &str, body: &str) -> PathBuf {
        fs::create_dir_all(dir).unwrap();
        let p = dir.join(name);
        let mut f = fs::File::create(&p).unwrap();
        f.write_all(body.as_bytes()).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&p, fs::Permissions::from_mode(0o755)).unwrap();
        }
        p
    }

    fn temp_root(tag: &str) -> PathBuf {
        let d =
            std::env::temp_dir().join(format!("dsh-launcher-proc-{}-{tag}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn fingerprint_timeout_kills_slow_process() {
        let t = Instant::now();
        let r = fingerprint_with_timeout(
            "sh -c 'sleep 30; echo done'",
            None,
            Duration::from_millis(400),
        );
        assert!(r.is_err());
        assert!(r.unwrap_err().contains("超时"));
        assert!(
            t.elapsed() < Duration::from_secs(10),
            "应在超时后及时返回,而不是干等 30s"
        );
    }

    #[test]
    fn fingerprint_collects_output_after_pipe_close() {
        // 进程先退出、管道后关闭:覆盖 Disconnected → 补 wait 采集输出/退出码的路径
        let r = fingerprint_with_timeout(
            "sh -c 'sleep 0.3; printf late-line'",
            None,
            Duration::from_secs(10),
        );
        assert_eq!(r.unwrap(), "late-line");
    }

    #[test]
    fn install_npm_into_timeout_kills_fake_npm() {
        let root = temp_root("timeout");
        let npm = write_exec(
            &root.join("bin"),
            "npm",
            "#!/bin/sh\necho 'fake npm start'\nexec sleep 30\n",
        );
        let r = install_npm_into(
            &root.join("v"),
            &npm,
            "@x/y@1",
            None,
            Duration::from_millis(400),
            |_| {},
        );
        assert!(r.unwrap_err().contains("超时"));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn install_npm_into_reports_failure() {
        let root = temp_root("failure");
        let npm = write_exec(
            &root.join("bin"),
            "npm",
            "#!/bin/sh\necho boom >&2\nexit 1\n",
        );
        let r = install_npm_into(
            &root.join("v"),
            &npm,
            "@x/y@1",
            None,
            Duration::from_secs(30),
            |_| {},
        );
        let err = r.unwrap_err();
        assert!(err.contains("失败"), "应报安装失败,得到: {err}");
        assert!(err.contains("boom"), "错误里应带 stderr 内容");
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn install_npm_into_success_and_retry_cleans_dir() {
        let root = temp_root("retry");
        let dir = root.join("v");
        let npm1 = write_exec(
            &root.join("bin"),
            "npm",
            "#!/bin/sh\ncd \"$3\" || exit 1\nmkdir -p node_modules/.bin\nprintf '#!/bin/sh\necho dsh\n' > node_modules/.bin/dsh\nchmod +x node_modules/.bin/dsh\nprintf stale > marker.txt\n",
        );
        let bin =
            install_npm_into(&dir, &npm1, "@x/y@1", None, Duration::from_secs(30), |_| {}).unwrap();
        assert!(bin.is_file(), "node_modules/.bin/dsh 应存在");
        assert!(dir.join("marker.txt").is_file());

        // 重试:换一个不留 marker 的假 npm,旧目录应被整个清掉
        let npm2 = write_exec(
            &root.join("bin2"),
            "npm",
            "#!/bin/sh\ncd \"$3\" || exit 1\nmkdir -p node_modules/.bin\nprintf '#!/bin/sh\necho dsh\n' > node_modules/.bin/dsh\nchmod +x node_modules/.bin/dsh\n",
        );
        let bin2 =
            install_npm_into(&dir, &npm2, "@x/y@1", None, Duration::from_secs(30), |_| {}).unwrap();
        assert!(bin2.is_file());
        assert!(
            !dir.join("marker.txt").is_file(),
            "重试应清掉旧目录的残留文件"
        );
        let _ = fs::remove_dir_all(&root);
    }
}
