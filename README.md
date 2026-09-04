# dsh-launcher / CLI 工具台

Rust + Tauri 2 桌面应用:为**所有命令行工具、无界面工具、非客户端 GUI 工具**提供统一的可视化管理。

- 工具库:登记/安装/指纹/删除任意 CLI 工具,非 DSH 工具可直接以任意参数运行
- 目录:对接 release-platform(产品/发布/制品浏览,下载安装,三模式认证)
- DSH 专属:启动台(Home/Profile)、首跑向导、环境自愈

设计文档:[DESIGN.md](DESIGN.md) · [DESIGN-TOOLS.md](DESIGN-TOOLS.md) · [DESIGN-ONBOARDING.md](DESIGN-ONBOARDING.md)

```bash
pnpm install
pnpm tauri dev   # 桌面应用开发模式
pnpm build       # 前端构建门禁
pnpm check:size  # 文件/函数体量门禁
```

## 环境变量与 worktree

真实 `.env` 含密钥,不进版本库;新建 worktree 的 `.env` 由 `githooks/post-checkout` 自动接入:

- 分支检出(含 `git worktree add`)时,已有 `.env` 一律不动;缺失时优先从主 worktree 复制,
  主 worktree 也没有则用 [`.env.example`](.env.example) 兜底,并打印告警提示替换占位值。
- `.git/hooks` 不随 clone 传播,全新 clone 后需一次性引导启用钩子:

```bash
./scripts/setup-hooks.sh   # 等价于 git config core.hooksPath githooks
```

引导后该 clone 下所有新建 worktree 自动获得可用的 `.env`,无需手工复制或链接。
