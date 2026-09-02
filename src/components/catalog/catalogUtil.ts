// 目录面板的小工具:本机平台提示文案。

function platformOs(): string {
  const p = navigator.platform.toLowerCase();
  if (p.includes("mac") || p.includes("darwin")) return "darwin";
  if (p.includes("linux")) return "linux";
  return p || "unknown";
}

function platformArch(): string {
  const p = navigator.platform.toLowerCase();
  if (p.includes("arm") || p.includes("aarch64")) return "arm64";
  return "amd64";
}

export function localPlatformHint(): string {
  return platformOs() + "/" + platformArch();
}
