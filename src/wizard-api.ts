// 首跑向导与修复中心的命令封装。独立文件守 api.ts 规模上限,经 api.ts 转发导出。

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export type CheckLevel = "blocker" | "warn" | "info";
export type CheckStatus = "pass" | "fail" | "skip";

export interface CheckItem {
  id: string;
  level: CheckLevel;
  status: CheckStatus;
  detail: string;
}

export type OnboardingStepName =
  | "welcome"
  | "check"
  | "fix"
  | "mode"
  | "install"
  | "home"
  | "launch"
  | "done";

export interface OnboardingState {
  step: OnboardingStepName;
  completed: boolean;
}

export interface RuntimeInfo {
  nodeVersion: string;
  bin: string;
  installedAtMs: number;
  sha256: string;
  source: string;
}

export function envCheck(): Promise<CheckItem[]> {
  return invoke("env_check");
}

export function envCheckFast(): Promise<CheckItem[]> {
  return invoke("env_check_fast");
}

export function onboardingGet(): Promise<OnboardingState> {
  return invoke("onboarding_get");
}

export function onboardingAdvance(step: OnboardingStepName): Promise<OnboardingState> {
  return invoke("onboarding_advance", { step });
}

export function onboardingComplete(): Promise<OnboardingState> {
  return invoke("onboarding_complete");
}

export function installRuntime(): Promise<RuntimeInfo> {
  return invoke("install_runtime");
}

export function repairRuntime(): Promise<RuntimeInfo> {
  return invoke("repair_runtime");
}

export function runtimeInfo(): Promise<RuntimeInfo | null> {
  return invoke("runtime_info");
}

export function npmLatestVersion(): Promise<string> {
  return invoke("npm_latest_version");
}

export function onRuntimeInstallLog(handler: (line: string) => void): Promise<UnlistenFn> {
  return listen<{ line: string }>("runtime-install-log", (e) => handler(e.payload.line));
}