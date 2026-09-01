import { useState } from "react";
import useAutoScrollRef from "../hooks/useAutoScrollRef";
import useHomes from "../hooks/useHomes";
import useProcessLog from "../hooks/useProcessLog";
import useProcessOps from "../hooks/useProcessOps";
import useProfiles from "../hooks/useProfiles";
import PanelHeader from "./PanelHeader";
import ProcessLogConsole from "./ProcessLogConsole";
import ProfilesTable from "./ProfilesTable";
import StartControls from "./StartControls";

function ProcessPanel() {
  const { homes } = useHomes();
  const [homeId, setHomeId] = useState("");
  const home = homes.find((h) => h.id === homeId);
  const { profiles, error: profileError, refresh } = useProfiles(home?.path);
  const { running, busy, error: opError, start, stop } = useProcessOps(refresh);
  const { lines, clear } = useProcessLog();
  const logRef = useAutoScrollRef(lines);
  const [patch, setPatch] = useState("");
  const [args, setArgs] = useState("");
  const [cwd, setCwd] = useState("");

  return (
    <section className="homes-panel">
      <PanelHeader
        title="Profiles"
        actions={
          <>
            <button onClick={() => void refresh()}>刷新</button>
            <button onClick={clear}>清空日志</button>
          </>
        }
      />

      <StartControls
        homes={homes}
        homeId={homeId}
        onHome={setHomeId}
        patch={patch}
        onPatch={setPatch}
        args={args}
        onArgs={setArgs}
        cwd={cwd}
        onCwd={setCwd}
        error={opError ?? profileError}
      />

      <ProfilesTable
        profiles={profiles}
        homeId={homeId || null}
        running={running}
        busy={busy}
        onStart={(p) => home && start(home, p, patch, args, cwd)}
        onStop={(p) => home && stop(home, p)}
      />

      <ProcessLogConsole lines={lines} logRef={logRef} />
    </section>
  );
}

export default ProcessPanel;
