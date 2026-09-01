import type { InstallKind } from "../../hooks/useInstallRunner";

export default function KindPicker({
  kind,
  onSelect,
  disabled,
}: {
  kind: InstallKind;
  onSelect: (kind: InstallKind) => void;
  disabled: boolean;
}) {
  return (
    <div className="kind-picker">
      <button
        type="button"
        className={kind === "npm" ? "kind-btn active" : "kind-btn"}
        onClick={() => onSelect("npm")}
        disabled={disabled}
      >
        npm 安装
      </button>
      <button
        type="button"
        className={kind === "dev" ? "kind-btn active" : "kind-btn"}
        onClick={() => onSelect("dev")}
        disabled={disabled}
      >
        dev 仓库
      </button>
    </div>
  );
}
