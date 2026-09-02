import ThemeToggle from "./ThemeToggle";
import ViewTabs, { type LauncherView } from "./ViewTabs";

// 顶栏:品牌 | 视图切换(启动台为默认重点) | 主题切换。
export default function Toolbar({
  view,
  onView,
}: {
  view: LauncherView;
  onView: (v: LauncherView) => void;
}) {
  return (
    <header className="topbar">
      <div className="brand">
        <img src="/assets/dsh-launcher-icon.png" alt="" className="brand-icon" />
        <h1>CLI 工具台</h1>
      </div>
      <ViewTabs view={view} onView={onView} />
      <div className="topbar-actions">
        <ThemeToggle />
      </div>
    </header>
  );
}
