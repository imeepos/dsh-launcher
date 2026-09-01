import { useCallback, useState } from "react";
import { fingerprintVersion, type VersionEntry } from "./api";
import "./App.css";
import ManualAddDialog from "./components/ManualAddDialog";
import InstallDialog from "./components/InstallDialog";
import DeleteConfirmDialog from "./components/DeleteConfirmDialog";
import HomesPanel from "./components/HomesPanel";
import ProcessPanel from "./components/ProcessPanel";
import VersionTable from "./components/VersionTable";
import Toolbar from "./components/Toolbar";
import ToastHost from "./components/ToastHost";
import useVersions from "./hooks/useVersions";
import { showFailure } from "./hooks/toastStore";

function App() {
  const { versions, refresh } = useVersions();
  const [manualOpen, setManualOpen] = useState(false);
  const [installOpen, setInstallOpen] = useState(false);
  const [deleteTarget, setDeleteTarget] = useState<VersionEntry | null>(null);

  // 对话框回调里的刷新失败不打断流程,轻提醒即可。
  const refreshAfterAction = useCallback(
    () => refresh().catch((e) => showFailure("刷新失败", e)),
    [refresh],
  );

  // 返回 Promise,让行内指纹按钮自己呈现 loading/成功/失败反馈。
  const doFingerprint = useCallback(
    (id: string) => fingerprintVersion(id).then(() => refresh()),
    [refresh],
  );

  return (
    <main className="container">
      <Toolbar
        onRefresh={refresh}
        onManualAdd={() => setManualOpen(true)}
        onInstall={() => setInstallOpen(true)}
      />
      <VersionTable versions={versions} onFingerprint={doFingerprint} onDelete={setDeleteTarget} />
      <HomesPanel versions={versions} />
      <ProcessPanel />
      {manualOpen && (
        <ManualAddDialog onClose={() => setManualOpen(false)} onAdded={refreshAfterAction} />
      )}
      {installOpen && (
        <InstallDialog onClose={() => setInstallOpen(false)} onDone={refreshAfterAction} />
      )}
      {deleteTarget && (
        <DeleteConfirmDialog
          version={deleteTarget}
          onClose={() => setDeleteTarget(null)}
          onDeleted={refreshAfterAction}
        />
      )}
      <ToastHost />
    </main>
  );
}

export default App;
