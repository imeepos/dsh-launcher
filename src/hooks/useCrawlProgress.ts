// 爬行进度估算:无精确进度的长任务按「日志行数 + 已耗时」推进,封顶 90%,成功由调用方补满。

import { useCallback, useEffect, useRef, useState } from "react";

const BASE_PERCENT = 5;
const CAP_PERCENT = 90;

export type LineSubscriber = (cb: (line: string) => void) => Promise<(() => void) | undefined | null>;

interface Params {
  subscribe: LineSubscriber;
  /** 只统计返回 true 的事件行 */
  match: (line: string) => boolean;
}

export default function useCrawlProgress({ subscribe, match }: Params) {
  const [lines, setLines] = useState<string[]>([]);
  const [percent, setPercent] = useState(BASE_PERCENT);
  const unRef = useRef<(() => void) | null | undefined>(null);
  const countRef = useRef(0);
  const startRef = useRef(0);
  const matchRef = useRef(match);
  matchRef.current = match;

  useEffect(() => {
    let stopped = false;
    subscribe((line) => {
      if (!matchRef.current(line)) return;
      countRef.current += 1;
      const kept = countRef.current;
      const sec = (Date.now() - startRef.current) / 1000;
      const estimate = Math.min(CAP_PERCENT, BASE_PERCENT + kept * 1.5 + sec / 8);
      setPercent(Math.floor(estimate));
      setLines((prev) => [...prev.slice(-200), line]);
    })
      .then((un) => {
        if (stopped) un?.();
        else unRef.current = un;
      })
      .catch(() => {});
    return () => {
      stopped = true;
      unRef.current?.();
    };
  }, [subscribe]);

  /** 计数清零并重新计时(点击开始时调用) */
  const reset = useCallback(() => {
    countRef.current = 0;
    startRef.current = Date.now();
    setPercent(BASE_PERCENT);
    setLines([]);
  }, []);

  /** 任务成功后补满 100% */
  const finish = useCallback(() => setPercent(100), []);

  return { lines, percent, reset, finish };
}