export type ToastKind = "success" | "error";

export interface Toast {
  id: number;
  kind: ToastKind;
  text: string;
}

const MAX_TOASTS = 4;
const AUTO_DISMISS_MS: Record<ToastKind, number> = { success: 2200, error: 4000 };

let toasts: Toast[] = [];
let nextId = 1;
const listeners = new Set<() => void>();

function emit(): void {
  for (const listener of listeners) listener();
}

// Tauri invoke rejects with plain strings; keep one formatting rule everywhere.
export function formatError(e: unknown): string {
  return typeof e === "string" ? e : String(e);
}

export function showToast(kind: ToastKind, text: string): void {
  const toast: Toast = { id: nextId++, kind, text };
  toasts = [...toasts, toast].slice(-MAX_TOASTS);
  emit();
  window.setTimeout(() => dismissToast(toast.id), AUTO_DISMISS_MS[kind]);
}

export function dismissToast(id: number): void {
  if (!toasts.some((t) => t.id === id)) return;
  toasts = toasts.filter((t) => t.id !== id);
  emit();
}

export function showSuccess(text: string): void {
  showToast("success", text);
}

// 失败轻提醒:统一入口,文案为「前缀: 原始错误」。
export function showFailure(prefix: string, e: unknown): void {
  showToast("error", prefix + ": " + formatError(e));
}

export function subscribeToasts(listener: () => void): () => void {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
}

export function getToasts(): Toast[] {
  return toasts;
}
