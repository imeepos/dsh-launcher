// launch 步:启动 profile 并观察 30 秒(存活 + 日志);观察逻辑在 useLaunchObservation。

import { useState } from "react";
import { startProfile } from "../../api";
import useLaunchObservation from "../../hooks/useLaunchObservation";
import { showFailure } from "../../hooks/toastStore";
import Spinner from "../Spinner";

interface Props {
  homeId: string;
  profile: string;
  onComplete: () => void;
}

type Phase = "idle" | "starting" | "watching" | "ok" | "fail";

export default function StepLaunch({ homeId, profile, onComplete }: Props) {
  const [phase, setPhase] = useState<Phase>("idle");
  const [logTail, setLogTail] = useState<string[]>([]);

  useLaunchObservation({
    homeId,
    profile,
    observing: phase === "watching",
    onLog: (line) => setLogTail((prev) => [...prev.slice(-8), line]),
    onFail: (why) => {
      setPhase("fail");
      showFailure("启动观察失败", why);
    },
    onHealthy: () => setPhase("ok"),
  });

  const start = async () => {
    setPhase("starting");
    try {
      await startProfile(homeId, profile, null, null, null);
      setPhase("watching");
    } catch (e) {
      setPhase("fail");
      showFailure("启动失败", e);
    }
  };

  return (
    <div className="ob-step">
      <h2>首次启动 {profile}</h2>
      <PhaseView phase={phase} onStart={start} onComplete={onComplete} />
      {logTail.length > 0 && <pre className="ob-log">{logTail.join("\n")}</pre>}
    </div>
  );
}

function PhaseView({ phase, onStart, onComplete }: { phase: Phase; onStart: () => void; onComplete: () => void }) {
  if (phase === "idle") {
    return (
      <button type="button" className="ob-primary" onClick={onStart}>启动并观察 30 秒</button>
    );
  }
  if (phase === "starting" || phase === "watching") {
    return (
      <div className="ob-center">
        <Spinner />
        <p className="ob-muted">
          {phase === "starting" ? "正在启动…" : "运行中,观察 30 秒确认健康…"}
        </p>
      </div>
    );
  }
  if (phase === "ok") {
    return (
      <div className="ob-center">
        <p className="ob-ok">✓ 启动成功,一切正常!</p>
        <button type="button" className="ob-primary" onClick={onComplete}>完成</button>
      </div>
    );
  }
  return <p className="ob-err">启动未通过健康检查,可重试;反复失败请查看日志反馈。</p>;
}