import type { InstallKind } from "../../hooks/useInstallRunner";

export function submitLabel(running: boolean, err: string | null, kind: InstallKind): string {
  if (running) return kind === "npm" ? "安装中…" : "登记中…";
  if (err) return "重试";
  return kind === "npm" ? "安装" : "登记";
}
