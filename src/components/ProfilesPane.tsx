import { useState } from "react";
import useHomes from "../hooks/useHomes";
import useProcessOps from "../hooks/useProcessOps";
import useProfiles from "../hooks/useProfiles";
import PanelHeader from "./PanelHeader";
import ProfilesTable from "./ProfilesTable";
import StartControls from "./StartControls";

// Profiles 面板:选 home + 启动参数,卡片式 profile 列表。日志在底部停靠台。
export default function ProfilesPane() {
  const { homes } = useHomes();
  const [homeId, setHomeId] = useState("");
  const home = homes.find((h) => h.id === homeId);
  const { profiles, error: profileError, refresh } = useProfiles(home?.path);
  const { running, busy, error: opError, start, stop } = useProcessOps(refresh);
  const [patch, setPatch] = useState("");
  const [args, setArgs] = useState("");
  const [cwd, setCwd] = useState("");

  return (
    <section className="pane">
      <PanelHeader
        title="Profiles"
        actions={<button onClick={() => void refresh()}>刷新</button>}
      />
      <div className="pane-body">
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
      </div>
    </section>
  );
}
