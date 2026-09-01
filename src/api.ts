// Tauri command 的类型化封装。字段与 src-tauri/src/registry.rs 的 serde 命名一致(camelCase)。

import { invoke } from "@tauri-apps/api/core";

export type VersionKind = "npm" | "dev" | "manual";

export interface VersionEntry {
  id: string;
  kind: VersionKind;
  spec?: string | null;
  bin: string;
  cwd?: string | null;
  fingerprint?: string | null;
  addedAtMs?: number | null;
}

export function listVersions(): Promise<VersionEntry[]> {
  return invoke("list_versions");
}

export function addManualVersion(
  bin: string,
  cwd: string | null,
  id: string | null,
): Promise<VersionEntry> {
  return invoke("add_manual_version", { bin, cwd, id });
}

export function fingerprintVersion(id: string): Promise<VersionEntry> {
  return invoke("fingerprint_version", { id });
}
