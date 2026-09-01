// fix 步:fail 的 blocker 逐卡片处理;runtime 卡一键修复(爬行进度),其余给指引。

import { useMemo } from "react";
import { blockersCleared } from "../../hooks/useEnvCheck";
import useEnvCheck from "../../hooks/useEnvCheck";
import useRuntimeRepair from "../../hooks/useRuntimeRepair";
import { showFailure } from "../../hooks/toastStore";
import Spinner from "../Spinner";

interface Props {
  onDone: () => void;
}

export default function StepFix({ onDone }: Props) {
  const { items, run } = useEnvCheck();
  const { lines, percent, busy, repair } = useRuntimeRepair();

  const failed = useMemo(() => (items ?? []).filter((i) => i.level === "blocker" && i.status === "fail"), [items]);
  const runtimeCard = failed.find((i) => i.id === "runtime");
  const others = failed.filter((i) => i.id !== "runtime");
  const cleared = items !== null && blockersCleared(items);

  const doRepair = async () => {
    try {
      await repair();
      await run();
    } catch (e) {
      showFailure("运行时修复失败", e);
    }
  };

  return (
    <div className="ob-step">
      <h2>修复问题</h2>
      {cleared && <p className="ob-ok">问题已解决,可以继续。</p>}
      {runtimeCard && (
        <RuntimeFixCard
          detail={runtimeCard.detail}
          lines={lines}
          percent={percent}
          busy={busy}
          onRepair={doRepair}
        />
      )}
      {others.map((i) => (
        <div key={i.id} className="ob-fix-card">
          <h3>{i.id}</h3>
          <p>{i.detail}。请处理后点击「重新检查」。</p>
        </div>
      ))}
      <div className="ob-row">
        <button type="button" disabled={busy} onClick={() => run().catch(() => {})}>
          重新检查
        </button>
        {cleared && (
          <button type="button" className="ob-primary" onClick={onDone}>继续</button>
        )}
      </div>
    </div>
  );
}

interface CardProps {
  detail: string;
  lines: string[];
  percent: number;
  busy: boolean;
  onRepair: () => void;
}

function RuntimeFixCard({ detail, lines, percent, busy, onRepair }: CardProps) {
  return (
    <div className="ob-fix-card">
      <h3>缺少运行环境</h3>
      <p>{detail}。将自动下载并安装(约 1-2 分钟,失败自动换源)。</p>
      {busy && (
        <div className="ob-progress">
          <div className="ob-progress-bar" style={{ width: `${percent}%` }} />
        </div>
      )}
      {busy && <Spinner />}
      <button type="button" className="ob-primary" disabled={busy} onClick={onRepair}>
        {busy ? `正在修复…(估算 ${percent}%)` : "一键安装运行环境"}
      </button>
      {lines.length > 0 && <pre className="ob-log">{lines.slice(-6).join("\n")}</pre>}
    </div>
  );
}