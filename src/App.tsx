import "./App.css";
import AppShell from "./components/AppShell";
import ConsoleDock from "./components/ConsoleDock";
import HomesPanel from "./components/HomesPanel";
import ProfilesPane from "./components/ProfilesPane";
import Toolbar from "./components/Toolbar";
import VersionsPane from "./components/VersionsPane";
import useVersions from "./hooks/useVersions";
import AppGate from "./components/AppGate";
import EnvStatusBar from "./components/EnvStatusBar";

// 页面容器:只做状态编排与布局装配。首跑向导未完成时由 AppGate 全屏接管。
function App() {
  const { versions, refresh } = useVersions();
  return (
    <AppGate>
      <AppShell>
        <EnvStatusBar />
        <Toolbar />
        <div className="workspace">
          <VersionsPane versions={versions} refresh={refresh} />
          <HomesPanel versions={versions} />
          <ProfilesPane />
        </div>
        <ConsoleDock />
      </AppShell>
    </AppGate>
  );
}

export default App;