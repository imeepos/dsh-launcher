import { useCallback, useEffect, useRef, useState } from "react";
import {
  addDevVersion,
  addManualVersion,
  fingerprintVersion,
  installNpmVersion,
  listVersions,
  onInstallProgress,
  removeVersion,
  type VersionEntry,
  type VersionKind,
} from "./api";
import "./App.css";

const DEV_REPO_PLACEHOLDER = "/Users/imeepos/ext512/ymm-001/deepseek-harness";
const DEFAULT_NPM_VERSION = "0.1.1-rc.2";

function KindBadge({ kind }: { kind: VersionKind }) {
  return (
    <span className={"badge badge-" + kind}>{kind}</span>
  );
}

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

  async function submit() {
    setSubmitting(true);
    setErr(null);
    try {
      await addManualVersion(bin, cwd.trim() ? cwd : null, id.trim() ? id : null);
      onAdded();
      onClose();
    } catch (e) {
      setErr(String(e));
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <div className="modal-overlay" role="dialog" aria-modal="true">
      <form className="modal" onSubmit={(e) => { e.preventDefault(); void submit(); }}>
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
          <input value={cwd} onChange={(e) => setCwd(e.target.value)} placeholder="运行时工作目录" />
        </label>
        <label>
          id(可选,默认 manual-&lt;bin 文件名&gt;)
          <input value={id} onChange={(e) => setId(e.target.value)} placeholder="my-dsh" />
        </label>
        {err && <p className="error">{err}</p>}
        <div className="modal-actions">
          <button type="button" onClick={onClose} disabled={submitting}>
            取消
          </button>
          <button type="submit" className="primary" disabled={submitting || !bin.trim()}>
            {submitting ? "添加中…" : "添加"}
          </button>
        </div>
      </form>
    </div>
  );
}

function InstallDialog({
  onClose,
  onDone,
}: {
  onClose: () => void;
  onDone: () => void;
}) {
  const [kind, setKind] = useState<"npm" | "dev">("npm");
  const [version, setVersion] = useState(DEFAULT_NPM_VERSION);
  const [repoPath, setRepoPath] = useState(DEV_REPO_PLACEHOLDER);
  const [log, setLog] = useState<string[]>([]);
  const [running, setRunning] = useState(false);
  const [err, setErr] = useState<string | null>(null);
  const logRef = useRef<HTMLPreElement | null>(null);

  useEffect(() => {
    let unlisten: (() => void) | null = null;
    let cancelled = false;
    void onInstallProgress((p) => {
      setLog((prev) => [...prev.slice(-300), p.line]);
    }).then((u) => {
      if (cancelled) u();
      else unlisten = u;
    });
    return () => {
      cancelled = true;
      if (unlisten) unlisten();
    };
  }, []);

  useEffect(() => {
    const el = logRef.current;
    if (el) el.scrollTop = el.scrollHeight;
  }, [log]);

  async function run() {
    setRunning(true);
    setErr(null);
    if (kind === "npm") setLog([]);
    try {
      if (kind === "npm") {
        if (!version.trim()) throw new Error("版本号不能为空");
        await installNpmVersion(version.trim(), null);
      } else {
        if (!repoPath.trim()) throw new Error("repo 路径不能为空");
        await addDevVersion(repoPath.trim(), null);
      }
      onDone();
      onClose();
    } catch (e) {
      setErr(String(e));
    } finally {
      setRunning(false);
    }
  }

  return (
    <div className="modal-overlay" role="dialog" aria-modal="true">
      <div className="modal">
        <h2>添加版本</h2>
        <div className="kind-picker">
          <button
            type="button"
            className={kind === "npm" ? "kind-btn active" : "kind-btn"}
            onClick={() => setKind("npm")}
            disabled={running}
          >
            npm 安装
          </button>
          <button
            type="button"
            className={kind === "dev" ? "kind-btn active" : "kind-btn"}
            onClick={() => setKind("dev")}
            disabled={running}
          >
            dev 仓库
          </button>
        </div>
        {kind === "npm" ? (
          <>
            <label>
              DSH 版本号
              <input
                value={version}
                onChange={(e) => setVersion(e.target.value)}
                placeholder="0.1.1-rc.2"
                disabled={running}
              />
            </label>
            <p className="hint">
              执行 npm install --prefix ~/.dsh-launcher/versions/v&lt;版本&gt; @deepseek-ai/dsh@&lt;版本&gt;
            </p>
          </>
        ) : (
          <>
            <label>
              repo checkout 路径
              <input
                value={repoPath}
                onChange={(e) => setRepoPath(e.target.value)}
                placeholder={DEV_REPO_PLACEHOLDER}
                disabled={running}
              />
            </label>
            <p className="hint">登记为 dev 版本:启动命令 pnpm dsh,cwd=repo 路径</p>
          </>
        )}
        {kind === "npm" && (log.length > 0 || running) && (
          <pre className="install-log" ref={logRef}>
            {log.join("\n")}
          </pre>
        )}
        {err && <p className="error">{err}</p>}
        <div className="modal-actions">
          <button type="button" onClick={onClose} disabled={running}>
            {running ? "后台运行,可关闭" : "取消"}
          </button>
          <button
            type="button"
            className="primary"
            onClick={() => void run()}
            disabled={running}
          >
            {running
              ? kind === "npm"
                ? "安装中…"
                : "登记中…"
              : err
                ? "重试"
                : kind === "npm"
                ? "安装"
                : "登记"}
          </button>
        </div>
      </div>
    </div>
  );
}

function DeleteConfirmDialog({
  version,
  onClose,
  onDeleted,
}: {
  version: VersionEntry;
  onClose: () => void;
  onDeleted: () => void;
}) {
  const [busy, setBusy] = useState(false);
  const [err, setErr] = useState<string | null>(null);

  async function confirm() {
    setBusy(true);
    setErr(null);
    try {
      await removeVersion(version.id);
      onDeleted();
      onClose();
    } catch (e) {
      setErr(String(e));
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="modal-overlay" role="dialog" aria-modal="true">
      <div className="modal">
        <h2>删除版本 {version.id}</h2>
        {version.kind === "npm" ? (
          <p>
            将摘除登记,并删除独立依赖树目录
            ~/.dsh-launcher/versions/{version.id},此操作不可恢复。
          </p>
        ) : (
          <p>仅摘除登记,不动磁盘上的任何文件。</p>
        )}
        {err && <p className="error">{err}</p>}
        <div className="modal-actions">
          <button type="button" onClick={onClose} disabled={busy}>
            取消
          </button>
          <button type="button" className="danger" onClick={() => void confirm()} disabled={busy}>
            {busy ? "删除中…" : "确认删除"}
          </button>
        </div>
      </div>
    </div>
  );
}

function App() {
  const [versions, setVersions] = useState<VersionEntry[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [busyId, setBusyId] = useState<string | null>(null);
  const [manualOpen, setManualOpen] = useState(false);
  const [installOpen, setInstallOpen] = useState(false);
  const [deleteTarget, setDeleteTarget] = useState<VersionEntry | null>(null);

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
          <button onClick={() => setManualOpen(true)}>手动添加</button>
          <button className="primary" onClick={() => setInstallOpen(true)}>
            安装新版本
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
                还没有版本,点「安装新版本」或「手动添加」
              </td>
            </tr>
          ) : (
            versions.map((v) => (
              <tr key={v.id}>
                <td>{v.id}</td>
                <td>
                  <KindBadge kind={v.kind} />
                </td>
                <td className="mono">{v.spec ?? v.bin}</td>
                <td className="mono">{v.cwd ?? "—"}</td>
                <td className="mono">{v.fingerprint ?? "未采集"}</td>
                <td>
                  <div className="row-actions">
                    <button
                      onClick={() => void doFingerprint(v.id)}
                      disabled={busyId !== null}
                    >
                      {busyId === v.id ? "采集中…" : "指纹"}
                    </button>
                    <button className="danger" onClick={() => setDeleteTarget(v)}>
                      删除
                    </button>
                  </div>
                </td>
              </tr>
            ))
          )}
        </tbody>
      </table>

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