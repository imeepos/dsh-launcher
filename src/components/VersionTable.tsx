import { effectiveTool, type VersionEntry } from "../api";
import AsyncButton from "./AsyncButton";
import KindBadge from "./KindBadge";

type VersionRowProps = {
  version: VersionEntry;
  onFingerprint: (id: string) => Promise<void>;
  onDelete: (version: VersionEntry) => void;
  onRun: (version: VersionEntry) => void;
};

function VersionRow({ version, onFingerprint, onDelete, onRun }: VersionRowProps) {
  const isGeneric = effectiveTool(version) !== "dsh";
  return (
    <tr>
      <td>
        <div className="cell-main">{version.id}</div>
        {version.cwd && <div className="cell-sub mono">{version.cwd}</div>}
      </td>
      <td>
        <KindBadge kind={version.kind} />
      </td>
      <td className="mono">{effectiveTool(version)}</td>
      <td className="mono">{version.spec ?? version.bin}</td>
      <td className="mono">{version.fingerprint ?? "—"}</td>
      <td>
        <div className="row-actions">
          {isGeneric && (
            <button className="primary" onClick={() => onRun(version)}>
              运行
            </button>
          )}
          <AsyncButton
            task={() => onFingerprint(version.id)}
            idle="指纹"
            loading="采集中…"
            success="已采集"
            successToast={"已采集 " + version.id + " 指纹"}
            failurePrefix="采集失败"
          />
          <button className="danger" onClick={() => onDelete(version)}>
            删除
          </button>
        </div>
      </td>
    </tr>
  );
}

export default function VersionTable({
  versions,
  onFingerprint,
  onDelete,
  onRun,
}: {
  versions: VersionEntry[];
  onFingerprint: (id: string) => Promise<void>;
  onDelete: (v: VersionEntry) => void;
  onRun: (v: VersionEntry) => void;
}) {
  return (
    <table className="version-table">
      <thead>
        <tr>
          <th>ID</th>
          <th>类型</th>
          <th>工具</th>
          <th>spec / bin</th>
          <th>指纹</th>
          <th>操作</th>
        </tr>
      </thead>
      <tbody>
        {versions.length === 0 ? (
          <tr>
            <td colSpan={6} className="empty-cell">
              <div className="empty-state">
                <img src="/assets/dsh-empty-state.png" alt="" className="empty-state-art" />
                <p className="empty-state-title">还没有任何工具</p>
                <p className="hint">点「目录」从 release-platform 安装,或「手动添加」登记本地工具</p>
              </div>
            </td>
          </tr>
        ) : (
          versions.map((version) => (
            <VersionRow
              key={version.id}
              version={version}
              onFingerprint={onFingerprint}
              onDelete={onDelete}
              onRun={onRun}
            />
          ))
        )}
      </tbody>
    </table>
  );
}
