import type { HomeEntry } from "../api";

interface Props {
  home: HomeEntry;
  selected: boolean;
  onSelect: () => void;
}

// 左栏 Home 紧凑卡:首字母头像 + 名称 + 绑定版本;选中 = 2px 蓝描边。
export default function HomeCard({ home, selected, onSelect }: Props) {
  return (
    <button
      className={"home-card" + (selected ? " selected" : "")}
      onClick={onSelect}
      aria-pressed={selected}
    >
      <span className="home-avatar">{home.id.slice(0, 1).toUpperCase()}</span>
      <span className="home-card-body">
        <span className="home-card-name">{home.id}</span>
        <span className="home-card-sub mono">
          {home.boundVersionId ?? "未绑定版本"}
        </span>
      </span>
    </button>
  );
}
