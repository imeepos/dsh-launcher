// 启动观察:订阅日志与退出事件并轮询运行列表;observe=true 期间生效,回调走 ref 防抖。

import { useEffect, useRef } from "react";
import { listRunning, onProcessExit, onProcessLog } from "../api";

const OBSERVE_MS = 30000;
const POLL_MS = 2000;

interface Params {
  homeId: string;
  profile: string;
  observing: boolean;
  onLog: (line: string) => void;
  onFail: (why: string) => void;
  onHealthy: () => void;
}

function startPolling(
  key: string,
  startedAt: number,
  onFail: (why: string) => void,
  onHealthy: () => void,
): () => void {
  let done = false;
  let timer: number | null = null;
  const stop = () => {
    done = true;
    if (timer !== null) window.clearInterval(timer);
  };
  timer = window.setInterval(() => {
    listRunning()
      .then((keys) => {
        if (done) return;
        if (!keys.includes(key)) {
          stop();
          onFail("进程不在运行列表中");
        } else if (Date.now() - startedAt >= OBSERVE_MS) {
          stop();
          onHealthy();
        }
      })
      .catch(() => {});
  }, POLL_MS);
  return stop;
}

export default function useLaunchObservation({
  homeId,
  profile,
  observing,
  onLog,
  onFail,
  onHealthy,
}: Params) {
  const cbRef = useRef({ onLog, onFail, onHealthy });
  cbRef.current = { onLog, onFail, onHealthy };

  useEffect(() => {
    if (!observing) return;
    const key = homeId + "/" + profile;
    const cleanups: (() => void)[] = [];
    let closed = false;
    onProcessLog((p) => {
      if (p.homeId === homeId && p.profile === profile) cbRef.current.onLog(p.line);
    })
      .then((u) => {
        if (closed) u();
        else cleanups.push(u);
      })
      .catch(() => {});
    onProcessExit((p) => {
      if (p.homeId === homeId && p.profile === profile) {
        cbRef.current.onFail("进程提前退出(exit " + (p.exitCode ?? "?") + ")");
      }
    })
      .then((u) => {
        if (closed) u();
        else cleanups.push(u);
      })
      .catch(() => {});
    const stopPolling = startPolling(key, Date.now(),
      (why) => cbRef.current.onFail(why),
      () => cbRef.current.onHealthy(),
    );
    return () => {
      closed = true;
      stopPolling();
      cleanups.forEach((fn) => fn());
    };
  }, [homeId, profile, observing]);
}