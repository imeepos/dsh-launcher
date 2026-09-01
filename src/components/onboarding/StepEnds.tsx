// 向导首尾两屏(欢迎/完成)与顶部步骤条。

import type { OnboardingStepName } from "../../api";

export function WelcomeCard({ onStart }: { onStart: () => void }) {
  return (
    <div className="ob-step">
      <h2>欢迎使用 dsh 启动器</h2>
      <p>
        接下来会自动准备运行环境、安装 dsh 并完成首次启动,大约 5-10 分钟。
        全程不需要终端命令,出问题也只需要点按钮。
      </p>
      <button type="button" className="ob-primary" onClick={onStart}>开始</button>
    </div>
  );
}

export function DoneCard({ onFinish }: { onFinish: () => void }) {
  return (
    <div className="ob-step">
      <h2>全部完成 🎉</h2>
      <p>以后在这里一键启动 dsh。</p>
      <button type="button" className="ob-primary" onClick={onFinish}>进入主界面</button>
    </div>
  );
}

const CRUMBS = ["check", "fix", "mode", "install", "home", "launch"] as const;

export function StepCrumbs({ current }: { current: OnboardingStepName }) {
  return (
    <ol className="ob-crumbs">
      {CRUMBS.map((s) => (
        <li key={s} className={current === s ? "ob-crumb now" : "ob-crumb"}>{s}</li>
      ))}
    </ol>
  );
}