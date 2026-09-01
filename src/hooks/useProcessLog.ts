import { useCallback, useEffect, useState } from "react";
import { onProcessLog } from "../api";

// Collects process-log event lines into a bounded buffer for the console.
function useProcessLog(max = 200) {
  const [lines, setLines] = useState<string[]>([]);

  const clear = useCallback(() => setLines([]), []);

  useEffect(() => {
    let un: (() => void) | null = null;
    let cancelled = false;
    void onProcessLog((p) => {
      const prefix = p.homeId + "/" + p.profile + (p.isErr ? " [err] " : " | ");
      setLines((prev) => [...prev.slice(-(max - 1)), prefix + p.line]);
    }).then((u) => {
      if (cancelled) u();
      else un = u;
    });
    return () => {
      cancelled = true;
      if (un) un();
    };
  }, [max]);

  return { lines, clear };
}

export default useProcessLog;
