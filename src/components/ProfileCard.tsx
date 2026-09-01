import type { ProfileInfo } from "../api";

interface Props {
  info: ProfileInfo;
  isRunning: boolean;
  busy: boolean;
  onStart: () => void;
  onStop: () => void;
}

// 运行大卡:名称 + 状态点在上,主按钮独占底部;运行中绿描边 + 渐变,按钮切换为红色停止。
export default function ProfileCard({ info, isRunning, busy, onStart, onStop }: Props) {
  return (
    <div className={"profile-card" + (isRunning ? " running" : "")}>
      <div className="profile-card-head">
        <span className="profile-name">{info.name}</span>
        <span
          className={"status-dot " + (isRunning ? "dot-running" : "dot-stopped")}
          aria-hidden
        />
      </div>
      <p className="profile-meta">
        {isRunning ? "运行中" : "已停止"} · {info.bundleCount} 个 bundle
      </p>
      <button
        className={"run-btn " + (isRunning ? "stop" : "primary")}
        onClick={isRunning ? onStop : onStart}
        disabled={busy}
      >
        {isRunning ? "停止" : "启动"}
      </button>
    </div>
  );
}
