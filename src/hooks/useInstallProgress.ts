import { useEffect, useRef, useState } from "react";

// npm 不提供机器可读的安装进度(管道模式下其进度渲染被禁用),
// 这里按「日志行数 + 已耗时」双渐近线爬行推进,封顶 90%;
// 结束时成功补满 100%,失败定格当前值,停留片刻后隐藏并复位。
function useInstallProgress(running: boolean, lineCount: number, failed: boolean) {
  const [percent, setPercent] = useState(0);
  const [visible, setVisible] = useState(false);
  const startedRef = useRef(false);
  const linesRef = useRef(0);
  linesRef.current = lineCount;

  useEffect(() => {
    if (running) {
      startedRef.current = true;
      setVisible(true);
      setPercent((p) => (p < 3 ? 3 : p));
      const start = Date.now();
      const iv = window.setInterval(() => {
        const secs = (Date.now() - start) / 1000;
        const target = 4 + 86 * (1 - Math.exp(-(linesRef.current * 0.05 + secs / 25)));
        setPercent((p) => Math.max(p, Math.min(90, Math.round(target))));
      }, 250);
      return () => window.clearInterval(iv);
    }
    if (!startedRef.current) return;
    if (!failed) setPercent(100);
    const t = window.setTimeout(() => {
      setVisible(false);
      setPercent(0);
      startedRef.current = false;
    }, 900);
    return () => window.clearTimeout(t);
  }, [running, failed]);

  return { percent, visible };
}

export default useInstallProgress;
