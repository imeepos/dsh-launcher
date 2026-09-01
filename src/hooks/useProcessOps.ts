import { useCallback, useState } from "react";
import { startProfile, stopProfile, type HomeEntry, type ProfileInfo } from "../api";
import useRunningKeys from "./useRunningKeys";

// Start/stop orchestration on top of running-key state.
function useProcessOps(refreshProfiles: () => Promise<void>) {
  const { running, refresh: refreshRunning, markRunning, unmarkRunning } = useRunningKeys();
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const start = useCallback(
    (home: HomeEntry, p: ProfileInfo, patch: string, args: string, cwd: string) => {
      const key = home.id + "/" + p.name;
      const argList = args.split(/\s+/).filter(Boolean);
      setBusy(true);
      setError(null);
      markRunning(key);
      startProfile(home.id, p.name, patch.trim() || null, argList.length ? argList : null, cwd.trim() || null)
        .then(() => refreshProfiles())
        .catch((e) => {
          setError(String(e));
          unmarkRunning(key);
        })
        .finally(() => setBusy(false));
    },
    [refreshProfiles, markRunning, unmarkRunning],
  );

  const stop = useCallback(
    (home: HomeEntry, p: ProfileInfo) => {
      setBusy(true);
      setError(null);
      stopProfile(home.id, p.name)
        .then(() => refreshRunning())
        .catch((e) => setError(String(e)))
        .finally(() => setBusy(false));
    },
    [refreshRunning],
  );

  return { running, busy, error, start, stop };
}

export default useProcessOps;
