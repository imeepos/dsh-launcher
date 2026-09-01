//! E2E:临时 HOME 下走完「预检红 → 装运行时 → 预检绿 → 向导全步进 → 完成」真实链路。
//! 依赖公网下载(约 30 秒),默认 ignore。
//! 运行:cargo test --test e2e_onboarding -- --ignored --nocapture

use dsh_launcher_lib::envcheck;
use dsh_launcher_lib::envcheck_probe;
use dsh_launcher_lib::onboarding;
use dsh_launcher_lib::registry::OnboardingStep;
use dsh_launcher_lib::runtime_install;

#[test]
#[ignore]
fn e2e_fresh_home_onboarding_chain() {
    let home = std::env::temp_dir().join(format!("dsh-e2e-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&home);
    std::env::set_var("HOME", &home);

    // 1. 全新环境快速预检:runtime 应为 fail
    let snap = envcheck_probe::collect_snapshot();
    let pre = envcheck::run_fast_preflight(&snap);
    assert!(
        pre.iter()
            .any(|i| i.id == "runtime" && i.status == envcheck::Status::Fail),
        "全新环境预检应报 runtime 缺失",
    );

    // 2. 安装运行时(双源真实下载 + sha256 校验 + 解压)
    let info = runtime_install::install_runtime(|l| println!("[rt] {l}")).expect("运行时安装失败");
    assert!(info.bin.join("node").is_file(), "node 应存在");

    // 3. 预检转绿
    let snap2 = envcheck_probe::collect_snapshot();
    let pre2 = envcheck::run_fast_preflight(&snap2);
    assert!(
        pre2.iter().all(|i| i.status == envcheck::Status::Pass),
        "装完运行时后预检应全过",
    );

    // 4. 向导全链步进(跳过 fix)→ 完成 → 状态落盘可续
    for step in ["check", "mode", "install", "home", "launch"] {
        let s = onboarding::parse_step(step).unwrap();
        onboarding::advance(s).expect("步进失败");
    }
    onboarding::complete().expect("完成失败");
    let saved = onboarding::load();
    assert!(saved.completed);
    assert_eq!(saved.step, OnboardingStep::Done);

    let _ = std::fs::remove_dir_all(&home);
}
