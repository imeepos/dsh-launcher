export default function Toolbar({
  onRefresh,
  onManualAdd,
  onInstall,
}: {
  onRefresh: () => void;
  onManualAdd: () => void;
  onInstall: () => void;
}) {
  return (
    <header className="toolbar">
      <h1>dsh-launcher</h1>
      <div className="toolbar-actions">
        <button onClick={onRefresh}>刷新</button>
        <button onClick={onManualAdd}>手动添加</button>
        <button className="primary" onClick={onInstall}>
          安装新版本
        </button>
      </div>
    </header>
  );
}
