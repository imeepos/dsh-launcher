import { useCallback, useEffect, useRef, useState } from "react";
import { onRuntimeInstallLog, repairRuntime } from "../api";

const BASE_PERCENT = 5;
const CAP_PERCENT = 90;

// 运行时修复:无精确进度,按「日志行数 + 已耗时」爬行估算,封顶 90%,成功补满 100%,
// 失败定格并交给调用方轻提醒。事件监听在卸载时解除。
export default function useRuntimeRepair() {
  const [lines, setLines] = useState<string[]>([]);
  const [percent, setPercent] = useState(0);
  const [busy, setBusy] = useState(false);
  const unlistenRef = useRef<(() => void) | null>(null);
  const countRef = useRef(0);
  const startRef = useRef(0);

  useEffect(() => {
    let stopped = false;
    onRuntimeInstallLog((line) => {
      countRef.current += 1;
      const kept = countRef.current;
      setLines((prev) => [...prev.slice(-200), line]);
      const elapsedSec = (Date.now() - startRef.current) / 1000;
      const estimate = Math.min(CAP_PERCENT, BASE_PERCENT + kept * 2 + elapsedSec / 6);
      setPercent(Math.floor(estimate));
    })
      .then((un) => {
        if (stopped) un();
        else unlistenRef.current = un;
      })
      .catch(() => {});
    return () => {
      stopped = true;
      unlistenRef.current?.();
    };
  }, []);

  const repair = useCallback(async () => {
    if (busy) return null;
    setBusy(true);
    setLines([]);
    countRef.current = 0;
    startRef.current = Date.now();
    setPercent(BASE_PERCENT);
    try {
      const info = await repairRuntime();
      setPercent(100);
      return info;
    } finally {
      setBusy(false);
    }
  }, [busy]);

  return { lines, percent, busy, repair };
}