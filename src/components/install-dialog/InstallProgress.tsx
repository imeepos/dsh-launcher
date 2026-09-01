// npm 安装的估算进度条:按日志行数与耗时爬行推进,完成补满 100%。
export default function InstallProgress({
  percent,
  visible,
}: {
  percent: number;
  visible: boolean;
}) {
  if (!visible) return null;
  return (
    <div className="progress-block">
      <div className="progress-meta">
        <span>正在安装…</span>
        <span className="mono">{percent}%</span>
      </div>
      <div
        className="progress-track"
        role="progressbar"
        aria-valuenow={percent}
        aria-valuemin={0}
        aria-valuemax={100}
      >
        <div className="progress-bar" style={{ width: percent + "%" }} />
      </div>
      <p className="hint">npm 不提供精确进度,此进度条按安装日志推进估算</p>
    </div>
  );
}
