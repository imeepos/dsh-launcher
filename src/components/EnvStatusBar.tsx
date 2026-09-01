// 主界面顶部环境状态条:进入时快速预检;有可修复项时点开抽屉一键修复。

import { useCallback, useEffect, useState } from "react";
import { envCheckFast, type CheckItem } from "../api";
import useRuntimeRepair from "../hooks/useRuntimeRepair";
import { showFailure } from "../hooks/toastStore";

export default function EnvStatusBar() {
  const [items, setItems] = useState<CheckItem[] | null>(null);
  const [open, setOpen] = useState(false);
  const { lines, percent, busy, repair } = useRuntimeRepair();

  const refresh = useCallback(() => {
    envCheckFast()
      .then(setItems)
      .catch(() => setItems(null));
  }, []);

  useEffect(() => {
    refresh();
  }, [refresh]);

  const fails = (items ?? []).filter((i) => i.status === "fail");
  const tone = items === null ? "gray" : fails.length === 0 ? "green" : "red";

  const doRepair = async () => {
    try {
      await repair();
      refresh();
    } catch (e) {
      showFailure("运行时修复失败", e);
    }
  };

  return (
    <div className="envbar">
      <button
        type="button"
        className={`envbar-dot ${tone}`}
        onClick={() => setOpen((v) => !v)}
      >
        环境{fails.length > 0 ? `(待修复 ${fails.length})` : "正常"}
      </button>
      {open && (
        <EnvDrawer fails={fails} lines={lines} percent={percent} busy={busy} onRepair={doRepair} />
      )}
    </div>
  );
}

interface DrawerProps {
  fails: CheckItem[];
  lines: string[];
  percent: number;
  busy: boolean;
  onRepair: () => void;
}

function EnvDrawer({ fails, lines, percent, busy, onRepair }: DrawerProps) {
  return (
    <div className="envbar-drawer">
      {fails.length === 0 && <p className="ob-ok">环境一切正常。</p>}
      {fails.map((i) => (
        <p key={i.id}>
          <b>{i.id}</b>: {i.detail}
        </p>
      ))}
      {fails.some((i) => i.id === "runtime") && (
        <button type="button" disabled={busy} onClick={onRepair}>
          {busy ? `修复中…(估算 ${percent}%)` : "一键修复运行环境"}
        </button>
      )}
      {busy && lines.length > 0 && <pre className="ob-log">{lines.slice(-5).join("\n")}</pre>}
    </div>
  );
}