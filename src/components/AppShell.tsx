import type { ReactNode } from "react";
import ToastHost from "./ToastHost";

// 应用外壳:顶栏 + 工作区 + 停靠台的布局框,轻提醒装配点。
export default function AppShell({ children }: { children: ReactNode }) {
  return (
    <main className="shell">
      {children}
      <ToastHost />
    </main>
  );
}
