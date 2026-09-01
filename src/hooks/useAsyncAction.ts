import { useCallback, useEffect, useRef, useState } from "react";

export type AsyncStatus = "idle" | "loading" | "success" | "error";

export type AsyncResult = { ok: true } | { ok: false; error: unknown };

const SUCCESS_RESET_MS = 1200;
const ERROR_RESET_MS = 1600;

// Drives one async action through idle -> loading -> success/error -> idle.
// The terminal state is held briefly so the button can show a micro-feedback.
function useAsyncAction() {
  const [status, setStatus] = useState<AsyncStatus>("idle");
  const timerRef = useRef<number | null>(null);

  useEffect(() => {
    return () => {
      if (timerRef.current !== null) window.clearTimeout(timerRef.current);
    };
  }, []);

  const clearResetTimer = useCallback(() => {
    if (timerRef.current !== null) {
      window.clearTimeout(timerRef.current);
      timerRef.current = null;
    }
  }, []);

  const scheduleReset = useCallback(
    (delay: number) => {
      clearResetTimer();
      timerRef.current = window.setTimeout(() => setStatus("idle"), delay);
    },
    [clearResetTimer],
  );

  const run = useCallback(
    async (task: () => Promise<void>): Promise<AsyncResult> => {
      if (status === "loading") return { ok: false, error: new Error("busy") };
      clearResetTimer();
      setStatus("loading");
      try {
        await task();
        setStatus("success");
        scheduleReset(SUCCESS_RESET_MS);
        return { ok: true };
      } catch (error) {
        setStatus("error");
        scheduleReset(ERROR_RESET_MS);
        return { ok: false, error };
      }
    },
    [status, clearResetTimer, scheduleReset],
  );

  return { status, run };
}

export default useAsyncAction;
