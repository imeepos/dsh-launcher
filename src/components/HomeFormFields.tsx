import type { HomeFormMode } from "./HomeFormDialog";

interface Props {
  mode: HomeFormMode;
  path: string;
  onPath: (v: string) => void;
  id: string;
  onId: (v: string) => void;
}

function pathLabel(mode: HomeFormMode) {
  if (mode === "create") return "目录路径(留空则自动 ~/.dsh-launcher/homes/<id>)";
  return "目录路径";
}

function pathPlaceholder(mode: HomeFormMode) {
  if (mode === "create") return "(自动)";
  if (mode === "clone") return "(自动 ~/.dsh-launcher/homes/<id>)";
  return "~/.dsh";
}

function HomeFormFields({ mode, path, onPath, id, onId }: Props) {
  return (
    <>
      <label>
        {pathLabel(mode)}
        <input
          value={path}
          onChange={(e) => onPath(e.target.value)}
          placeholder={pathPlaceholder(mode)}
          autoFocus
        />
      </label>
      <label>
        id(可选)
        <input value={id} onChange={(e) => onId(e.target.value)} placeholder="main" />
      </label>
    </>
  );
}

export default HomeFormFields;
