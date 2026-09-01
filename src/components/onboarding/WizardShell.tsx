// 首跑向导外壳:读注册表断点状态,装配步骤条与步骤体;状态落盘由后端负责。

import { useState } from "react";
import Spinner from "../Spinner";
import useOnboarding from "../../hooks/useOnboarding";
import { showFailure } from "../../hooks/toastStore";
import StepBody, { type Flow } from "./StepBody";
import { StepCrumbs } from "./StepEnds";

export default function WizardShell({ onFinished }: { onFinished: () => void }) {
  const { state, advance, complete } = useOnboarding();
  const [version, setVersion] = useState<string | null>(null);
  const [versionId, setVersionId] = useState<string | null>(null);
  const [homeId, setHomeId] = useState<string | null>(null);
  const [profile, setProfile] = useState("main");

  if (!state) {
    return (
      <div className="ob-wizard">
        <div className="ob-center"><Spinner /></div>
      </div>
    );
  }

  const go = (step: Parameters<typeof advance>[0]) => {
    advance(step).catch((e) => showFailure("步进失败", e));
  };

  const finish = () => {
    complete()
      .catch((e) => showFailure("标记完成失败", e))
      .finally(onFinished);
  };

  const flow: Flow = {
    state, version, setVersion, versionId, setVersionId,
    homeId, setHomeId, profile, setProfile, go, finish,
  };

  return (
    <div className="ob-wizard">
      <header className="ob-header">
        <h1>dsh 启动器 · 首次设置</h1>
        <StepCrumbs current={state.step} />
      </header>
      <main className="ob-main">
        <StepBody flow={flow} />
      </main>
    </div>
  );
}