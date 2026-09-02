export type LauncherView = "launch" | "versions" | "homes" | "catalog";

const TABS: { id: LauncherView; label: string }[] = [
  { id: "launch", label: "启动台" },
  { id: "versions", label: "工具库" },
  { id: "homes", label: "DSH Homes" },
  { id: "catalog", label: "目录" },
];

// 顶栏视图切换:选中态 = 抬升底板 + 蓝字 + 底部 3px 指示条。
export default function ViewTabs({
  view,
  onView,
}: {
  view: LauncherView;
  onView: (v: LauncherView) => void;
}) {
  return (
    <nav className="view-tabs" aria-label="视图切换">
      {TABS.map((t) => (
        <button
          key={t.id}
          className={"view-tab" + (view === t.id ? " active" : "")}
          aria-pressed={view === t.id}
          onClick={() => onView(t.id)}
        >
          {t.label}
        </button>
      ))}
    </nav>
  );
}
