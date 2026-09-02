import { useState } from "react";
import "./App.css";
import AppGate from "./components/AppGate";
import AppShell from "./components/AppShell";
import ConsoleDock from "./components/ConsoleDock";
import EnvStatusBar from "./components/EnvStatusBar";
import HomesPanel from "./components/HomesPanel";
import LaunchPad from "./components/LaunchPad";
import CatalogPane from "./components/catalog/CatalogPane";
import Toolbar from "./components/Toolbar";
import VersionsPane from "./components/VersionsPane";
import useVersions from "./hooks/useVersions";
import type { LauncherView } from "./components/ViewTabs";

// 页面容器:只做状态编排与视图装配。首跑向导未完成时由 AppGate 全屏接管。
// 三个视图保持挂载,切换只做显隐,保留各自的选中与表单状态。
function App() {
  const { versions, refresh } = useVersions();
  const [view, setView] = useState<LauncherView>("launch");
  return (
    <AppGate>
      <AppShell>
        <EnvStatusBar />
        <Toolbar view={view} onView={setView} />
        <LaunchPad hidden={view !== "launch"} />
        <div className="view" hidden={view !== "versions"}>
          <VersionsPane versions={versions} refresh={refresh} />
        </div>
        <div className="view" hidden={view !== "homes"}>
          <HomesPanel versions={versions} />
        </div>
        <div className="view" hidden={view !== "catalog"}>
          <CatalogPane refresh={refresh} />
        </div>
        <ConsoleDock />
      </AppShell>
    </AppGate>
  );
}

export default App;
