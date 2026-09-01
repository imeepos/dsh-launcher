import type { RefObject } from "react";

export default function InstallLogView({
  log,
  logRef,
}: {
  log: string[];
  logRef: RefObject<HTMLPreElement | null>;
}) {
  return (
    <pre className="install-log" ref={logRef}>
      {log.join("\n")}
    </pre>
  );
}
