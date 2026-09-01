import { useState } from "react";
import type { VersionEntry } from "./api";
import "./App.css";
import ManualAddDialog from "./components/ManualAddDialog";
import InstallDialog from "./components/InstallDialog";
import DeleteConfirmDialog from "./components/DeleteConfirmDialog";
import HomesPanel from "./components/HomesPanel";
import ProcessPanel from "./components/ProcessPanel";
import VersionTable from "./components/VersionTable";
import Toolbar from "./components/Toolbar";
import useVersions from "./hooks/useVersions";

function App() {
  const { versions, error, busyId, refresh, doFingerprint } = useVersions();
  const [manualOpen, setManualOpen] = useState(false);
  const [installOpen, setInstallOpen] = useState(false);
  const [deleteTarget, setDeleteTarget] = useState<VersionEntry | null>(null);

  return (
    <main className="container">
      <Toolbar
        onRefresh={() => void refresh()}
        onManualAdd={() => setManualOpen(true)}
        onInstall={() => setInstallOpen(true)}
      />
      {error && <p className="error">{error}</p>}
      <VersionTable
        versions={versions}
        busyId={busyId}
        onFingerprint={(id) => void doFingerprint(id)}
        onDelete={setDeleteTarget}
      />
      <HomesPanel versions={versions} />
      <ProcessPanel />
      {manualOpen && (
        <ManualAddDialog onClose={() => setManualOpen(false)} onAdded={() => void refresh()} />
      )}
      {installOpen && (
        <InstallDialog onClose={() => setInstallOpen(false)} onDone={() => void refresh()} />
      )}
      {deleteTarget && (
        <DeleteConfirmDialog
          version={deleteTarget}
          onClose={() => setDeleteTarget(null)}
          onDeleted={() => void refresh()}
        />
      )}
    </main>
  );
}

export default App;
