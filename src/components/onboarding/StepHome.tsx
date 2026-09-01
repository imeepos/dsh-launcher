// home 步:默认 ~/.dsh + profile main;创建后绑定刚安装的版本。

import { useState } from "react";
import { bindHomeVersion, createHome, type HomeEntry } from "../../api";
import { showFailure } from "../../hooks/toastStore";

interface Props {
  versionId: string | null;
  onDone: (home: HomeEntry, profile: string) => void;
}

export default function StepHome({ versionId, onDone }: Props) {
  const [profile, setProfile] = useState("main");
  const [busy, setBusy] = useState(false);

  const create = async () => {
    if (busy) return;
    setBusy(true);
    try {
      const home = await createHome("~/.dsh", "main");
      if (versionId) await bindHomeVersion(home.id, versionId);
      onDone(home, profile.trim() || "main");
    } catch (e) {
      showFailure("创建 home 失败", e);
      setBusy(false);
    }
  };

  return (
    <div className="ob-step">
      <h2>创建工作目录(home)</h2>
      <p>
        home 是 dsh 存放配置与数据的地方。默认创建 <code>~/.dsh</code>;
        如果你以前用过 dsh,这里的已有目录也会被自动识别。
      </p>
      <label className="ob-row">
        profile 名称
        <input value={profile} onChange={(e) => setProfile(e.target.value)} />
      </label>
      <button type="button" className="ob-primary" disabled={busy} onClick={create}>
        {busy ? "创建中…" : "创建并继续"}
      </button>
      {versionId === null && (
        <p className="ob-muted">未选择版本,之后可在主界面手动绑定。</p>
      )}
    </div>
  );
}