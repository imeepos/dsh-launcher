import useCatalog from "../../hooks/useCatalog";
import useCatalogInstall from "../../hooks/useCatalogInstall";
import PanelHeader from "../PanelHeader";
import ArtifactTable from "./ArtifactTable";
import ReleaseTable from "./ReleaseTable";
import RpSettingsForm from "./RpSettingsForm";

// 目录视图:release-platform 连接 → 发布浏览 → 制品安装(DESIGN-TOOLS.md §2)。
export default function CatalogPane({ refresh }: { refresh: () => Promise<void> }) {
  const catalog = useCatalog();
  const install = useCatalogInstall(refresh);
  return (
    <section className="pane">
      <PanelHeader
        title="目录"
        actions={
          <span className="hint">
            {catalog.connected ? "已连接 release-platform" : "未连接"}
          </span>
        }
      />
      <div className="pane-body">
        <RpSettingsForm
          cfg={catalog.cfg}
          busy={catalog.busy}
          connected={catalog.connected}
          onConnect={(baseUrl, auth) => void catalog.connect(baseUrl, auth)}
          onSave={(baseUrl, auth) => void catalog.connect(baseUrl, auth)}
        />
        {catalog.error && <p className="error">{catalog.error}</p>}
        {catalog.connected && (
          <ReleaseTable
            releases={catalog.releases}
            selectedVersion={catalog.selectedVersion}
            busy={catalog.loadingArtifacts ? "artifacts" : null}
            onSelect={(versionId) => void catalog.loadArtifacts(versionId)}
          />
        )}
        {catalog.connected && catalog.selectedVersion && (
          <ArtifactTable
            versionId={catalog.selectedVersion}
            artifacts={catalog.artifacts}
            installing={install.installing}
            onInstall={(id, tool, semver) => install.install(id, tool, semver)}
          />
        )}
        {install.lines.length > 0 && (
          <pre className="install-log">{install.lines.join("\n")}</pre>
        )}
        {install.err && <p className="error">{install.err}</p>}
      </div>
    </section>
  );
}
