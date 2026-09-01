import useAutoScrollRef from "../hooks/useAutoScrollRef";
import useProcessLog from "../hooks/useProcessLog";
import PanelHeader from "./PanelHeader";
import ProcessLogConsole from "./ProcessLogConsole";

// 底部日志停靠台:独立订阅进程日志事件,横跨三栏。
export default function ConsoleDock() {
  const { lines, clear } = useProcessLog();
  const logRef = useAutoScrollRef(lines);
  return (
    <section className="console-dock">
      <PanelHeader
        title="日志控制台"
        actions={<button onClick={clear}>清空</button>}
      />
      <ProcessLogConsole lines={lines} logRef={logRef} />
    </section>
  );
}
