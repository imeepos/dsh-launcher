import { useCallback, useEffect, useState } from "react";
import {
  onboardingAdvance,
  onboardingComplete,
  onboardingGet,
  type OnboardingState,
  type OnboardingStepName,
} from "../api";
import { showFailure } from "./toastStore";

// 首跑状态:进入即读取,advance/complete 成功后本地同步,失败走轻提醒。
export default function useOnboarding() {
  const [state, setState] = useState<OnboardingState | null>(null);

  const refresh = useCallback(async () => {
    setState(await onboardingGet());
  }, []);

  useEffect(() => {
    refresh().catch((e) => showFailure("读取首跑状态失败", e));
  }, [refresh]);

  const advance = useCallback(async (step: OnboardingStepName) => {
    const next = await onboardingAdvance(step);
    setState(next);
    return next;
  }, []);

  const complete = useCallback(async () => {
    const next = await onboardingComplete();
    setState(next);
    return next;
  }, []);

  return { state, refresh, advance, complete };
}