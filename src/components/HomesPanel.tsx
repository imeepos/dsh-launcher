import { useState } from "react";
import { bindHomeVersion, removeHome, type HomeEntry, type VersionEntry } from "../api";
import useHomes from "../hooks/useHomes";
import HomeFormDialog, { type HomeFormMode } from "./HomeFormDialog";
import HomesTable from "./HomesTable";

function HomesPanel({ versions }: { versions: VersionEntry[] }) {
  const { homes, error, busy, refresh, execute } = useHomes();
  const [form, setForm] = useState<{ mode: HomeFormMode; source?: HomeEntry } | null>(null);

  return (
    <section className="pane">
      <header className="panel-header">
        <h2>Homes</h2>
        <div className="toolbar-actions">
          <button onClick={() => setForm({ mode: "add" })}>登记</button>
          <button className="primary" onClick={() => setForm({ mode: "create" })}>
            新建
          </button>
        </div>
      </header>

      <div className="pane-body">
      {error && <p className="error">{error}</p>}

      <HomesTable
        homes={homes}
        versions={versions}
        busy={busy}
        onBind={(homeId, v) => void execute(() => bindHomeVersion(homeId, v))}
        onClone={(h) => setForm({ mode: "clone", source: h })}
        onDelete={(homeId) => void execute(() => removeHome(homeId))}
      />
      </div>

      {form && (
        <HomeFormDialog
          mode={form.mode}
          source={form.source}
          onClose={() => setForm(null)}
          onDone={() => void refresh()}
        />
      )}
    </section>
  );
}

export default HomesPanel;
