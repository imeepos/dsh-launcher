//! 进程运行时测试:假 dsh 脚本,验证发现/启动/锁/SIGTERM 全链路。

#[cfg(test)]
mod tests {
    use crate::launcher_process as proc;
    use std::fs;
    use std::io::Write;
    use std::path::{Path, PathBuf};
    use std::time::{Duration, Instant};

    fn temp_home(tag: &str) -> PathBuf {
        let d =
            std::env::temp_dir().join(format!("dsh-launcher-procrt-{}-{tag}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(d.join("profiles/default")).unwrap();
        d
    }

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

    const FAKE_DSH: &str =
        // trap 必须先于 ready 行安装,否则 SIGTERM 早到会直接杀掉脚本
        "#!/bin/sh\ntrap 'exit 0' TERM\necho fake-dsh-started\nwhile :; do sleep 0.1; done\n";

    #[test]
    fn list_profiles_discovers_and_counts_bundles() {
        let home = temp_home("discover");
        let pkg = home.join("profiles/default/package.json");
        fs::write(&pkg, r#"{"dsh":{"profile":{"bundles":["a","b"]}}}"#).unwrap();
        fs::create_dir_all(home.join("profiles/no-pkg")).unwrap();
        let found = proc::list_profiles(home.to_str().unwrap()).unwrap();
        assert_eq!(found.len(), 1, "无 package.json 的目录应跳过");
        assert_eq!(found[0].name, "default");
        assert_eq!(found[0].bundle_count, 2);
        let empty = proc::list_profiles("/nonexistent-home-xyz").unwrap();
        assert!(empty.is_empty(), "无 profiles 目录应返回空");
        let _ = fs::remove_dir_all(&home);
    }

    #[test]
    fn build_start_command_orders_args_and_sets_home() {
        let home = PathBuf::from("/tmp/dsh-home-x");
        let cmd = proc::build_start_command(
            "pnpm dsh",
            &home,
            "default",
            Some("p.yml"),
            &["chat".to_string()],
            None,
        )
        .unwrap();
        let args: Vec<String> = cmd
            .get_args()
            .map(|s| s.to_string_lossy().into_owned())
            .collect();
        assert_eq!(
            args,
            vec![
                "dsh",
                "--profile",
                "default",
                "--patch",
                "p.yml",
                "--",
                "chat"
            ]
        );
        let home_env = cmd.get_envs().find(|(k, _)| *k == "DSH_HOME").unwrap();
        assert_eq!(home_env.1, Some(home.as_os_str()));
    }

    #[test]
    fn run_lock_is_exclusive() {
        let home = temp_home("lock");
        let _first = proc::acquire_run_lock(&home, "default").unwrap();
        let second = proc::acquire_run_lock(&home, "default");
        assert!(second.is_err(), "同 (home,profile) 二次加锁必须失败");
        let _ = fs::remove_dir_all(&home);
    }

    #[test]
    fn start_stop_fake_dsh_exits_zero_and_releases_lock() {
        let home = temp_home("e2e");
        let bin = write_exec(&home.join("bin"), "fake-dsh", FAKE_DSH);
        let mut run = proc::start_dsh(
            bin.to_str().unwrap(),
            "main",
            &home,
            "default",
            "v-test",
            None,
            &[],
            None,
        )
        .unwrap();
        let (rx, _t_out, _t_err) = {
            let mut g = run.child.lock().unwrap();
            proc::spawn_pipe_forwarders(&mut g)
        };
        // 等 ready 行:确保脚本已装好 trap,再发 SIGTERM
        let mut started = false;
        while !started {
            match rx.recv_timeout(Duration::from_secs(5)) {
                Ok((false, line)) if line.contains("fake-dsh-started") => started = true,
                Ok(_) => {}
                Err(_) => panic!("假 dsh 未输出 ready 行"),
            }
        }
        proc::stop_dsh(&run.child).unwrap();
        let deadline = Instant::now() + Duration::from_secs(10);
        let status = loop {
            if let Some(s) = proc::try_wait(&run.child) {
                break s;
            }
            assert!(Instant::now() < deadline, "假 dsh 未在 10s 内退出");
            std::thread::sleep(Duration::from_millis(50));
        };
        assert_eq!(status.code(), Some(0), "SIGTERM 后 dsh 约定 exit 0");
        drop(run);
        let again = proc::acquire_run_lock(&home, "default");
        assert!(again.is_ok(), "进程退出并 drop 后锁应释放");
        let _ = fs::remove_dir_all(&home);
    }
}
