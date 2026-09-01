import type { VersionKind } from "../api";

export default function KindBadge({ kind }: { kind: VersionKind }) {
  return <span className={"badge badge-" + kind}>{kind}</span>;
}
