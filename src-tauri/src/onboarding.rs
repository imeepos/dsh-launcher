//! 首跑状态机(设计文档 §3):welcome → check → fix* → mode → install → home → launch → done。
//! 状态存注册表,每步落盘,崩溃重开从断点继续;同步骤重复幂等。

use crate::commands::{peek_registry, with_registry};
use crate::registry::{OnboardingState, OnboardingStep, RegResult, Registry};

/// 当前首跑状态;注册表读不出时回退到默认值。
pub fn load() -> OnboardingState {
    peek_registry().map(|r| r.onboarding).unwrap_or_default()
}

/// 步进到 to:从当前步可达才落盘(同步骤重复允许,跨步拒绝)。
pub fn advance(to: OnboardingStep) -> RegResult<OnboardingState> {
    with_registry(|reg: &mut Registry| {
        let from = reg.onboarding.step;
        if reg.onboarding.completed {
            return Err("首跑已完成,不能再步进".into());
        }
        if to != from && !next_allowed(from).contains(&to) {
            return Err(format!("不允许从 {from:?} 步进到 {to:?}"));
        }
        reg.onboarding.step = to;
        Ok(reg.onboarding.clone())
    })
}

/// 向导完成:step 置 done 并标记 completed,以后启动不再导向导。
pub fn complete() -> RegResult<OnboardingState> {
    with_registry(|reg: &mut Registry| {
        reg.onboarding.step = OnboardingStep::Done;
        reg.onboarding.completed = true;
        Ok(reg.onboarding.clone())
    })
}

/// 允许的下一步集;fix/install/home/launch 可自环(重试属幂等动作)。
fn next_allowed(step: OnboardingStep) -> Vec<OnboardingStep> {
    use OnboardingStep::*;
    match step {
        Welcome => vec![Check],
        Check => vec![Fix, Mode],
        Fix => vec![Fix, Check, Mode],
        Mode => vec![Install],
        Install => vec![Install, Home],
        Home => vec![Home, Launch],
        Launch => vec![Launch, Done],
        Done => vec![],
    }
}

/// 命令层用:把字符串步骤名解析成枚举。
pub fn parse_step(s: &str) -> RegResult<OnboardingStep> {
    use OnboardingStep::*;
    Ok(match s.trim().to_lowercase().as_str() {
        "welcome" => Welcome,
        "check" => Check,
        "fix" => Fix,
        "mode" => Mode,
        "install" => Install,
        "home" => Home,
        "launch" => Launch,
        "done" => Done,
        other => return Err(format!("未知首跑步骤 {other:?}")),
    })
}
