// Line-counting and offset-to-line mapping helpers for the size guard.
export function physicalLines(text) {
  const n = text.split("\n").length;
  return text.endsWith("\n") ? n - 1 : n;
}

export const lineStarts = (t) => {
  const starts = [0];
  for (let i = 0; i < t.length; i++) if (t[i] === "\n") starts.push(i + 1);
  return starts;
};

export function lineAt(starts, index) {
  let lo = 0;
  let hi = starts.length - 1;
  while (lo < hi) {
    const mid = (lo + hi + 1) >> 1;
    if (starts[mid] <= index) lo = mid;
    else hi = mid - 1;
  }
  return lo + 1;
}
