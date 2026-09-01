// check 步:全量环境检查流式展示;blocker 全过 → mode,否则 → fix。

import { useCallback, useEffect, useState } from "react";
import type { CheckItem } from "../../api";
import { blockersCleared } from "../../hooks/useEnvCheck";
import useEnvCheck from "../../hooks/useEnvCheck";
import Spinner from "../Spinner";

const ICON: Record<string, string> = { pass: "✓", fail: "✕", skip: "—" };

interface Props {
  onDone: (cleared: boolean) => void;
}

export default function StepCheck({ onDone }: Props) {
  const { items, busy, run } = useEnvCheck();
  const [done, setDone] = useState(false);

  const runCheck = useCallback(() => {
    setDone(false);
    run().catch(() => setDone(true));
  }, [run]);

  useEffect(() => {
    runCheck();
  }, [runCheck]);

  if (busy || !items) {
    return (
      <div className="ob-center">
        <Spinner />
        <p className="ob-muted">正在检查运行环境…</p>
      </div>
    );
  }

  const cleared = blockersCleared(items);
  return (
    <div className="ob-step">
      <h2>环境检查</h2>
      <ul className="ob-checklist">
        {items.map((item: CheckItem) => (
          <li key={item.id} className={`ob-check ob-${item.status}`}>
            <span className="ob-check-icon">{ICON[item.status] ?? "?"}</span>
            <span className="ob-check-id">{item.id}</span>
            <span className="ob-check-detail">{item.detail}</span>
          </li>
        ))}
      </ul>
      {done && (
        <button
          type="button"
          className="ob-primary"
          onClick={() => onDone(cleared)}
        >
          {cleared ? "一切正常,继续" : "去修复问题"}
        </button>
      )}
    </div>
  );
}