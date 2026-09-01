//! 集成冒烟测试:依赖真实环境(dev repo、公网 npm),默认 ignore。
//! 运行:cargo test --test smoke -- --ignored --nocapture

use dsh_launcher_lib::launcher;
use dsh_launcher_lib::runtime_install;

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

/// 真机安装 managed runtime 到临时目录:双源下载+sha256+解压+runtime.json
#[test]
#[ignore]
fn smoke_runtime_install() {
    let base = std::env::temp_dir().join(format!("dsh-rt-smoke-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let info = runtime_install::install_runtime_into(&base, |line| println!("[rt] {line}"))
        .expect("运行时安装失败");
    println!("node = {}", info.bin.join("node").display());
    assert!(info.bin.join("node").is_file(), "node 应存在");
    let ver = std::process::Command::new(info.bin.join("node"))
        .arg("--version")
        .output()
        .expect("运行 node 失败");
    assert!(ver.status.success(), "node --version 应成功");
    println!(
        "node --version = {}",
        String::from_utf8_lossy(&ver.stdout).trim()
    );
    let saved = runtime_install::read_runtime_from(&base);
    assert!(saved.is_some(), "runtime.json 应已落盘");
    let _ = std::fs::remove_dir_all(&base);
}
