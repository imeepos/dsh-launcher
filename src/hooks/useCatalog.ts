import { useCallback, useState } from "react";
import { rpListArtifacts, rpListReleases, type RpAuth, type RpRecord } from "../rp-api";
import useRpConnection from "./useRpConnection";

// 目录面板:连接状态 + 发布/制品两级浏览(DESIGN-TOOLS.md §2)。
function useCatalog() {
  const conn = useRpConnection();
  const [releases, setReleases] = useState<RpRecord[]>([]);
  const [artifacts, setArtifacts] = useState<RpRecord[]>([]);
  const [selectedVersion, setSelectedVersion] = useState<string | null>(null);
  const [loadingArtifacts, setLoadingArtifacts] = useState(false);
  const [browseError, setBrowseError] = useState<string | null>(null);

  const connect = useCallback(
    async (baseUrl: string, auth: RpAuth | null) => {
      setBrowseError(null);
      const ok = await conn.connect(baseUrl, auth);
      if (!ok) return;
      try {
        setReleases(await rpListReleases(null, null, 50));
      } catch (e) {
        setBrowseError(String(e));
      }
    },
    [conn],
  );

  const loadArtifacts = useCallback(async (versionId: string) => {
    setBrowseError(null);
    setSelectedVersion(versionId);
    setLoadingArtifacts(true);
    try {
      setArtifacts(await rpListArtifacts(versionId, null, null));
    } catch (e) {
      setBrowseError(String(e));
    } finally {
      setLoadingArtifacts(false);
    }
  }, []);

  return {
    cfg: conn.cfg,
    connected: conn.connected,
    busy: conn.busy,
    error: conn.error ?? browseError,
    releases,
    artifacts,
    selectedVersion,
    loadingArtifacts,
    connect,
    loadArtifacts,
  };
}

export default useCatalog;
