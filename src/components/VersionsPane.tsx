import { useCallback, useState } from "react";
import { fingerprintVersion, type VersionEntry } from "../api";
import AsyncButton from "./AsyncButton";
import DeleteConfirmDialog from "./DeleteConfirmDialog";
import InstallDialog from "./InstallDialog";
import ManualAddDialog from "./ManualAddDialog";
import PanelHeader from "./PanelHeader";
import ToolRunDialog from "./ToolRunDialog";
import VersionTable from "./VersionTable";
import { showFailure } from "../hooks/toastStore";

function PaneActions({
  refresh,
  onManualAdd,
  onInstall,
}: {
  refresh: () => Promise<void>;
  onManualAdd: () => void;
  onInstall: () => void;
}) {
  return (
    <>
      {/* 点击 -> 刷新中… -> 已刷新(短暂) -> 刷新;失败:✕ + 轻提醒 */}
      <AsyncButton
        task={refresh}
        idle="刷新"
        loading="刷新中…"
        success="已刷新"
        failurePrefix="刷新失败"
      />
      <button onClick={onManualAdd}>手动添加</button>
      <button className="primary" onClick={onInstall}>
        安装新版本
      </button>
    </>
  );
}

function VersionDialogs({
  refresh,
  manualOpen,
  installOpen,
  deleteTarget,
  runTarget,
  close,
}: {
  refresh: () => Promise<void>;
  manualOpen: boolean;
  installOpen: boolean;
  deleteTarget: VersionEntry | null;
  runTarget: VersionEntry | null;
  close: () => void;
}) {
  return (
    <>
      {manualOpen && <ManualAddDialog onClose={close} onAdded={refresh} />}
      {installOpen && <InstallDialog onClose={close} onDone={refresh} />}
      {deleteTarget && <DeleteConfirmDialog version={deleteTarget} onClose={close} onDeleted={refresh} />}
      {runTarget && <ToolRunDialog version={runTarget} onClose={close} />}
    </>
  );
}

export default function VersionsPane({
  versions,
  refresh,
}: {
  versions: VersionEntry[];
  refresh: () => Promise<void>;
}) {
  const [manualOpen, setManualOpen] = useState(false);
  const [installOpen, setInstallOpen] = useState(false);
  const [deleteTarget, setDeleteTarget] = useState<VersionEntry | null>(null);
  const [runTarget, setRunTarget] = useState<VersionEntry | null>(null);
  const refreshAfterAction = useCallback(
    () => refresh().catch((e) => showFailure("刷新失败", e)),
    [refresh],
  );
  const doFingerprint = useCallback(
    (id: string) => fingerprintVersion(id).then(() => refresh()),
    [refresh],
  );
  const close = () => {
    setManualOpen(false);
    setInstallOpen(false);
    setDeleteTarget(null);
    setRunTarget(null);
  };
  return (
    <section className="pane">
      <PanelHeader
        title="工具库"
        actions={
          <PaneActions
            refresh={refresh}
            onManualAdd={() => setManualOpen(true)}
            onInstall={() => setInstallOpen(true)}
          />
        }
      />
      <div className="pane-body">
        <VersionTable
          versions={versions}
          onFingerprint={doFingerprint}
          onDelete={setDeleteTarget}
          onRun={setRunTarget}
        />
      </div>
      <VersionDialogs
        refresh={refreshAfterAction}
        manualOpen={manualOpen}
        installOpen={installOpen}
        deleteTarget={deleteTarget}
        runTarget={runTarget}
        close={close}
      />
    </section>
  );
}
