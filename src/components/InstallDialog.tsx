import { useState } from "react";
import useInstallLog from "../hooks/useInstallLog";
import useInstallRunner, { type InstallKind } from "../hooks/useInstallRunner";
import KindPicker from "./install-dialog/KindPicker";
import { DevFields, NpmFields } from "./install-dialog/InstallFields";
import InstallLogView from "./install-dialog/InstallLogView";
import InstallActions from "./install-dialog/InstallActions";
import { submitLabel } from "./install-dialog/submitLabel";

const DEV_REPO_PLACEHOLDER = "/Users/imeepos/ext512/ymm-001/deepseek-harness";
const DEFAULT_NPM_VERSION = "0.1.1-rc.2";

export default function InstallDialog({
  onClose,
  onDone,
}: {
  onClose: () => void;
  onDone: () => void;
}) {
  const [kind, setKind] = useState<InstallKind>("npm");
  const [version, setVersion] = useState(DEFAULT_NPM_VERSION);
  const [repoPath, setRepoPath] = useState(DEV_REPO_PLACEHOLDER);
  const { log, logRef, resetLog } = useInstallLog();
  const { running, err, run } = useInstallRunner(
    { kind, version, repoPath },
    resetLog,
    onDone,
    onClose,
  );

  return (
    <div className="modal-overlay" role="dialog" aria-modal="true">
      <div className="modal">
        <h2>添加版本</h2>
        <KindPicker kind={kind} onSelect={setKind} disabled={running} />
        {kind === "npm" ? (
          <NpmFields version={version} onVersionChange={setVersion} disabled={running} />
        ) : (
          <DevFields
            repoPath={repoPath}
            onRepoPathChange={setRepoPath}
            disabled={running}
            placeholder={DEV_REPO_PLACEHOLDER}
          />
        )}
        {kind === "npm" && (log.length > 0 || running) && (
          <InstallLogView log={log} logRef={logRef} />
        )}
        {err && <p className="error">{err}</p>}
        <InstallActions
          onClose={onClose}
          onRun={() => void run()}
          running={running}
          label={submitLabel(running, err, kind)}
        />
      </div>
    </div>
  );
}
