import type { RpRecord } from "../../api";

// 发布表:浏览主入口(releases 时间倒序);选行后拉取其制品。
export default function ReleaseTable({
  releases,
  selectedVersion,
  busy,
  onSelect,
}: {
  releases: RpRecord[];
  selectedVersion: string | null;
  busy: string | null;
  onSelect: (versionId: string) => void;
}) {
  return (
    <table className="version-table">
      <thead>
        <tr>
          <th>发布</th>
          <th>渠道</th>
          <th>状态</th>
          <th>版本</th>
          <th>创建时间</th>
          <th>操作</th>
        </tr>
      </thead>
      <tbody>
        {releases.length === 0 ? (
          <tr>
            <td colSpan={6} className="empty-cell">
              <p className="hint">平台暂无可见发布</p>
            </td>
          </tr>
        ) : (
          releases.map((rel) => {
            const id = String(rel.id ?? "");
            const versionId = String(rel.version_id ?? "");
            return (
              <tr key={id} className={selectedVersion === versionId ? "selected" : undefined}>
                <td className="mono">{id}</td>
                <td>{String(rel.channel ?? "—")}</td>
                <td>{String(rel.state ?? "—")}</td>
                <td className="mono">{versionId || "—"}</td>
                <td className="mono">{String(rel.created_at ?? "").slice(0, 19)}</td>
                <td>
                  <button disabled={busy !== null || !versionId} onClick={() => onSelect(versionId)}>
                    {busy === "artifacts" && selectedVersion === versionId ? "加载中…" : "制品"}
                  </button>
                </td>
              </tr>
            );
          })
        )}
      </tbody>
    </table>
  );
}
