//! 集成冒烟测试:依赖真实环境(dev repo、公网 npm),默认 ignore。
//! 运行:cargo test --test smoke -- --ignored --nocapture

use dsh_launcher_lib::launcher;

/// 对 dev repo 取非空指纹(pnpm dsh --version,cwd=repoPath)
#[test]
#[ignore]
fn smoke_dev_repo_fingerprint() {
    let repo = "/Users/imeepos/ext512/ymm-001/deepseek-harness";
    let fp = launcher::fingerprint(launcher::DEV_BIN, Some(repo)).expect("dev 指纹采集失败");
    println!("dev fingerprint = {fp}");
    assert!(!fp.trim().is_empty(), "指纹不能为空");
}

/// npm 安装 0.1.1-rc.2 并取非空指纹(公网),结束后清理冒烟目录
#[test]
#[ignore]
fn smoke_npm_install_and_fingerprint() {
    let spec = "@deepseek-ai/dsh@0.1.1-rc.2";
    let id = "smoke-v0.1.1-rc.2";
    let bin =
        launcher::install_npm(spec, id, |line| println!("[npm] {line}")).expect("npm 安装失败");
    println!("bin = {}", bin.display());
    assert!(bin.is_file(), "node_modules/.bin/dsh 应存在");
    let fp = launcher::fingerprint(&bin.to_string_lossy(), None).expect("npm 指纹采集失败");
    println!("npm fingerprint = {fp}");
    assert!(!fp.trim().is_empty(), "指纹不能为空");
    launcher::remove_version_dir(id).expect("清理冒烟目录失败");
}
