import { useCallback, useEffect, useState } from "react";
import {
  rpConnect,
  rpGetConfig,
  rpSetConfig,
  type RpAuth,
  type RpSettings,
} from "../rp-api";

// release-platform 连接状态:配置读写与连接测试(密码模式在后端换 JWT 驻内存)。
function useRpConnection() {
  const [cfg, setCfg] = useState<RpSettings | null>(null);
  const [connected, setConnected] = useState(false);
  const [busy, setBusy] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    void rpGetConfig()
      .then(setCfg)
      .catch(() => setCfg(null));
  }, []);

  const save = useCallback(async (baseUrl: string, auth: RpAuth | null) => {
    setBusy("save");
    setError(null);
    try {
      setCfg(await rpSetConfig(baseUrl, auth));
      return true;
    } catch (e) {
      setError(String(e));
      return false;
    } finally {
      setBusy(null);
    }
  }, []);

  /** 返回是否成功;状态由本 hook 内部维护,调用方无需读旧值 */
  const connect = useCallback(
    async (baseUrl: string, auth: RpAuth | null): Promise<boolean> => {
      setBusy("connect");
      setError(null);
      try {
        if (!(await save(baseUrl, auth))) return false;
        await rpConnect();
        setConnected(true);
        return true;
      } catch (e) {
        setConnected(false);
        setError(String(e));
        return false;
      } finally {
        setBusy(null);
      }
    },
    [save],
  );

  return { cfg, connected, busy, error, save, connect };
}

export default useRpConnection;
