// Function detection over masked text: locate brace bodies and measure spans.
const RUST_FN = /(?:^|[\n;])\s*(?:(?:pub\s*\([^)]*\)|pub|const|unsafe|async|extern)\s+)*fn\s+([A-Za-z_][A-Za-z0-9_]*)/g;
const TS_FN_DECL = /\bfunction\b\s*\*?\s*(?:[A-Za-z_$][\w$]*)?/g;
const TS_ARROW = /=>\s*\{/g;
const TS_METHOD = /(?:^|[\n;{,])\s*(?:(?:public|private|protected|static|readonly|override|async|get|set)\s+)*([A-Za-z_$][\w$]*)\s*(?:<[^<>]*>)?\s*\((?:[^()]|\([^()]*\))*\)\s*(?::\s*[^{;=]+)?\s*\{/g;
const CONTROL = new Set(["if", "for", "while", "switch", "catch", "do", "else", "try", "finally", "return", "throw", "new", "typeof", "delete", "void", "await", "yield", "in", "of", "with", "function"]);

function matchBrace(text, open) {
  let depth = 0;
  for (let i = open; i < text.length; i++) {
    if (text[i] === "{") depth++;
    else if (text[i] === "}" && --depth === 0) return i;
  }
  return -1;
}

function skipParens(text, i) {
  while (i < text.length && /\s/.test(text[i])) i++;
  if (text[i] !== "(") return i;
  let depth = 0;
  for (; i < text.length; i++) {
    if (text[i] === "(") depth++;
    else if (text[i] === ")" && --depth === 0) return i + 1;
  }
  return i;
}

function collectFn(masked, name, open, out) {
  const end = matchBrace(masked, open);
  if (end > 0) out.push({ name, start: open, end });
}

export function rustFunctions(masked) {
  const fns = [];
  for (const m of masked.matchAll(RUST_FN)) {
    const open = masked.indexOf("{", m.index + m[0].length);
    if (open >= 0) collectFn(masked, m[1], open, fns);
  }
  return fns;
}

export function tsFunctions(masked) {
  const fns = [];
  for (const m of masked.matchAll(TS_FN_DECL)) {
    const open = masked.indexOf("{", skipParens(masked, m.index + m[0].length));
    const name = m[0].trim().split(/\s+/).pop() || "function";
    if (open >= 0) collectFn(masked, name, open, fns);
  }
  for (const m of masked.matchAll(TS_ARROW)) {
    const before = masked.slice(Math.max(0, m.index - 120), m.index);
    const am = /([A-Za-z_$][\w$]*)\s*=\s*(?:async\s*)?$/.exec(before);
    collectFn(masked, am ? am[1] : "arrow", m.index + m[0].length - 1, fns);
  }
  for (const m of masked.matchAll(TS_METHOD)) {
    if (!CONTROL.has(m[1])) collectFn(masked, m[1], m.index + m[0].length - 1, fns);
  }
  return fns;
}
