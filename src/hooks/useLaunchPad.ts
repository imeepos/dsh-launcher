import { useState } from "react";
import useHomes from "./useHomes";
import useProcessOps from "./useProcessOps";
import useProfiles from "./useProfiles";

// 启动台状态机:home 选择、profiles 发现、运行操作与启动选项,供 LaunchPad 渲染层消费。
function useLaunchPad() {
  const { homes, refresh, busy } = useHomes();
  const [homeId, setHomeId] = useState("");
  const home = homes.find((h) => h.id === homeId);
  const { profiles, error: profileError, refresh: refreshProfiles } = useProfiles(home?.path);
  const { running, busy: opBusy, error: opError, start, stop } = useProcessOps(refreshProfiles);
  const [optionsOpen, setOptionsOpen] = useState(false);
  const [opts, setOpts] = useState({ patch: "", args: "", cwd: "" });

  const refreshAll = () => {
    void refresh();
    void refreshProfiles();
  };

  return {
    homes, busy, homeId, setHomeId, home,
    profiles, error: opError ?? profileError,
    running, opBusy, start, stop,
    optionsOpen, setOptionsOpen, opts, setOpts,
    refreshAll,
  };
}

export default useLaunchPad;
