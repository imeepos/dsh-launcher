import { useCallback, useEffect, useState } from "react";
import { listHomes, type HomeEntry } from "../api";

// Owns the home list state; execute() wraps any home op with busy/error + refresh.
function useHomes() {
  const [homes, setHomes] = useState<HomeEntry[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const refresh = useCallback(async () => {
    try {
      setHomes(await listHomes());
      setError(null);
    } catch (e) {
      setError(String(e));
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const execute = useCallback(
    async (action: () => Promise<unknown>) => {
      setBusy(true);
      setError(null);
      try {
        await action();
        await refresh();
      } catch (e) {
        setError(String(e));
      } finally {
        setBusy(false);
      }
    },
    [refresh],
  );

  return { homes, error, busy, refresh, execute };
}

export default useHomes;
