import { useCallback, useEffect, useState } from "react";
import {
  listRunning,
  startTool,
  stopProfile,
  type VersionEntry,
} from "../api";
import { TOOL_HOME_ID } from "../rp-api";

// 通用工具运行状态机:探测运行中 → 启动/停止(SIGTERM 语义)。
function useToolRun(version: VersionEntry) {
  const key = TOOL_HOME_ID + "/" + version.id;
  const [args, setArgs] = useState("");
  const [cwd, setCwd] = useState(version.cwd ?? "");
  const [running, setRunning] = useState(false);
  const [busy, setBusy] = useState(false);
  const [err, setErr] = useState<string | null>(null);

  useEffect(() => {
    void listRunning()
      .then((keys) => setRunning(keys.includes(key)))
      .catch(() => setRunning(false));
  }, [key]);

  const toggle = useCallback(async () => {
    setBusy(true);
    setErr(null);
    try {
      if (running) {
        await stopProfile(TOOL_HOME_ID, version.id);
        setRunning(false);
      } else {
        const argList = args.split(/\s+/).filter(Boolean);
        await startTool(version.id, argList.length ? argList : null, cwd.trim() || null);
        setRunning(true);
      }
    } catch (e) {
      setErr(String(e));
    } finally {
      setBusy(false);
    }
  }, [running, args, cwd, version.id]);

  return { key, args, setArgs, cwd, setCwd, running, busy, err, toggle };
}

export default useToolRun;
