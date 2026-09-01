import { useState } from "react";
import { removeVersion, type VersionEntry } from "../api";
import { showSuccess } from "../hooks/toastStore";
import SubmitButton from "./SubmitButton";

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
      showSuccess("已删除版本 " + version.id);
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
          <SubmitButton
            className="danger"
            busy={busy}
            label="确认删除"
            busyLabel="删除中…"
            onClick={() => void confirm()}
          />
        </div>
      </div>
    </div>
  );
}
