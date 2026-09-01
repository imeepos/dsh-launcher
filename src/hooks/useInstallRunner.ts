import { useState } from "react";
import { addDevVersion, installNpmVersion } from "../api";

export type InstallKind = "npm" | "dev";

interface InstallInputs {
  kind: InstallKind;
  version: string;
  repoPath: string;
}

// Runs the npm-install / dev-repo registration flow and tracks running/error state.
function useInstallRunner(
  inputs: InstallInputs,
  resetLog: () => void,
  onDone: () => void,
  onClose: () => void,
) {
  const [running, setRunning] = useState(false);
  const [err, setErr] = useState<string | null>(null);

  async function run() {
    setRunning(true);
    setErr(null);
    if (inputs.kind === "npm") resetLog();
    try {
      if (inputs.kind === "npm") {
        if (!inputs.version.trim()) throw new Error("版本号不能为空");
        await installNpmVersion(inputs.version.trim(), null);
      } else {
        if (!inputs.repoPath.trim()) throw new Error("repo 路径不能为空");
        await addDevVersion(inputs.repoPath.trim(), null);
      }
      onDone();
      onClose();
    } catch (e) {
      setErr(String(e));
    } finally {
      setRunning(false);
    }
  }

  return { running, err, run };
}

export default useInstallRunner;
