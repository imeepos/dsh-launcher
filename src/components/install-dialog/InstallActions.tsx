import SubmitButton from "../SubmitButton";

export default function InstallActions({
  onClose,
  onRun,
  running,
  idleLabel,
  busyLabel,
}: {
  onClose: () => void;
  onRun: () => void;
  running: boolean;
  idleLabel: string;
  busyLabel: string;
}) {
  return (
    <div className="modal-actions">
      <button type="button" onClick={onClose} disabled={running}>
        {running ? "后台运行,可关闭" : "取消"}
      </button>
      <SubmitButton
        className="primary"
        busy={running}
        label={idleLabel}
        busyLabel={busyLabel}
        onClick={onRun}
      />
    </div>
  );
}
