import type { ReactNode } from "react";
import ToastHost from "./ToastHost";

// 应用外壳:布局框 + 轻提醒装配点。后续三栏树/状态栏挂在这里扩展。
export default function AppShell({ children }: { children: ReactNode }) {
  return (
    <main className="container app-shell">
      {children}
      <ToastHost />
    </main>
  );
}
