import { useState } from "react";
import type { RpAuth, RpAuthMode, RpSettings } from "../../rp-api";

export interface RpFormValues {
  token: string;
  issuerUrl: string;
  username: string;
  password: string;
  tenant: string;
  subject: string;
}

const EMPTY: RpFormValues = {
  token: "", issuerUrl: "", username: "", password: "", tenant: "", subject: "",
};

const keep = (edited: string, saved?: string | null) =>
  edited.trim() ? edited.trim() : saved ?? null;

export function buildRpAuth(
  mode: RpAuthMode,
  values: RpFormValues,
  saved: RpFormValues,
): RpAuth {
  if (mode === "bearer") return { mode, token: keep(values.token, saved.token) };
  if (mode === "devheaders")
    return { mode, tenant: keep(values.tenant, saved.tenant), subject: keep(values.subject, saved.subject) };
  return {
    mode,
    issuerUrl: keep(values.issuerUrl, saved.issuerUrl),
    username: keep(values.username, saved.username),
    password: keep(values.password, saved.password),
  };
}

// 表单状态机:编辑值 + 由已存配置合并出提交用 RpAuth(留空 = 保留已存)。
export function useRpSettingsForm(cfg: RpSettings | null) {
  const auth = cfg?.auth ?? null;
  const [baseUrl, setBaseUrl] = useState(cfg?.baseUrl ?? "http://192.168.0.102:38080");
  const [mode, setMode] = useState<RpAuthMode>(auth?.mode ?? "password");
  const [values, setValues] = useState<RpFormValues>({
    ...EMPTY,
    issuerUrl: auth?.issuerUrl ?? "http://192.168.0.102:38086/oauth",
    username: auth?.username ?? "",
    tenant: auth?.tenant ?? "",
  });
  const patch = (p: Partial<RpFormValues>) => setValues((v) => ({ ...v, ...p }));
  const saved: RpFormValues = {
    ...EMPTY,
    token: auth?.token ?? "",
    issuerUrl: auth?.issuerUrl ?? "",
    username: auth?.username ?? "",
    password: auth?.password ?? "",
    tenant: auth?.tenant ?? "",
    subject: auth?.subject ?? "",
  };
  return { baseUrl, setBaseUrl, mode, setMode, values, patch, saved, submitAuth: buildRpAuth(mode, values, saved) };
}

function PasswordFields({
  values,
  onChange,
}: {
  values: RpFormValues;
  onChange: (p: Partial<RpFormValues>) => void;
}) {
  return (
    <>
      <label>
        Issuer URL
        <input
          value={values.issuerUrl}
          onChange={(e) => onChange({ issuerUrl: e.target.value })}
          placeholder="http://192.168.0.102:38086/oauth"
        />
      </label>
      <label>
        用户名
        <input
          value={values.username}
          onChange={(e) => onChange({ username: e.target.value })}
          autoComplete="username"
        />
      </label>
      <label>
        密码
        <input
          type="password"
          value={values.password}
          onChange={(e) => onChange({ password: e.target.value })}
          autoComplete="current-password"
        />
      </label>
    </>
  );
}

// 认证字段按模式渲染;凭据留空 = 保留已存值(提交时由 buildRpAuth 合并)。
export default function RpAuthFields({
  mode,
  values,
  onChange,
}: {
  mode: RpAuthMode;
  values: RpFormValues;
  onChange: (p: Partial<RpFormValues>) => void;
}) {
  if (mode === "bearer") {
    return (
      <label>
        Token
        <input
          type="password"
          value={values.token}
          onChange={(e) => onChange({ token: e.target.value })}
          placeholder="rpat_…"
        />
      </label>
    );
  }
  if (mode === "devheaders") {
    return (
      <>
        <label>
          X-Tenant-ID
          <input value={values.tenant} onChange={(e) => onChange({ tenant: e.target.value })} />
        </label>
        <label>
          X-Subject
          <input value={values.subject} onChange={(e) => onChange({ subject: e.target.value })} />
        </label>
      </>
    );
  }
  return <PasswordFields values={values} onChange={onChange} />;
}
