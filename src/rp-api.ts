// release-platform 目录与通用工具运行的 Tauri 命令封装(DESIGN-TOOLS.md §2)。
import { invoke } from "@tauri-apps/api/core";
import type { VersionEntry } from "./api";

export type RpAuthMode = "devheaders" | "bearer" | "password";

export interface RpAuth {
  mode: RpAuthMode;
  token?: string | null;
  issuerUrl?: string | null;
  username?: string | null;
  password?: string | null;
  tenant?: string | null;
  subject?: string | null;
}

export interface RpSettings {
  baseUrl: string;
  auth?: RpAuth | null;
}

export type RpRecord = Record<string, unknown>;

/** 通用工具运行键的 home 段 */
export const TOOL_HOME_ID = "__tool__";

export function rpGetConfig(): Promise<RpSettings> {
  return invoke("rp_get_config");
}

export function rpSetConfig(baseUrl: string, auth: RpAuth | null): Promise<RpSettings> {
  return invoke("rp_set_config", { baseUrl, auth });
}

export function rpConnect(): Promise<{ ok: boolean; baseUrl: string; products: number }> {
  return invoke("rp_connect");
}

export function rpListProducts(): Promise<RpRecord[]> {
  return invoke("rp_list_products");
}

export function rpListReleases(
  channel: string | null,
  status: string | null,
  limit: number | null,
): Promise<RpRecord[]> {
  return invoke("rp_list_releases", { channel, status, limit });
}

export function rpListArtifacts(
  versionId: string,
  os: string | null,
  arch: string | null,
): Promise<RpRecord[]> {
  return invoke("rp_list_artifacts", { versionId, os, arch });
}

export function rpInstallArtifact(
  artifactId: string,
  tool: string | null,
  semver: string | null,
): Promise<VersionEntry> {
  return invoke("rp_install_artifact", { artifactId, tool, semver });
}

/** 通用工具启动:返回运行键 __tool__/<versionId>;停止用 stopProfile(TOOL_HOME_ID, id) */
export function startTool(
  versionId: string,
  args: string[] | null,
  cwd: string | null,
): Promise<string> {
  return invoke("start_tool", { versionId, args, cwd });
}
