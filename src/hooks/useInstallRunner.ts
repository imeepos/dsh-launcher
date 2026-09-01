import { useState } from "react";
import { addDevVersion, installNpmVersion } from "../api";
import { showSuccess } from "./toastStore";

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
      const entry =
        inputs.kind === "npm"
          ? await installNpmVersion(validate(inputs.version, "版本号不能为空"), null)
          : await addDevVersion(validate(inputs.repoPath, "repo 路径不能为空"), null);
      showSuccess((inputs.kind === "npm" ? "安装完成 " : "登记完成 ") + entry.id);
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

function validate(value: string, message: string): string {
  const trimmed = value.trim();
  if (!trimmed) throw new Error(message);
  return trimmed;
}

export default useInstallRunner;
