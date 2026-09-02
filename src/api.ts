// Tauri command 的类型化封装。字段与 src-tauri/src/registry.rs 的 serde 命名一致(camelCase)。

export * from "./wizard-api";

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export type VersionKind = "npm" | "dev" | "manual";

export interface VersionEntry {
  id: string;
  kind: VersionKind;
  spec?: string | null;
  bin: string;
  cwd?: string | null;
  fingerprint?: string | null;
  addedAtMs?: number | null;
  /** 工具归属名;缺省视为 dsh(DESIGN-TOOLS.md §1) */
  tool?: string | null;
}

/** 通用工具运行键的 home 段 */
export const TOOL_HOME_ID = "__tool__";

export function effectiveTool(v: VersionEntry): string {
  const t = v.tool?.trim();
  return t ? t : "dsh";
}

export interface InstallProgressPayload {
  id: string;
  line: string;
}

export function listVersions(): Promise<VersionEntry[]> {
  return invoke("list_versions");
}

export function addManualVersion(
  bin: string,
  cwd: string | null,
  id: string | null,
  tool: string | null,
): Promise<VersionEntry> {
  return invoke("add_manual_version", { bin, cwd, id, tool });
}

export function fingerprintVersion(id: string): Promise<VersionEntry> {
  return invoke("fingerprint_version", { id });
}

export function installNpmVersion(
  version: string,
  id: string | null,
): Promise<VersionEntry> {
  return invoke("install_npm_version", { version, id });
}

export function addDevVersion(
  repoPath: string,
  id: string | null,
): Promise<VersionEntry> {
  return invoke("add_dev_version", { repoPath, id });
}

export function removeVersion(id: string): Promise<void> {
  return invoke("remove_version", { id });
}

export interface HomeEntry {
  id: string;
  path: string;
  boundVersionId?: string | null;
  lastGoodVersionId?: string | null;
}

export function listHomes(): Promise<HomeEntry[]> {
  return invoke("list_homes");
}

export function addHome(path: string, id: string | null): Promise<HomeEntry> {
  return invoke("add_home", { path, id });
}

export function createHome(path: string | null, id: string | null): Promise<HomeEntry> {
  return invoke("create_home", { path, id });
}

export function cloneHome(
  sourceId: string,
  newPath: string | null,
  newId: string | null,
): Promise<HomeEntry> {
  return invoke("clone_home", { sourceId, newPath, newId });
}

export function bindHomeVersion(homeId: string, versionId: string | null): Promise<HomeEntry> {
  return invoke("bind_home_version", { homeId, versionId });
}

export function removeHome(homeId: string): Promise<void> {
  return invoke("remove_home", { homeId });
}

export function onInstallProgress(
  handler: (p: InstallProgressPayload) => void,
): Promise<UnlistenFn> {
  return listen<InstallProgressPayload>("install-progress", (event) =>
    handler(event.payload),
  );
}

export interface ProfileInfo {
  name: string;
  bundleCount: number;
}

export interface ProcessLogPayload {
  homeId: string;
  profile: string;
  line: string;
  isErr: boolean;
}

export interface ProcessExitPayload {
  homeId: string;
  profile: string;
  exitCode?: number | null;
}

export function listProfiles(homePath: string): Promise<ProfileInfo[]> {
  return invoke("list_profiles", { homePath });
}

export function startProfile(
  homeId: string,
  profile: string,
  patch: string | null,
  args: string[] | null,
  cwd: string | null,
): Promise<void> {
  return invoke("start_profile", { homeId, profile, patch, args, cwd });
}

export function stopProfile(homeId: string, profile: string): Promise<void> {
  return invoke("stop_profile", { homeId, profile });
}

export function listRunning(): Promise<string[]> {
  return invoke("list_running");
}

export function onProcessLog(
  handler: (p: ProcessLogPayload) => void,
): Promise<UnlistenFn> {
  return listen<ProcessLogPayload>("process-log", (event) => handler(event.payload));
}

export function onProcessExit(
  handler: (p: ProcessExitPayload) => void,
): Promise<UnlistenFn> {
  return listen<ProcessExitPayload>("process-exit", (event) => handler(event.payload));
}

export * from "./rp-api";
