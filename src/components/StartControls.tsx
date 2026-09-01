import type { HomeEntry } from "../api";

interface Props {
  homes: HomeEntry[];
  homeId: string;
  onHome: (v: string) => void;
  patch: string;
  onPatch: (v: string) => void;
  args: string;
  onArgs: (v: string) => void;
  cwd: string;
  onCwd: (v: string) => void;
  error?: string | null;
}

function StartControls(p: Props) {
  return (
    <div className="start-controls">
      {p.error && <p className="error">{p.error}</p>}
      <select className="bind-select" value={p.homeId} onChange={(e) => p.onHome(e.target.value)}>
        <option value="">选择 home…</option>
        {p.homes.map((h) => (
          <option key={h.id} value={h.id}>
            {h.id}
          </option>
        ))}
      </select>
      <input value={p.patch} onChange={(e) => p.onPatch(e.target.value)} placeholder="--patch 文件(可选)" />
      <input
        value={p.args}
        onChange={(e) => p.onArgs(e.target.value)}
        placeholder="附加参数(可选,空格分隔)"
      />
      <input value={p.cwd} onChange={(e) => p.onCwd(e.target.value)} placeholder="工作区 cwd(可选)" />
    </div>
  );
}

export default StartControls;
