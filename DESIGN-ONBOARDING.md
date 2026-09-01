# dsh-launcher 萌新首跑设计（全新系统 → 跑起来）

出发点：用户电脑是新装的系统——没有 Node、npm、pnpm、git，没有 `~/.dsh`，
也没有终端使用经验。本设计覆盖「双击安装包 → 一路默认 → 看到 dsh 日志」全链路，
以及之后的日常自愈。dsh 语义基准见 DESIGN.md，本文只解决「环境从零到一」。

## 0. 设计原则

1. 零终端、零 sudo、不写系统 PATH、不动用户已有环境。
2. 每步幂等可重试：向导任意时刻崩溃/被杀，重开后从断点继续，已通过的检查不重跑。
3. 错误说人话：每个失败 = 一句标题 + 一句原因 + 一个大按钮（能自动修就自动修）。
4. 高级能力折叠不隐藏：dev 版本、系统 Node、自定义镜像收进「高级」，默认路径只有「下一步」。

## 1. 核心决策：自带 Node 运行时（managed runtime）

- 现状 `resolve_npm()`（launcher_npm.rs）依赖系统 PATH，新装系统必然报「找不到 npm」。
- 决策：Launcher 自管一份 Node，装在 `~/.dsh-launcher/runtime/node-v<ver>-<os>-<arch>/`；
  所有 spawn（npm 安装、指纹实测、启动 dsh）把该 `bin/` 前置进 PATH。
- 即使系统有 Node 也默认不用：版本/glibc 不可控，故障面大。设置保留「使用系统 Node」开关（高级）。
- dev 版本（pnpm + git + repo）定位为高级玩法，不进萌新向导；缺依赖只提示，不代装。

### 下载与校验

- 双源：官方 `nodejs.org/dist` 与国内 `npmmirror.com/mirrors/node`；并发探测，选先可达者。
- 流程：取 `SHASUMS256.txt` → 下载 `node-v<ver>-<os>-<arch>.tar.xz`（进度百分比 + 预估时间）
  → sha256 校验 → 解压到 runtime 目录 → 写 `runtime.json`。
- 任一步失败自动换另一源重试一次；再失败出问题卡片（错误分类见 §5）。
- 安装动作与 npm 版本安装共用注册表串行锁，向导与修复中心并发触发天然互斥。

## 2. 环境检查（envcheck）

检查项模型：`{ id, level: blocker|warn|info, status: pass|fail|skip, detail, fix? }`。
检查逻辑纯函数化（输入系统快照，输出清单），便于单测与两处复用（首跑全量 / 日常预检）。

| id | 内容 | 级别 | 修复动作 |
|---|---|---|---|
| sys_os | macOS ≥ 12 或 Linux glibc ≥ 2.17 | blocker | 无（提示升级系统） |
| sys_arch | arm64 / x86_64 已知架构 | blocker | 无 |
| disk_space | base 卷可用 ≥ 1GB（runtime≈50MB + 版本≈300MB + home SQLite） | blocker | 磁盘清理指引 |
| base_dir | `~/.dsh-launcher` 可创建可写 | blocker | 权限指引 / 重试 |
| net_registry | npm registry HTTPS 可达（2s 探测） | blocker | 切换镜像 |
| net_node_dist | node 发行版镜像可达 | blocker | 切换镜像 |
| runtime | 运行时存在且 `node --version` 正常 | blocker | 安装 / 重装运行时 |
| existing_dsh | PATH 里有 dsh 或 `~/.dsh` 存在 | info | 「导入已有 home」 |

- 首跑跑全量；日常启动只跑快速预检（runtime 存在、registry 可解析、选中版本 bin 存在、
  home 存在、陈旧锁），全部 O(1) stat，毫秒级。
- 陈旧锁判定：锁文件在但对应 pid 无存活进程 → warn，修复 = 确认进程不在后清理。

## 3. 首跑向导（onboarding wizard）

状态机：`welcome → check → fix* → mode → install → home → launch → done`。
状态每步落盘 `registry.onboarding`，重启续跑；每步幂等，重复执行安全。

- **welcome**：一屏说明「将自动准备运行环境，约 5–10 分钟」，只有一个开始按钮。
- **check**：并发执行全量检查，流式打勾动画；blocker 全过 → 直达 mode。
- **fix***：每个 blocker 一张问题卡片：人话标题 + 原因 + 大按钮（自动修复/重试/换镜像/复制详情）。
  runtime 安装卡内嵌下载进度；修复后自动重跑该检查，绿了才放行。
- **mode**：快速（推荐，npm 最新稳定版，一键）/ 自定义（列可用版本选择）/
  高级（dev repo，折叠，注明需自行准备 git + pnpm，向导不代装）。
- **install**：复用现有 `install_npm`，注入 runtime PATH；日志流展示（复用 InstallLogView）。
- **home**：默认 `~/.dsh` + profile `main`，名字可改；检测到已有 home → 出现「导入」选项（只登记不搬家）。
- **launch**：按 DESIGN.md 启动命令起 profile → 取指纹 → 健康判定：
  进程存活 30s + 日志累计 > 1KB + 无 fatal 级别行 → 成功徽章；
  失败自动 `--dump-default-config` 转诊断（衔接 M4），讲清「哪一步、什么错、下一步点哪」。
- **done**：三张卡片：打开日志 / 在访达打开 home / 下次直接点启动。

## 4. 修复中心（repair center，日常自愈）

- 每次启动后台跑快速预检；问题以卡片挂在主界面顶部环境状态条（绿/黄/红），点开抽屉处理。
- 修复动作与向导共用同一套后端命令（单一实现）：
  `install_runtime` / `reinstall_runtime` / `install_version` / `import_home` / `clear_stale_lock`。
- 常见映射：runtime 被删 → 重装；版本目录不完整 → 重装该版本；home 丢失 → 新建空 home
  （明示数据不可恢复）；陈旧锁 → 确认进程不在后清理。

## 5. 错误分类学

固定六类：network / disk / permission / checksum / compat / unknown。
每类一个模板：标题、一句原因、建议动作、重试按钮。所有网络操作有超时与进度；
所有下载落盘前过 sha256；任何失败都可「复制详情」（脱敏诊断），方便萌新求助。

## 6. 数据模型增量（registry.json）

```jsonc
{
  "runtime": { "nodeVersion": "22.x.x", "bin": "~/.dsh-launcher/runtime/.../bin",
               "installedAt": "...", "sha256": "..." },
  "settings": { "npmRegistry": "https://registry.npmmirror.com",
                "nodeDistMirror": "https://npmmirror.com/mirrors/node", "useSystemNode": false },
  "onboarding": { "v": 1, "completed": false, "step": "install", "checks": [] }
}
```

- 向后兼容：老 registry 缺这些字段 → `onboarding.completed = true`（老用户不弹向导），
  runtime 缺失仍会进修复中心。
- `useSystemNode = true` 时 `resolve_npm()` 维持现逻辑；envcheck 的 runtime 项改查系统 node ≥ 18。

## 7. 模块拆分（守 200 行 / 50 行函数）

- Rust：`envcheck.rs`（检查项与系统快照）、`runtime_install.rs`（下载/校验/解压/换源）、
  `onboarding.rs`（状态机与步进）、`repair.rs`（修复动作）；`commands.rs` 只做命令注册。
- 前端：`components/onboarding/`（WizardShell、CheckList、FixCard、StepMode、StepHome、StepLaunch）、
  `hooks/useOnboarding.ts`、`hooks/useEnvCheck.ts`；主界面加 `EnvStatusBar` 与修复中心抽屉。
- 文案常量独立 `src/i18n/zh.ts`，错误模板集中管理。

## 8. 分发注意

- macOS：.dmg 自包含（WKWebView 系统自带），无额外依赖。
- Linux：新装系统常缺 webkit2gtk，须发 AppImage（捆绑运行库）或 deb/rpm 并声明依赖；
  列入 O2 验收环境（干净容器）。
- v1 仍 macOS/Linux；Windows 后补（node.exe zip 免安装，思路同 §1）。

## 9. 里程碑（接 DESIGN.md 表）

| 阶段 | 内容 | 验收 |
|---|---|---|
| O1 | envcheck 全量/预检 + 运行时安装（双源/校验/换源） | 无 node 容器内装好运行时并落盘 runtime.json；cargo test 过 |
| O2 | 首跑向导全流程（断点续跑）+ 首启健康判定 | 删 `~/.dsh-launcher` 后裸装，一路默认走到成功徽章 |
| O3 | 修复中心 + 镜像/系统 Node 设置 | 人为删 runtime、断网，主界面出卡片且一键修复成功 |