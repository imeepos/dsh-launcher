#!/usr/bin/env node
// Size guard for dsh-launcher sources.
// Rules:
//   1. Every project source file <= 200 physical lines.
//   2. Every function/method in TS/TSX/JS and Rust files <= 50 physical lines,
//      measured with a practical heuristic: comments/strings are masked out,
//      then function bodies are located by brace matching.
// Scope: git-known project files (tracked + uncommitted additions, i.e.
// `git ls-files --cached --others --exclude-standard`), excluding generated
// dirs (node_modules, dist, target, ...) via .gitignore plus an explicit list,
// lockfiles, and non-source extensions. Known heuristic limits: closures and
// expression-bodied arrow functions have no brace body and are skipped; regex
// literals are not masked.
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const MAX_FILE = 200;
const MAX_FN = 50;
const CODE_EXT = new Set([".ts", ".tsx", ".js", ".jsx", ".mjs", ".cjs"]);
const TEXT_EXT = new Set([".css", ".html", ".json", ".md", ".toml", ".yaml", ".yml"]);
const LOCKFILES = new Set(["cargo.lock", "pnpm-lock.yaml", "yarn.lock", "package-lock.json", "bun.lockb"]);
const GEN_DIRS = new Set(["node_modules", "dist", "dist-ssr", "target", "build", "out", "coverage", ".git", "gen"]);

function listSourceFiles() {
  const out = execFileSync(
    "git",
    ["ls-files", "--cached", "--others", "--exclude-standard"],
    { cwd: ROOT, encoding: "utf8" },
  );
  return out.split("\n").filter(Boolean).filter((f) => {
    const parts = f.split("/");
    const ext = path.extname(f);
    return (
      !parts.some((p) => GEN_DIRS.has(p)) &&
      !LOCKFILES.has(parts[parts.length - 1].toLowerCase()) &&
      (CODE_EXT.has(ext) || TEXT_EXT.has(ext))
    );
  });
}

function physicalLines(text) {
  const n = text.split("\n").length;
  return text.endsWith("\n") ? n - 1 : n;
}

const lineStarts = (t) => {
  const starts = [0];
  for (let i = 0; i < t.length; i++) if (t[i] === "\n") starts.push(i + 1);
  return starts;
};

function lineAt(starts, index) {
  let lo = 0;
  let hi = starts.length - 1;
  while (lo < hi) {
    const mid = (lo + hi + 1) >> 1;
    if (starts[mid] <= index) lo = mid;
    else hi = mid - 1;
  }
  return lo + 1;
}

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

// --- masking: blank out comments/strings while keeping offsets and newlines ---
const blankAt = (chars, j) => {
  if (chars[j] !== "\n") chars[j] = " ";
};

function maskRust(text) {
  const chars = text.split("");
  let i = 0;
  while (i < text.length) {
    const c = text[i];
    const raw = /^r(#*)"/.exec(text.slice(i));
    if (c === "/" && text[i + 1] === "/") {
      while (i < text.length && text[i] !== "\n") { blankAt(chars, i); i++; }
    } else if (c === "/" && text[i + 1] === "*") {
      let depth = 0;
      while (i < text.length) {
        if (text.startsWith("/*", i)) { depth++; blankAt(chars, i); blankAt(chars, i + 1); i += 2; }
        else if (text.startsWith("*/", i)) { depth--; blankAt(chars, i); blankAt(chars, i + 1); i += 2; if (depth === 0) break; }
        else { blankAt(chars, i); i++; }
      }
    } else if (raw) {
      const close = '"' + raw[1];
      const end = text.indexOf(close, i + raw[0].length);
      const stop = end < 0 ? text.length : end + close.length;
      for (let j = i; j < stop; j++) blankAt(chars, j);
      i = stop;
    } else if (c === '"') {
      blankAt(chars, i);
      let j = i + 1;
      while (j < text.length && text[j] !== '"' && text[j] !== "\n") {
        if (text[j] === "\\") { blankAt(chars, j); j++; }
        blankAt(chars, j);
        j++;
      }
      if (j < text.length && text[j] === '"') blankAt(chars, j);
      i = j + 1;
    } else if (c === "'") {
      const m = /^'(\\.|[^'\\])'/.exec(text.slice(i, i + 5));
      if (m) { for (let j = 0; j < m[0].length; j++) blankAt(chars, i + j); i += m[0].length; }
      else i++; // lifetime tick, not a char literal
    } else i++;
  }
  return chars.join("");
}

const PREQUOTE = "=([{,:;&|?+-*%~^<>!";
const KW_BEFORE = /(?:^|[^\w$])(?:return|throw|case|typeof|instanceof|new|delete|void|in|of|await|yield|import|export|default)$/;

function quoteStartsString(text, i) {
  let j = i - 1;
  while (j >= 0 && /\s/.test(text[j])) j--;
  if (j < 0) return true;
  if (PREQUOTE.includes(text[j])) return true;
  return KW_BEFORE.test(text.slice(Math.max(0, j - 12), j + 1));
}

function maskTs(text) {
  const chars = text.split("");
  let i = 0;
  while (i < text.length) {
    const c = text[i];
    if (c === "/" && text[i + 1] === "/") {
      while (i < text.length && text[i] !== "\n") { blankAt(chars, i); i++; }
    } else if (c === "/" && text[i + 1] === "*") {
      while (i < text.length && !text.startsWith("*/", i)) { blankAt(chars, i); i++; }
      blankAt(chars, i);
      blankAt(chars, i + 1);
      i += 2;
    } else if (c === '"' || c === "'" || c === "`") {
      const q = c;
      if (q !== "`" && !quoteStartsString(text, i)) { i++; continue; } // JSX/text apostrophe
      blankAt(chars, i);
      let j = i + 1;
      while (j < text.length) {
        if (text[j] === "\\") { blankAt(chars, j); blankAt(chars, j + 1); j += 2; continue; }
        if (text[j] === q || (text[j] === "\n" && q !== "`")) break;
        blankAt(chars, j);
        j++;
      }
      if (j < text.length && text[j] === q) blankAt(chars, j);
      i = j + 1;
    } else i++;
  }
  return chars.join("");
}

// --- function detection over masked text ---
const RUST_FN = /(?:^|[\n;])\s*(?:(?:pub\s*\([^)]*\)|pub|const|unsafe|async|extern)\s+)*fn\s+([A-Za-z_][A-Za-z0-9_]*)/g;
const TS_FN_DECL = /\bfunction\b\s*\*?\s*(?:[A-Za-z_$][\w$]*)?/g;
const TS_ARROW = /=>\s*\{/g;
const TS_METHOD = /(?:^|[\n;{,])\s*(?:(?:public|private|protected|static|readonly|override|async|get|set)\s+)*([A-Za-z_$][\w$]*)\s*(?:<[^<>]*>)?\s*\((?:[^()]|\([^()]*\))*\)\s*(?::\s*[^{;=]+)?\s*\{/g;
const CONTROL = new Set(["if", "for", "while", "switch", "catch", "do", "else", "try", "finally", "return", "throw", "new", "typeof", "delete", "void", "await", "yield", "in", "of", "with", "function"]);

function collectFn(masked, name, open, out) {
  const end = matchBrace(masked, open);
  if (end > 0) out.push({ name, start: open, end });
}

function rustFunctions(masked) {
  const fns = [];
  for (const m of masked.matchAll(RUST_FN)) {
    const open = masked.indexOf("{", m.index + m[0].length);
    if (open >= 0) collectFn(masked, m[1], open, fns);
  }
  return fns;
}

function tsFunctions(masked) {
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

function checkFile(rel, problems) {
  const text = readFileSync(path.join(ROOT, rel), "utf8");
  const lines = physicalLines(text);
  if (lines > MAX_FILE) problems.push(`${rel}: file is ${lines} lines (max ${MAX_FILE})`);
  if (!CODE_EXT.has(path.extname(rel))) return { lines: 0, fns: [] };
  const isRust = rel.endsWith(".rs");
  const masked = isRust ? maskRust(text) : maskTs(text);
  return { lines, fns: isRust ? rustFunctions(masked) : tsFunctions(masked), starts: lineStarts(text) };
}

function main() {
  const problems = [];
  let fnCount = 0;
  let files;
  try {
    files = listSourceFiles();
  } catch {
    console.error("check-size: must run inside a git repository (git ls-files failed)");
    process.exitCode = 2;
    return;
  }
  for (const rel of files) {
    let r;
    try {
      r = checkFile(rel, problems);
    } catch {
      continue; // unreadable/binary file slipped past the extension filter
    }
    for (const fn of r.fns) {
      fnCount++;
      const from = lineAt(r.starts, fn.start);
      const len = lineAt(r.starts, fn.end) - from + 1;
      if (len > MAX_FN) problems.push(`${rel}:${from}: function "${fn.name}" spans ${len} lines (max ${MAX_FN})`);
    }
  }
  for (const p of problems) console.error("check-size: " + p);
  console.log(`check-size: ${files.length} files and ${fnCount} functions checked — ${problems.length ? `${problems.length} problem(s) found` : "all within limits"}`);
  process.exitCode = problems.length ? 1 : 0;
}

main();
