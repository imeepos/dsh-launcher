import { useCallback, useEffect, useState } from "react";
import { listRunning, onProcessExit } from "../api";

// Tracks running process keys; process-exit events remove keys automatically.
function useRunningKeys() {
  const [running, setRunning] = useState<string[]>([]);

  const refresh = useCallback(async () => {
    try {
      setRunning(await listRunning());
    } catch {
      // 拉取失败不阻塞面板;启动/停止错误单独展示
    }
  }, []);

  useEffect(() => {
    void refresh();
    let un: (() => void) | null = null;
    let cancelled = false;
    void onProcessExit((p) => {
      const key = p.homeId + "/" + p.profile;
      setRunning((prev) => prev.filter((k) => k !== key));
    }).then((u) => {
      if (cancelled) u();
      else un = u;
    });
    return () => {
      cancelled = true;
      if (un) un();
    };
  }, [refresh]);

  const markRunning = useCallback(
    (key: string) => setRunning((prev) => (prev.includes(key) ? prev : [...prev, key])),
    [],
  );

  const unmarkRunning = useCallback(
    (key: string) => setRunning((prev) => prev.filter((k) => k !== key)),
    [],
  );

  return { running, refresh, markRunning, unmarkRunning };
}

export default useRunningKeys;
