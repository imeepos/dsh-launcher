import type { HomeEntry, VersionEntry } from "../api";

interface Props {
  home: HomeEntry;
  versions: VersionEntry[];
  busy: boolean;
  onBind: (versionId: string | null) => void;
  onClone: () => void;
  onDelete: () => void;
}

function HomeRow({ home, versions, busy, onBind, onClone, onDelete }: Props) {
  return (
    <tr>
      <td>
        <div className="cell-main">{home.id}</div>
        {home.lastGoodVersionId && (
          <div className="cell-sub mono">上次成功 {home.lastGoodVersionId}</div>
        )}
      </td>
      <td className="mono">{home.path}</td>
      <td>
        <select
          className="bind-select"
          value={home.boundVersionId ?? ""}
          onChange={(e) => onBind(e.target.value || null)}
          disabled={busy}
        >
          <option value="">未绑定</option>
          {versions.map((v) => (
            <option key={v.id} value={v.id}>
              {v.id}
            </option>
          ))}
        </select>
      </td>
      <td>
        <div className="row-actions">
          <button onClick={onClone} disabled={busy}>
            克隆
          </button>
          <button className="danger" onClick={onDelete} disabled={busy}>
            删除
          </button>
        </div>
      </td>
    </tr>
  );
}

export default HomeRow;
