import { useCallback, useEffect, useState } from "react";
import { listProfiles, type ProfileInfo } from "../api";

// Discovers profiles for one home path; empty homePath yields an empty list.
function useProfiles(homePath?: string) {
  const [profiles, setProfiles] = useState<ProfileInfo[]>([]);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    if (!homePath) {
      setProfiles([]);
      return;
    }
    try {
      setProfiles(await listProfiles(homePath));
      setError(null);
    } catch (e) {
      setError(String(e));
    }
  }, [homePath]);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  return { profiles, error, refresh };
}

export default useProfiles;
