import type { RpAuth, RpSettings } from "../../rp-api";
import RpAuthFields, { useRpSettingsForm } from "./RpAuthFields";

// 连接配置表单:baseUrl + 三种认证模式;「连接测试」先保存再连接。
export default function RpSettingsForm({
  cfg,
  busy,
  connected,
  onConnect,
  onSave,
}: {
  cfg: RpSettings | null;
  busy: string | null;
  connected: boolean;
  onConnect: (baseUrl: string, auth: RpAuth | null) => void;
  onSave: (baseUrl: string, auth: RpAuth | null) => void;
}) {
  const form = useRpSettingsForm(cfg);
  return (
    <fieldset className="rp-settings" disabled={busy !== null}>
      <legend>release-platform 连接</legend>
      <label>
        服务地址
        <input
          value={form.baseUrl}
          onChange={(e) => form.setBaseUrl(e.target.value)}
          placeholder="http://192.168.0.102:38080"
        />
      </label>
      <label>
        认证方式
        <select
          value={form.mode}
          onChange={(e) => form.setMode(e.target.value as typeof form.mode)}
        >
          <option value="password">账号密码(issuer password grant)</option>
          <option value="bearer">Bearer Token(rpat_ / JWT)</option>
          <option value="devheaders">Dev Headers(本地联调)</option>
        </select>
      </label>
      <RpAuthFields mode={form.mode} values={form.values} onChange={form.patch} />
      <div className="modal-actions">
        <button
          onClick={() => onSave(form.baseUrl, form.submitAuth)}
          disabled={busy !== null}
        >
          {busy === "save" ? "保存中…" : "仅保存"}
        </button>
        <button
          className="primary"
          onClick={() => onConnect(form.baseUrl, form.submitAuth)}
          disabled={busy !== null}
        >
          {busy === "connect" ? "连接中…" : connected ? "重连" : "连接测试"}
        </button>
      </div>
    </fieldset>
  );
}
