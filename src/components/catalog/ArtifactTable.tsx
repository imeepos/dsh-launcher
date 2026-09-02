import { useState } from "react";
import { localPlatformHint } from "./catalogUtil";

type Props = {
  versionId: string;
  artifacts: Record<string, unknown>[];
  installing: string | null;
  onInstall: (artifactId: string, tool: string | null, semver: string | null) => void;
};

function ArtifactRows({
  artifacts,
  installing,
  onInstall,
}: {
  artifacts: Record<string, unknown>[];
  installing: string | null;
  onInstall: (artifactId: string) => void;
}) {
  return (
    <>
      {artifacts.length === 0 ? (
        <tr>
          <td colSpan={6} className="empty-cell">
            <p className="hint">该版本没有匹配的制品</p>
          </td>
        </tr>
      ) : (
        artifacts.map((a) => {
          const id = String(a.id ?? "");
          const size = Number(a.size_bytes ?? 0);
          const sizeText = size > 0 ? (size / 1024).toFixed(1) + " KB" : "—";
          return (
            <tr key={id}>
              <td className="mono">{id}</td>
              <td>{String(a.os ?? "—")}</td>
              <td>{String(a.arch ?? "—")}</td>
              <td className="mono">{sizeText}</td>
              <td className="mono">{String(a.sha256 ?? "").slice(0, 12) || "—"}</td>
              <td>
                <button
                  className="primary"
                  disabled={installing !== null}
                  onClick={() => onInstall(id)}
                >
                  {installing === id ? "安装中…" : "安装"}
                </button>
              </td>
            </tr>
          );
        })
      )}
    </>
  );
}

// 制品表:按 os/arch 展示;安装时工具名默认用版本 id。
export default function ArtifactTable({ versionId, artifacts, installing, onInstall }: Props) {
  const [tool, setTool] = useState("");
  const installWithTool = (id: string) => onInstall(id, tool.trim() || null, versionId);
  return (
    <div>
      <p className="hint">
        版本 {versionId} 的制品(本机 {localPlatformHint()}) — 安装为工具:
        <input
          value={tool}
          onChange={(e) => setTool(e.target.value)}
          placeholder="工具名,留空用版本 id"
          style={{ width: 180, marginLeft: 6 }}
        />
      </p>
      <table className="version-table">
        <thead>
          <tr>
            <th>制品</th>
            <th>OS</th>
            <th>Arch</th>
            <th>大小</th>
            <th>sha256</th>
            <th>操作</th>
          </tr>
        </thead>
        <tbody>
          <ArtifactRows
            artifacts={artifacts}
            installing={installing}
            onInstall={(id) => installWithTool(id)}
          />
        </tbody>
      </table>
    </div>
  );
}
