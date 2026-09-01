import { useCallback, useEffect, useState } from "react";
import { listVersions, type VersionEntry } from "../api";
import { showFailure } from "./toastStore";

// Owns the version list state shared by every dialog flow in App.
// refresh() rethrows so callers (e.g. AsyncButton) can render their own failure feedback.
function useVersions() {
  const [versions, setVersions] = useState<VersionEntry[]>([]);

  const refresh = useCallback(async () => {
    setVersions(await listVersions());
  }, []);

  // 首屏加载失败也走轻提醒。
  useEffect(() => {
    refresh().catch((e) => showFailure("刷新失败", e));
  }, [refresh]);

  return { versions, refresh };
}

export default useVersions;
