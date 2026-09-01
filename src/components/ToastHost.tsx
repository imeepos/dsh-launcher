import { useSyncExternalStore } from "react";
import { dismissToast, getToasts, subscribeToasts, type ToastKind } from "../hooks/toastStore";

const KIND_ICON: Record<ToastKind, string> = { success: "✓", error: "✕" };

// 全局轻提醒宿主:右下角浮层,成功自动消失,失败可手动关闭。
export default function ToastHost() {
  const toasts = useSyncExternalStore(subscribeToasts, getToasts);
  if (toasts.length === 0) return null;
  return (
    <div className="toast-host" aria-live="polite">
      {toasts.map((t) => (
        <div key={t.id} className={"toast toast-" + t.kind} role="status">
          <span className="toast-icon" aria-hidden="true">
            {KIND_ICON[t.kind]}
          </span>
          <span className="toast-text">{t.text}</span>
          {t.kind === "error" && (
            <button
              type="button"
              className="toast-close"
              onClick={() => dismissToast(t.id)}
              aria-label="关闭提醒"
            >
              ✕
            </button>
          )}
        </div>
      ))}
    </div>
  );
}
