// 首跑分流:向导未完成 → 全屏向导;已完成(老用户)→ 主界面。

import type { ReactNode } from "react";
import Spinner from "./Spinner";
import WizardShell from "./onboarding/WizardShell";
import useOnboarding from "../hooks/useOnboarding";

export default function AppGate({ children }: { children: ReactNode }) {
  const { state } = useOnboarding();

  if (state === null) {
    return (
      <div className="ob-wizard">
        <div className="ob-center"><Spinner /></div>
      </div>
    );
  }

  if (!state.completed) {
    // 状态已全部落盘在后端,完成向导后整页重载即回到主界面。
    return <WizardShell onFinished={() => window.location.reload()} />;
  }

  return <>{children}</>;
}