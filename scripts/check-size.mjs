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
import { CODE_EXT, GEN_DIRS, LOCKFILES, MAX_FILE, MAX_FN, TEXT_EXT } from "./check-size/constants.mjs";
import { lineAt, lineStarts, physicalLines } from "./check-size/text.mjs";
import { maskRust, maskTs } from "./check-size/mask.mjs";
import { rustFunctions, tsFunctions } from "./check-size/functions.mjs";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

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
