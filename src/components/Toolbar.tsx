import AsyncButton from "./AsyncButton";
import ThemeToggle from "./ThemeToggle";

export default function Toolbar({
  onRefresh,
  onManualAdd,
  onInstall,
}: {
  onRefresh: () => Promise<void>;
  onManualAdd: () => void;
  onInstall: () => void;
}) {
  return (
    <header className="toolbar">
      <div className="brand">
        <img src="/assets/dsh-launcher-icon.png" alt="" className="brand-icon" />
        <h1>dsh-launcher</h1>
      </div>
      <div className="toolbar-actions">
        <ThemeToggle />
        {/* 点击 -> 刷新中… -> 已刷新(短暂) -> 刷新;失败:✕ + 轻提醒 */}
        <AsyncButton
          task={onRefresh}
          idle="刷新"
          loading="刷新中…"
          success="已刷新"
          failurePrefix="刷新失败"
        />
        <button onClick={onManualAdd}>手动添加</button>
        <button className="primary" onClick={onInstall}>
          安装新版本
        </button>
      </div>
    </header>
  );
}
