# dsh-launcher 设计文档

Rust + Tauri 2 桌面启动器：可视化管理多版本 DSH、多 DSH_HOME、多 profile。

## dsh 语义基准（设计依据，勿凭感觉改）

1. **Home 解析优先级**（`resolveDshHome()`）：显式配置 > `$DSH_HOME` 环境变量 > `~/.dsh`；空白值视为未设置；支持 `~` 展开。Launcher 启动 dsh 时必须清掉环境里游离的 `DSH_*` 后显式设置 `DSH_HOME`，独占其来源。
2. **bundle 双锚点解析**：bundle 名先从**运行中的 dsh 安装**解析（模板 bundle 如 `dsh-base` 永远来自运行版本），再从 profile 目录解析（`dsh plugin add` 的自装插件在 profile 自己的 node_modules）。因此同一 home 被不同版本启动行为不同——**home 绑定版本是一等公民**，跨版本启动须警告。
3. **启动命令**：`env -u DSH_HOME DSH_HOME=<home> <bin> --profile <name> [--patch a.yml] -- <args>`。停止用 **SIGTERM**（dsh 约定 exit 0）；SIGINT 是用户中断（130），不要拿它当"停止"。
4. **工作区 = 进程 cwd**：各表面 Agent 的 `meta.cwd` 取 `process.cwd()`，CLI 没有 `--cwd` 参数（headless 命令行只有 task 位置参数和 --help）。Launcher 每次启动提供工作区选择，spawn 时设 `cwd` 即可；后续编号顺延。
4. **home 内部结构归 dsh 管**：`profiles/<name>/`（含 `package.json` 的 `dsh.profile.bundles` 清单与 `patchReload`、用户层 `cordis.patch.yml`）+ home 层 `cordis.patch.yml`。根 `cordis.yml` 由 dsh 每次启动重写为空列表，**UI 不提供编辑入口**。
5. **无启动诊断**：`--dump-config`（全组合，不执行 `!!js`）与 `--dump-default-config`（跳过用户层，坏 patch 时的恢复诊断）。
6. **每个 (home, profile) 单实例**：并发会撞 SQLite 与端口，launcher 侧加运行锁。
7. v1 只支持 macOS/Linux；Windows 后补。

## 数据模型（`~/.dsh-launcher/registry.json`）

```jsonc
{
  "versions": [{
    "id": "v0.1.1-rc.2",
    "kind": "npm",                // npm | dev
    "spec": "@deepseek-ai/dsh@0.1.1-rc.2",
    "bin": "~/.dsh-launcher/versions/v0.1.1-rc.2/node_modules/.bin/dsh",
    "cwd": null,                  // dev kind 为 repoPath
    "fingerprint": "0.1.1-rc.2"   // spawn <bin> --version 实测写回
  }],
  "homes": [{
    "id": "main",
    "path": "~/.dsh",
    "boundVersionId": "v0.1.1-rc.2",
    "lastGoodVersionId": null     // 最后一次成功启动的版本
  }],
  "history": []                   // {startedAt, homeId, profile, versionId, exitCode}
}
```

- npm 版本安装：`npm install --prefix ~/.dsh-launcher/versions/<id> @deepseek-ai/dsh@<ver>`（公网已确认可用），每版本独立依赖树。
- dev 版本：登记 repo checkout，启动命令 `pnpm dsh`，cwd=repoPath。
- home 只登记路径不搬家；删除 npm 版本连目录一起删，dev 只摘登记。

## 里程碑

| 阶段 | 内容 | 验收 |
|---|---|---|
| M0 | Tauri 脚手架 + registry 读写 + 指纹实测 + 最小版本列表 UI | `pnpm build` 与 `cargo check` 零错误；registry 单测过 |
| M1 | 版本管理：npm 安装 / dev 登记 / 指纹 / 删除 | 对 dev repo 与 npm 安装版各取到非空指纹 |
| M2 | home 管理：登记/新建/克隆 + 版本绑定 | 同版本双 home 同时各起一个 profile |
| M3 | profile 发现 + 启动/停止 + 日志流 + 运行锁 | SIGTERM 停止 exit 0；二次启动被锁拒绝 |
| M4 | dump 查看器 + 失败诊断（自动 dump-default 对比） | 坏 patch 时 UI 标出坏行 |
| M5 | patch 编辑器 + `dsh plugin` 转发 | live profile 改 patch 热生效 |

## UI 结构

三栏树：版本 → Home → Profile，底部详情区（日志控制台 / dump 分层视图 / 失败诊断）。每层右键菜单对应该层能力（见里程碑）。
