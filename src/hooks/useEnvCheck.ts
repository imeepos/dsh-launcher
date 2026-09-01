import { useCallback, useState } from "react";
import { envCheck, type CheckItem } from "../api";

// 全量环境检查:向导 check/fix 步共用;busy 防重入。
export default function useEnvCheck() {
  const [items, setItems] = useState<CheckItem[] | null>(null);
  const [busy, setBusy] = useState(false);

  const run = useCallback(async () => {
    setBusy(true);
    try {
      setItems(await envCheck());
    } finally {
      setBusy(false);
    }
  }, []);

  return { items, busy, run };
}

// 与后端 blockers_cleared 同口径:没有 fail 的 blocker 即放行(skip 不挡路)。
export function blockersCleared(items: CheckItem[]): boolean {
  return !items.some((i) => i.level === "blocker" && i.status === "fail");
}