// install 步:npm 安装所选版本;进度走 useCrawlProgress 爬行估算(封顶 90%,成功补满)。

import { useCallback, useState } from "react";
import { installNpmVersion, onInstallProgress } from "../../api";
import useCrawlProgress from "../../hooks/useCrawlProgress";
import { showFailure } from "../../hooks/toastStore";
import Spinner from "../Spinner";

interface Props {
  version: string;
  onDone: (versionId: string) => void;
}

export default function StepInstall({ version, onDone }: Props) {
  const [busy, setBusy] = useState(false);
  const [failed, setFailed] = useState(false);
  // 后端进度事件的 id 规则与 install_npm_version 一致:显式 id ?? `v${version}`。
  const expectedId = `v${version}`;
  const subscribe = useCallback(
    (cb: (line: string) => void) =>
      onInstallProgress((p) => {
        if (p.id === expectedId) cb(p.line);
      }),
    [expectedId],
  );
  const { lines, percent, reset, finish } = useCrawlProgress({ subscribe, match: () => true });

  const doInstall = async () => {
    if (busy) return;
    setBusy(true);
    setFailed(false);
    reset();
    try {
      const entry = await installNpmVersion(version, null);
      finish();
      onDone(entry.id);
    } catch (e) {
      setFailed(true);
      showFailure("安装失败", e);
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="ob-step">
      <h2>安装 dsh {version}</h2>
      {!busy && (
        <button type="button" className="ob-primary" onClick={doInstall}>
          开始安装
        </button>
      )}
      {busy && <InstallBusyView percent={percent} />}
      {failed && <p className="ob-err">安装失败,可点击重试;反复失败请检查网络。</p>}
      {lines.length > 0 && <pre className="ob-log">{lines.slice(-8).join("\n")}</pre>}
    </div>
  );
}

function InstallBusyView({ percent }: { percent: number }) {
  return (
    <div className="ob-center">
      <Spinner />
      <div className="ob-progress">
        <div className="ob-progress-bar" style={{ width: `${percent}%` }} />
      </div>
      <p className="ob-muted">正在安装,预计 1-3 分钟(进度为估算)…</p>
    </div>
  );
}