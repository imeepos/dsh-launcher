export default function InstallActions({
  onClose,
  onRun,
  running,
  label,
}: {
  onClose: () => void;
  onRun: () => void;
  running: boolean;
  label: string;
}) {
  return (
    <div className="modal-actions">
      <button type="button" onClick={onClose} disabled={running}>
        {running ? "后台运行,可关闭" : "取消"}
      </button>
      <button type="button" className="primary" onClick={onRun} disabled={running}>
        {label}
      </button>
    </div>
  );
}
