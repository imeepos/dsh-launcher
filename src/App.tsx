import { useCallback, useEffect, useState, type FormEvent } from "react";
import {
  addManualVersion,
  fingerprintVersion,
  listVersions,
  type VersionEntry,
} from "./api";
import "./App.css";

function ManualAddDialog({
  onClose,
  onAdded,
}: {
  onClose: () => void;
  onAdded: () => void;
}) {
  const [bin, setBin] = useState("");
  const [cwd, setCwd] = useState("");
  const [id, setId] = useState("");
  const [err, setErr] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);

  async function submit(e: FormEvent) {
    e.preventDefault();
    setSubmitting(true);
    setErr(null);
    try {
      await addManualVersion(bin, cwd.trim() ? cwd : null, id.trim() ? id : null);
      onAdded();
      onClose();
    } catch (e2) {
      setErr(String(e2));
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <div className="modal-overlay" role="dialog" aria-modal="true">
      <form className="modal" onSubmit={(e) => void submit(e)}>
        <h2>手动添加版本</h2>
        <label>
          bin(可执行文件,必填,支持 ~)
          <input
            value={bin}
            onChange={(e) => setBin(e.target.value)}
            placeholder="~/.local/bin/dsh"
            autoFocus
          />
        </label>
        <label>
          cwd(可选)
          <input
            value={cwd}
            onChange={(e) => setCwd(e.target.value)}
            placeholder="运行时工作目录"
          />
        </label>
        <label>
          id(可选,默认 manual-&lt;bin 文件名&gt;)
          <input
            value={id}
            onChange={(e) => setId(e.target.value)}
            placeholder="my-dsh"
          />
        </label>
        {err && <p className="error">{err}</p>}
        <div className="modal-actions">
          <button type="button" onClick={onClose} disabled={submitting}>
            取消
          </button>
          <button type="submit" disabled={submitting || !bin.trim()}>
            {submitting ? "添加中…" : "添加"}
          </button>
        </div>
      </form>
    </div>
  );
}

function App() {
  const [versions, setVersions] = useState<VersionEntry[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [busyId, setBusyId] = useState<string | null>(null);
  const [manualOpen, setManualOpen] = useState(false);

  const refresh = useCallback(async () => {
    try {
      setVersions(await listVersions());
      setError(null);
    } catch (e) {
      setError(String(e));
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  async function doFingerprint(id: string) {
    setBusyId(id);
    setError(null);
    try {
      await fingerprintVersion(id);
      await refresh();
    } catch (e) {
      setError(String(e));
    } finally {
      setBusyId(null);
    }
  }

  return (
    <main className="container">
      <header className="toolbar">
        <h1>dsh-launcher</h1>
        <div className="toolbar-actions">
          <button onClick={() => void refresh()}>刷新</button>
          <button className="primary" onClick={() => setManualOpen(true)}>
            手动添加
          </button>
        </div>
      </header>

      {error && <p className="error">{error}</p>}

      <table className="version-table">
        <thead>
          <tr>
            <th>ID</th>
            <th>类型</th>
            <th>spec / bin</th>
            <th>cwd</th>
            <th>指纹</th>
            <th>操作</th>
          </tr>
        </thead>
        <tbody>
          {versions.length === 0 ? (
            <tr>
              <td colSpan={6} className="empty">
                还没有版本,点「手动添加」登记一个
              </td>
            </tr>
          ) : (
            versions.map((v) => (
              <tr key={v.id}>
                <td>{v.id}</td>
                <td>{v.kind}</td>
                <td className="mono">{v.spec ?? v.bin}</td>
                <td className="mono">{v.cwd ?? "—"}</td>
                <td className="mono">{v.fingerprint ?? "未采集"}</td>
                <td>
                  <button
                    onClick={() => void doFingerprint(v.id)}
                    disabled={busyId !== null}
                  >
                    {busyId === v.id ? "采集中…" : "指纹"}
                  </button>
                </td>
              </tr>
            ))
          )}
        </tbody>
      </table>

      {manualOpen && (
        <ManualAddDialog
          onClose={() => setManualOpen(false)}
          onAdded={() => void refresh()}
        />
      )}
    </main>
  );
}

export default App;
