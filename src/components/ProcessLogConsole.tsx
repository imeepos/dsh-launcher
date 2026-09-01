import type { RefObject } from "react";

interface Props {
  lines: string[];
  logRef: RefObject<HTMLPreElement | null>;
}

function ProcessLogConsole({ lines, logRef }: Props) {
  return (
    <pre className="install-log" ref={logRef}>
      {lines.length === 0 ? "(暂无日志)" : lines.join("\n")}
    </pre>
  );
}

export default ProcessLogConsole;
