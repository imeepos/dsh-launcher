import type { HomeEntry, VersionEntry } from "../api";
import HomeRow from "./HomeRow";

interface Props {
  homes: HomeEntry[];
  versions: VersionEntry[];
  busy: boolean;
  onBind: (homeId: string, versionId: string | null) => void;
  onClone: (home: HomeEntry) => void;
  onDelete: (homeId: string) => void;
}

function HomesTable({ homes, versions, busy, onBind, onClone, onDelete }: Props) {
  return (
    <table className="version-table">
      <thead>
        <tr>
          <th>ID</th>
          <th>路径</th>
          <th>绑定版本</th>
          <th>操作</th>
        </tr>
      </thead>
      <tbody>
        {homes.length === 0 ? (
          <tr>
            <td colSpan={4} className="empty">
              还没有 home,点「新建」或「登记」
            </td>
          </tr>
        ) : (
          homes.map((h) => (
            <HomeRow
              key={h.id}
              home={h}
              versions={versions}
              busy={busy}
              onBind={(v) => onBind(h.id, v)}
              onClone={() => onClone(h)}
              onDelete={() => onDelete(h.id)}
            />
          ))
        )}
      </tbody>
    </table>
  );
}

export default HomesTable;
