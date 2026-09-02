import useManualAdd from "../hooks/useManualAdd";
import ManualAddFields from "./ManualAddFields";
import SubmitButton from "./SubmitButton";

export default function ManualAddDialog({
  onClose,
  onAdded,
}: {
  onClose: () => void;
  onAdded: () => void;
}) {
  const { bin, cwd, id, tool, err, submitting, setBin, setCwd, setId, setTool, submit } =
    useManualAdd(onAdded, onClose);

  return (
    <div className="modal-overlay" role="dialog" aria-modal="true">
      <form className="modal" onSubmit={(e) => { e.preventDefault(); void submit(); }}>
        <h2>手动添加版本</h2>
        <ManualAddFields
          bin={bin}
          cwd={cwd}
          id={id}
          tool={tool}
          onBinChange={setBin}
          onCwdChange={setCwd}
          onIdChange={setId}
          onToolChange={setTool}
        />
        {err && <p className="error">{err}</p>}
        <div className="modal-actions">
          <button type="button" onClick={onClose} disabled={submitting}>
            取消
          </button>
          <SubmitButton
            type="submit"
            className="primary"
            busy={submitting}
            label="添加"
            busyLabel="添加中…"
            disabled={!bin.trim()}
          />
        </div>
      </form>
    </div>
  );
}
