import { useEffect, useRef, useState } from "react";
import useAutoScrollRef from "../hooks/useAutoScrollRef";
import useProcessLog from "../hooks/useProcessLog";
import PanelHeader from "./PanelHeader";
import ProcessLogConsole from "./ProcessLogConsole";

// 底部日志停靠台:默认折叠为单行预览条;有新日志自动展开,可手动收起。
export default function ConsoleDock() {
  const { lines, clear } = useProcessLog();
  const logRef = useAutoScrollRef(lines);
  const [open, setOpen] = useState(false);
  const seen = useRef(0);

  useEffect(() => {
    if (lines.length > seen.current) setOpen(true);
    seen.current = lines.length;
  }, [lines]);

  if (!open) {
    const latest = lines[lines.length - 1] ?? "暂无日志";
    return (
      <button className="console-bar" onClick={() => setOpen(true)} aria-expanded={false}>
        <span aria-hidden>▲</span>
        <span className="latest-label">日志控制台</span>
        <span className="latest">{latest}</span>
      </button>
    );
  }
  return (
    <section className="console-dock">
      <PanelHeader
        title="日志控制台"
        actions={
          <>
            <button onClick={clear}>清空</button>
            <button onClick={() => setOpen(false)}>收起</button>
          </>
        }
      />
      <ProcessLogConsole lines={lines} logRef={logRef} />
    </section>
  );
}
