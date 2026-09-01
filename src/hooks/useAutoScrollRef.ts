import { useEffect, useRef } from "react";

// Keeps a pre/console pinned to the bottom whenever dep changes.
function useAutoScrollRef(dep: unknown) {
  const ref = useRef<HTMLPreElement | null>(null);
  useEffect(() => {
    const el = ref.current;
    if (el) el.scrollTop = el.scrollHeight;
  }, [dep]);
  return ref;
}

export default useAutoScrollRef;
