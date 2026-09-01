import type { ProfileInfo } from "../api";

interface Props {
  info: ProfileInfo;
  isRunning: boolean;
  busy: boolean;
  onStart: (p: ProfileInfo) => void;
  onStop: (p: ProfileInfo) => void;
}

function ProfileRow({ info, isRunning, busy, onStart, onStop }: Props) {
  return (
    <tr>
      <td className="mono">{info.name}</td>
      <td>{info.bundleCount}</td>
      <td>{isRunning ? "运行中" : "已停止"}</td>
      <td>
        {isRunning ? (
          <button className="danger" onClick={() => onStop(info)} disabled={busy}>
            停止
          </button>
        ) : (
          <button className="primary" onClick={() => onStart(info)} disabled={busy}>
            启动
          </button>
        )}
      </td>
    </tr>
  );
}

export default ProfileRow;
