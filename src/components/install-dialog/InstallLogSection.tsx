import { useState, type RefObject } from "react";
import InstallLogView from "./InstallLogView";

// 安装日志默认折叠,点击切换展开查看;按钮上带当前行数。
export default function InstallLogSection({
  log,
  logRef,
}: {
  log: string[];
  logRef: RefObject<HTMLPreElement | null>;
}) {
  const [expanded, setExpanded] = useState(false);
  if (log.length === 0) return null;
  return (
    <div className="log-section">
      <button
        type="button"
        className="log-toggle"
        onClick={() => setExpanded((v) => !v)}
        aria-expanded={expanded}
      >
        <span className="log-caret" aria-hidden="true">
          {expanded ? "▾" : "▸"}
        </span>
        安装日志({log.length} 行)
      </button>
      {expanded && <InstallLogView log={log} logRef={logRef} />}
    </div>
  );
}
