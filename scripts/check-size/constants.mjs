// Limits and file-classification sets for the size guard.
export const MAX_FILE = 200;
export const MAX_FN = 50;
export const CODE_EXT = new Set([".ts", ".tsx", ".js", ".jsx", ".mjs", ".cjs"]);
export const TEXT_EXT = new Set([".css", ".html", ".json", ".md", ".toml", ".yaml", ".yml"]);
export const LOCKFILES = new Set(["cargo.lock", "pnpm-lock.yaml", "yarn.lock", "package-lock.json", "bun.lockb"]);
export const GEN_DIRS = new Set(["node_modules", "dist", "dist-ssr", "target", "build", "out", "coverage", ".git", "gen"]);
