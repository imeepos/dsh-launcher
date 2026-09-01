import type { VersionEntry } from "../api";
import KindBadge from "./KindBadge";

export default function VersionTable({
  versions,
  busyId,
  onFingerprint,
  onDelete,
}: {
  versions: VersionEntry[];
  busyId: string | null;
  onFingerprint: (id: string) => void;
  onDelete: (v: VersionEntry) => void;
}) {
  return (
    <table className="version-table">
      <thead>
        <tr>
          <th>ID</th>
          <th>类型</th>
          <th>spec / bin</th>
          <th>cwd</th>
          <th>指纹</th>
          <th>操作</th>
        </tr>
      </thead>
      <tbody>
        {versions.length === 0 ? (
          <tr>
            <td colSpan={6} className="empty-cell">
              <div className="empty-state">
                <img src="/assets/dsh-empty-state.png" alt="" className="empty-state-art" />
                <p className="empty-state-title">还没有任何 DSH 版本</p>
                <p className="hint">点「安装新版本」从 npm 安装，或「手动添加」登记本地版本</p>
              </div>
            </td>
          </tr>
        ) : (
          versions.map((v) => (
            <tr key={v.id}>
              <td>{v.id}</td>
              <td>
                <KindBadge kind={v.kind} />
              </td>
              <td className="mono">{v.spec ?? v.bin}</td>
              <td className="mono">{v.cwd ?? "—"}</td>
              <td className="mono">{v.fingerprint ?? "未采集"}</td>
              <td>
                <div className="row-actions">
                  <button
                    onClick={() => onFingerprint(v.id)}
                    disabled={busyId !== null}
                  >
                    {busyId === v.id ? "采集中…" : "指纹"}
                  </button>
                  <button className="danger" onClick={() => onDelete(v)}>
                    删除
                  </button>
                </div>
              </td>
            </tr>
          ))
        )}
      </tbody>
    </table>
  );
}
