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
