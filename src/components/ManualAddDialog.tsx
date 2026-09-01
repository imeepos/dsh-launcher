import { useState } from "react";
import { addManualVersion } from "../api";
import ManualAddFields from "./ManualAddFields";

export default function ManualAddDialog({
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
        <ManualAddFields
          bin={bin}
          cwd={cwd}
          id={id}
          onBinChange={setBin}
          onCwdChange={setCwd}
          onIdChange={setId}
        />
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
