import { useState } from "react";
import { removeVersion, type VersionEntry } from "../api";

export default function DeleteConfirmDialog({
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
