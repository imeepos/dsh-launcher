import { useCallback, useEffect, useRef, useState } from "react";
import { onInstallProgress, rpInstallArtifact } from "../api";
import { showSuccess } from "./toastStore";

// 制品安装执行器:进度行来自 install-progress 事件(rp- 前缀),成功后刷新工具库。
function useCatalogInstall(onInstalled: () => Promise<void>) {
  const [installing, setInstalling] = useState<string | null>(null);
  const [lines, setLines] = useState<string[]>([]);
  const [err, setErr] = useState<string | null>(null);
  const unlisten = useRef<(() => void) | null>(null);

  useEffect(
    () => () => {
      unlisten.current?.();
    },
    [],
  );

  const install = useCallback(
    (artifactId: string, tool: string | null, semver: string | null) => {
      setInstalling(artifactId);
      setErr(null);
      setLines([]);
      void onInstallProgress((p) => {
        if (p.id.startsWith("rp-")) setLines((prev) => [...prev.slice(-100), p.line]);
      }).then((u) => {
        unlisten.current?.();
        unlisten.current = u;
      });
      rpInstallArtifact(artifactId, tool, semver)
        .then((entry) => {
          showSuccess("已登记 " + entry.id);
          void onInstalled();
        })
        .catch((e) => setErr(String(e)))
        .finally(() => setInstalling(null));
    },
    [onInstalled],
  );

  return { installing, lines, err, install };
}

export default useCatalogInstall;
