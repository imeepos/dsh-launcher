import type { ProfileInfo } from "../api";
import ProfileRow from "./ProfileRow";

interface Props {
  profiles: ProfileInfo[];
  homeId: string | null;
  running: string[];
  busy: boolean;
  onStart: (p: ProfileInfo) => void;
  onStop: (p: ProfileInfo) => void;
}

function ProfilesTable({ profiles, homeId, running, busy, onStart, onStop }: Props) {
  return (
    <table className="version-table">
      <thead>
        <tr>
          <th>Profile</th>
          <th>Bundles</th>
          <th>状态</th>
          <th>操作</th>
        </tr>
      </thead>
      <tbody>
        {!homeId ? (
          <tr>
            <td colSpan={4} className="empty">
              先选择一个 home
            </td>
          </tr>
        ) : profiles.length === 0 ? (
          <tr>
            <td colSpan={4} className="empty">
              该 home 还没有 profile(启动一次 dsh 后会生成 profiles/&lt;name&gt;)
            </td>
          </tr>
        ) : (
          profiles.map((info) => (
            <ProfileRow
              key={info.name}
              info={info}
              isRunning={running.includes(homeId + "/" + info.name)}
              busy={busy}
              onStart={onStart}
              onStop={onStop}
            />
          ))
        )}
      </tbody>
    </table>
  );
}

export default ProfilesTable;
