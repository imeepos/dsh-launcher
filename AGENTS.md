# AGENTS.md

## 项目代码规范

### 文件与函数规模

- 单个源码文件不得超过 200 行（按物理行计数）。
- 单个函数、方法或 React 组件函数不得超过 50 行。
- 复杂逻辑应按职责拆分为多个模块、组件或辅助函数，不要通过压缩代码规避限制。
- 测试文件、脚本和配置源码同样遵守文件规模限制。

### 模块设计

- 前端 React 组件按职责拆分：页面容器只负责状态编排，表格、对话框、表单字段和操作区独立成组件。
- React 副作用和可复用状态逻辑提取到 src/hooks/。
- Tauri/Rust 代码按职责拆分：命令装配、注册表模型、注册表持久化、业务方法、进程运行时和 npm 安装逻辑分别维护。
- 模块入口文件只做声明、导出和装配，不承载大量业务实现。
- 保持现有 API、命令名、序列化字段名和用户可见错误语义不变；重构优先保证行为等价。

### 前端规范

- 使用 TypeScript 类型定义，API 封装集中在 src/api.ts。
- 样式按职责拆分到 src/styles/，由 src/App.css 按稳定顺序导入。
- 组件回调通过明确的 Props 类型传递，避免隐式共享状态。
- 异步操作必须处理 loading、错误和清理逻辑；事件监听在组件卸载时解除。

### Rust/Tauri 规范

- 注册表写入必须经过现有串行锁和原子保存流程。
- 文件路径、版本 ID 和 npm 安装目录必须进行校验，防止目录穿越。
- 子进程必须配置合理的 stdin/stdout/stderr，并处理退出码、超时和清理。
- 修改 Rust 后在 src-tauri 目录执行格式化和测试。

### 验证要求

提交前至少执行：

    pnpm build
    pnpm check:size

涉及 Rust 时还要执行：

    cargo fmt --check
    cargo check
    cargo test

Rust 命令的工作目录是 src-tauri/。代码规模检查由 scripts/check-size.mjs 提供，并通过 package script pnpm check:size 调用。

### 提交规范

- 提交前确认 git status，避免遗漏新增文件或误提交生成物。
- 使用简洁、明确的 Conventional Commit 风格提交信息，例如 refactor: split launcher modules。
- 不提交 dist/、target/、node_modules/ 等生成目录，除非项目明确要求。
