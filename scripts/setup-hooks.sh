#!/bin/sh
# 一次性引导:在本 clone 启用 githooks/ 下的 git 钩子。
# .git/hooks 不随 clone 传播,全新 clone 后执行本脚本一次即可,
# 对该 clone 的所有 worktree 生效(含新建 worktree 的 .env 自动接入)。

set -eu
cd "$(git rev-parse --show-toplevel)"

if [ ! -x githooks/post-checkout ]; then
  echo "setup-hooks: githooks/post-checkout missing or not executable" >&2
  exit 1
fi

git config core.hooksPath githooks
echo "setup-hooks: core.hooksPath=githooks enabled;"
echo "setup-hooks: new checkouts will auto-provision .env (existing .env files are never touched)"
