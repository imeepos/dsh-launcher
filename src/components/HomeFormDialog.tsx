import { useState } from "react";
import { addHome, cloneHome, createHome, type HomeEntry } from "../api";
import HomeFormFields from "./HomeFormFields";

export type HomeFormMode = "add" | "create" | "clone";

interface Props {
  mode: HomeFormMode;
  source?: HomeEntry;
  onClose: () => void;
  onDone: () => void;
}

async function submitHomeForm(
  mode: HomeFormMode,
  source: HomeEntry | null,
  path: string,
  id: string,
) {
  const idArg = id.trim() ? id : null;
  if (mode === "add") return addHome(path, idArg);
  if (mode === "create") return createHome(path.trim() ? path : null, idArg);
  return cloneHome((source as HomeEntry).id, path.trim() ? path : null, idArg);
}

function titleOf(mode: HomeFormMode, source?: HomeEntry) {
  if (mode === "add") return "登记 home(既有目录)";
  if (mode === "create") return "新建 home";
  return "克隆 home " + (source?.id ?? "");
}

function HomeFormDialog({ mode, source, onClose, onDone }: Props) {
  const [path, setPath] = useState("");
  const [id, setId] = useState(mode === "clone" && source ? source.id + "-clone" : "");
  const [err, setErr] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);

  async function submit() {
    setSubmitting(true);
    setErr(null);
    try {
      await submitHomeForm(mode, source ?? null, path, id);
      onDone();
      onClose();
    } catch (e) {
      setErr(String(e));
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <div className="modal-overlay" role="dialog" aria-modal="true">
      <form
        className="modal"
        onSubmit={(e) => {
          e.preventDefault();
          void submit();
        }}
      >
        <h2>{titleOf(mode, source)}</h2>
        <HomeFormFields mode={mode} path={path} onPath={setPath} id={id} onId={setId} />
        {err && <p className="error">{err}</p>}
        <div className="modal-actions">
          <button type="button" onClick={onClose} disabled={submitting}>
            取消
          </button>
          <button type="submit" className="primary" disabled={submitting}>
            {submitting ? "处理中…" : "确定"}
          </button>
        </div>
      </form>
    </div>
  );
}

export default HomeFormDialog;
