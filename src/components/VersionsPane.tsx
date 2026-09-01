import { useCallback, useState } from "react";
import { fingerprintVersion, type VersionEntry } from "../api";
import AsyncButton from "./AsyncButton";
import DeleteConfirmDialog from "./DeleteConfirmDialog";
import InstallDialog from "./InstallDialog";
import ManualAddDialog from "./ManualAddDialog";
import PanelHeader from "./PanelHeader";
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
  close,
}: {
  refresh: () => Promise<void>;
  manualOpen: boolean;
  installOpen: boolean;
  deleteTarget: VersionEntry | null;
  close: () => void;
}) {
  return (
    <>
      {manualOpen && <ManualAddDialog onClose={close} onAdded={refresh} />}
      {installOpen && <InstallDialog onClose={close} onDone={refresh} />}
      {deleteTarget && <DeleteConfirmDialog version={deleteTarget} onClose={close} onDeleted={refresh} />}
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
  // 对话框回调里的刷新失败不打断流程,轻提醒即可。
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
  };
  return (
    <section className="pane">
      <PanelHeader
        title="版本库"
        actions={
          <PaneActions
            refresh={refresh}
            onManualAdd={() => setManualOpen(true)}
            onInstall={() => setInstallOpen(true)}
          />
        }
      />
      <div className="pane-body">
        <VersionTable versions={versions} onFingerprint={doFingerprint} onDelete={setDeleteTarget} />
      </div>
      <VersionDialogs
        refresh={refreshAfterAction}
        manualOpen={manualOpen}
        installOpen={installOpen}
        deleteTarget={deleteTarget}
        close={close}
      />
    </section>
  );
}
