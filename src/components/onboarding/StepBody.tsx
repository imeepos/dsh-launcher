// 步骤体路由:按断点状态渲染当前步;首尾两屏(欢迎/完成)在 StepEnds。

import type { OnboardingState, OnboardingStepName } from "../../api";
import StepCheck from "./StepCheck";
import StepFix from "./StepFix";
import StepHome from "./StepHome";
import StepInstall from "./StepInstall";
import StepLaunch from "./StepLaunch";
import StepMode from "./StepMode";
import { DoneCard, WelcomeCard } from "./StepEnds";

export interface Flow {
  state: OnboardingState;
  version: string | null;
  setVersion: (v: string) => void;
  versionId: string | null;
  setVersionId: (id: string) => void;
  homeId: string | null;
  setHomeId: (id: string) => void;
  profile: string;
  setProfile: (p: string) => void;
  go: (step: OnboardingStepName) => void;
  finish: () => void;
}

export default function StepBody({ flow }: { flow: Flow }) {
  const { state } = flow;
  switch (state.step) {
    case "welcome":
      return <WelcomeCard onStart={() => flow.go("check")} />;
    case "check":
      return <StepCheck onDone={(cleared) => flow.go(cleared ? "mode" : "fix")} />;
    case "fix":
      return <StepFix onDone={() => flow.go("mode")} />;
    case "mode":
      return (
        <StepMode
          onNext={(v) => {
            flow.setVersion(v);
            flow.go("install");
          }}
        />
      );
    case "install":
      return <InstallBranch flow={flow} />;
    case "home":
      return <HomeBranch flow={flow} />;
    case "launch":
      return <LaunchBranch flow={flow} />;
    case "done":
      return <DoneCard onFinish={flow.finish} />;
  }
}

function InstallBranch({ flow }: { flow: Flow }) {
  if (flow.version === null) {
    return (
      <div className="ob-step">
        <p>缺少版本信息。</p>
        <button type="button" onClick={() => flow.go("mode")}>返回重选</button>
      </div>
    );
  }
  return (
    <StepInstall
      version={flow.version}
      onDone={(id) => {
        flow.setVersionId(id);
        flow.go("home");
      }}
    />
  );
}

function HomeBranch({ flow }: { flow: Flow }) {
  return (
    <StepHome
      versionId={flow.versionId}
      onDone={(h, p) => {
        flow.setHomeId(h.id);
        flow.setProfile(p);
        flow.go("launch");
      }}
    />
  );
}

function LaunchBranch({ flow }: { flow: Flow }) {
  return <StepLaunch homeId={flow.homeId ?? ""} profile={flow.profile} onComplete={flow.finish} />;
}