import type { InstallKind } from "../../hooks/useInstallRunner";

export interface SubmitLabels {
  idle: string;
  busy: string;
}

export function submitLabels(err: string | null, kind: InstallKind): SubmitLabels {
  const idle = err ? "重试" : kind === "npm" ? "安装" : "登记";
  return { idle, busy: kind === "npm" ? "安装中…" : "登记中…" };
}
