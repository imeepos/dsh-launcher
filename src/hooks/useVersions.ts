import { useCallback, useEffect, useState } from "react";
import { fingerprintVersion, listVersions, type VersionEntry } from "../api";

// Owns the version list state shared by every dialog flow in App.
function useVersions() {
  const [versions, setVersions] = useState<VersionEntry[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [busyId, setBusyId] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    try {
      setVersions(await listVersions());
      setError(null);
    } catch (e) {
      setError(String(e));
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  async function doFingerprint(id: string) {
    setBusyId(id);
    setError(null);
    try {
      await fingerprintVersion(id);
      await refresh();
    } catch (e) {
      setError(String(e));
    } finally {
      setBusyId(null);
    }
  }

  return { versions, error, busyId, refresh, doFingerprint };
}

export default useVersions;
