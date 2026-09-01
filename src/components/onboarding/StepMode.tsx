// mode 步:快速(npm 最新稳定版,推荐)/ 自定义版本号;dev 折叠说明,向导不代装。

import { useState } from "react";
import { npmLatestVersion } from "../../api";
import { showFailure } from "../../hooks/toastStore";

interface Props {
  onNext: (version: string) => void;
}

export default function StepMode({ onNext }: Props) {
  const [busy, setBusy] = useState(false);

  const goQuick = async () => {
    setBusy(true);
    try {
      onNext(await npmLatestVersion());
    } catch (e) {
      showFailure("查询最新版本失败", e);
      setBusy(false);
    }
  };

  return (
    <div className="ob-step">
      <h2>选择安装方式</h2>
      <div className="ob-mode-card">
        <h3>快速安装(推荐)</h3>
        <p>自动安装 npm 上的最新稳定版,全程默认即可。</p>
        <button type="button" className="ob-primary" disabled={busy} onClick={goQuick}>
          {busy ? "查询最新版本…" : "一键安装最新稳定版"}
        </button>
      </div>
      <CustomVersionForm onNext={onNext} />
      <details className="ob-advanced">
        <summary>高级:开发版(dev repo)</summary>
        <p>
          开发版需要自行准备 git 与 pnpm 并检出仓库,向导不代装;
          完成向导后可在主界面「添加开发版」登记。
        </p>
      </details>
    </div>
  );
}

function CustomVersionForm({ onNext }: Props) {
  const [custom, setCustom] = useState("");

  const goCustom = () => {
    const v = custom.trim().replace(/^v/, "");
    if (!v) {
      showFailure("版本号", "请输入版本号,例如 0.1.1-rc.2");
      return;
    }
    onNext(v);
  };

  return (
    <div className="ob-mode-card">
      <h3>安装指定版本</h3>
      <div className="ob-row">
        <input
          value={custom}
          placeholder="例如 0.1.1-rc.2"
          onChange={(e) => setCustom(e.target.value)}
        />
        <button type="button" onClick={goCustom}>安装此版本</button>
      </div>
    </div>
  );
}