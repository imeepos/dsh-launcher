import "./App.css";
import AppShell from "./components/AppShell";
import ConsoleDock from "./components/ConsoleDock";
import HomesPanel from "./components/HomesPanel";
import ProfilesPane from "./components/ProfilesPane";
import Toolbar from "./components/Toolbar";
import VersionsPane from "./components/VersionsPane";
import useVersions from "./hooks/useVersions";

// 页面容器:只做状态编排与布局装配。
function App() {
  const { versions, refresh } = useVersions();
  return (
    <AppShell>
      <Toolbar />
      <div className="workspace">
        <VersionsPane versions={versions} refresh={refresh} />
        <HomesPanel versions={versions} />
        <ProfilesPane />
      </div>
      <ConsoleDock />
    </AppShell>
  );
}

export default App;
