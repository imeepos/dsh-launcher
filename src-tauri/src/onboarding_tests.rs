//! onboarding 纯逻辑单测:转移表与序列化缺省值,不碰真实注册表。

#[cfg(test)]
mod tests {
    use crate::onboarding::parse_step;
    use crate::registry::{OnboardingState, OnboardingStep, Registry};

    #[test]
    fn missing_onboarding_field_defaults_to_welcome() {
        let s: OnboardingState = serde_json::from_str("{}").unwrap();
        assert_eq!(s.step, OnboardingStep::Welcome);
        assert!(!s.completed);
    }

    #[test]
    fn default_registry_starts_welcome() {
        let reg = Registry::default();
        assert_eq!(reg.onboarding.step, OnboardingStep::Welcome);
        assert!(!reg.onboarding.completed);
    }

    #[test]
    fn parse_step_accepts_all_names() {
        assert_eq!(parse_step("welcome").unwrap(), OnboardingStep::Welcome);
        assert_eq!(parse_step("Launch").unwrap(), OnboardingStep::Launch);
        assert_eq!(parse_step(" done ").unwrap(), OnboardingStep::Done);
        assert!(parse_step("nonsense").is_err());
    }

    #[test]
    fn step_serializes_lowercase() {
        let json = serde_json::to_string(&OnboardingStep::Install).unwrap();
        assert_eq!(json, "\"install\"");
    }
}
