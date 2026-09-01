import { useEffect, useRef, useState } from "react";
import { onInstallProgress } from "../api";

// Streams install progress lines into a capped log and keeps the <pre> scrolled down.
function useInstallLog() {
  const [log, setLog] = useState<string[]>([]);
  const logRef = useRef<HTMLPreElement | null>(null);

  useEffect(() => {
    let unlisten: (() => void) | null = null;
    let cancelled = false;
    void onInstallProgress((p) => {
      setLog((prev) => [...prev.slice(-300), p.line]);
    }).then((u) => {
      if (cancelled) u();
      else unlisten = u;
    });
    return () => {
      cancelled = true;
      if (unlisten) unlisten();
    };
  }, []);

  useEffect(() => {
    const el = logRef.current;
    if (el) el.scrollTop = el.scrollHeight;
  }, [log]);

  return { log, logRef, resetLog: () => setLog([]) };
}

export default useInstallLog;
