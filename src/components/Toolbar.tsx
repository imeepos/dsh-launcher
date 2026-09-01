import ThemeToggle from "./ThemeToggle";

// 顶栏只承载全局项:品牌 + 主题切换;业务操作在各面板头部。
export default function Toolbar() {
  return (
    <header className="topbar">
      <div className="brand">
        <img src="/assets/dsh-launcher-icon.png" alt="" className="brand-icon" />
        <h1>dsh-launcher</h1>
      </div>
      <div className="topbar-actions">
        <ThemeToggle />
      </div>
    </header>
  );
}
