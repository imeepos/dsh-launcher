import type { ReactNode } from "react";

interface Props {
  title: string;
  actions?: ReactNode;
}

function PanelHeader({ title, actions }: Props) {
  return (
    <header className="panel-header">
      <h2>{title}</h2>
      <div className="toolbar-actions">{actions}</div>
    </header>
  );
}

export default PanelHeader;
