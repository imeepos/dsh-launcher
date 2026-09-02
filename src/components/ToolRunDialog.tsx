import useToolRun from "../hooks/useToolRun";
import { showSuccess } from "../hooks/toastStore";
import type { VersionEntry } from "../api";
import SubmitButton from "./SubmitButton";

// 通用工具运行对话框:任意参数启动/停止,单实例;日志进底部停靠台。
export default function ToolRunDialog({
  version,
  onClose,
}: {
  version: VersionEntry;
  onClose: () => void;
}) {
  const run = useToolRun(version);
  async function submit() {
    await run.toggle();
    showSuccess((run.running ? "已发送停止信号 " : "已启动 ") + version.id);
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
        <h2>运行 {version.id}</h2>
        <p className="hint mono">{version.bin}</p>
        <label>
          参数(空格分隔)
          <input value={run.args} onChange={(e) => run.setArgs(e.target.value)} placeholder="--help" autoFocus />
        </label>
        <label>
          cwd(可选)
          <input value={run.cwd} onChange={(e) => run.setCwd(e.target.value)} placeholder="运行时工作目录" />
        </label>
        {run.err && <p className="error">{run.err}</p>}
        <div className="modal-actions">
          <button type="button" onClick={onClose} disabled={run.busy}>
            关闭
          </button>
          <SubmitButton
            type="submit"
            className={run.running ? "danger" : "primary"}
            busy={run.busy}
            label={run.running ? "停止" : "启动"}
            busyLabel={run.running ? "停止中…" : "启动中…"}
          />
        </div>
      </form>
    </div>
  );
}
