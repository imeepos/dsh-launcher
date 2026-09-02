# UI 设计 v2(装机引擎 + 环境管家)

> 状态:待确认。前置:DESIGN-PRODUCT.md(宪章)与 DESIGN-DB.md v2(已确认)。
> 设计原则:**普通人第一眼就会用**——大按钮、说人话、过程透明、出事能退。

## 1. 信息架构

```text
顶栏:  AI 装机管家 | [首页] [AI 助手] [环境] [备份] [设置]        主题切换
底部:  动作流水停靠条(agent_actions 实时,点击展开)
首启:  三步向导全屏接管(欢迎 → 体检 → 选 AI 工具装)仅一次
```

| 视图 | 服务域(表) | 定位 |
|---|---|---|
| 首页 | host_profile, probe_*, components, installs, agent_tasks | 一眼看懂电脑能不能用 AI,一键装好 |
| AI 助手 | agent_sessions/messages/tasks/actions | 对话式排障与执行,过程透明 |
| 环境 | components/versions/installs/shims/env_edges | 管理已装与可装的一切 |
| 备份 | snapshots / restores | 保命能力 |
| 设置 | engine_config, settings | 引擎/网络/策略 |

## 2. 视图明细

### 2.1 首页(默认视图)

```text
┌────────────────────────────────────────────────────┐
│  你的电脑准备好用 AI 了吗?                [重新体检] │
│  ● 就绪(3 分钟前体检)  缺 Node.js · 磁盘充足       │
├────────────────────────────────────────────────────┤
│  选一个 AI 工具,我帮你装好:                          │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐  │
│  │ dsh     │ │Claude   │ │Codex    │ │Gemini   │  │
│  │ ✓可安装  │ │ Code    │ │ CLI     │ │ CLI     │  │
│  │         │ │✓可安装   │ │⚠需 Node │ │⚠需 Node │  │
│  └─────────┘ └─────────┘ └─────────┘ └─────────┘  │
│        [ 一键装好所选工具(含全部依赖) ]              │
├────────────────────────────────────────────────────┤
│ 最近动态: 09-30 装好 Claude Code · 09-30 体检通过    │
└────────────────────────────────────────────────────┘
```

- 状态行:probe_runs 最新 overall + 失败/警告项(check_key 人话映射)。
- 工具卡:components(kind='ai_tool');依赖满足度由 component_deps 对 installs 实时求值
  → ✓可安装 / ⚠缺 X / ✓已装(显示版本与打开方式)。
- [一键装好] → 创建 agent_task(goal=所选工具) → 跳计划确认对话框(§2.2)。

### 2.2 计划确认与执行(全产品核心交互,模态)

```text
要做什么: 装好 Claude Code(含依赖)
┌ 步骤预览 ────────────────────────────────────┐
│ 1. 安装 Node.js 22 LTS(缺)      约 40MB     │
│ 2. 写入 PATH(shims 目录)                    │
│ 3. npm 安装 @anthropic-ai/claude-code        │
│ 4. 验证 claude --version                     │
│ 每步开始前自动备份;失败自动回滚到初始状态      │
└──────────────────────────────────────────────┘
            [取消]  [开始执行]
执行中 → 步骤逐条点亮(ActionTimeline),失败步红显+原因+已自动回滚提示
完成 → 交接报告:装了什么/命令怎么用(claude)/在哪卸载
```

- auto_apply=false(默认)必经确认;设置开启后自动跳过(执行中仍可暂停)。

### 2.3 AI 助手

```text
[会话列表 ▾]  新对话
┌ 对话区 ──────────────────────┬ 任务侧栏 ─────────┐
│ 用户: 我要装 codex            │ 任务: 装好 codex   │
│ 助手: 缺 Node,计划 3 步…      │ 状态: 等待确认     │
│ (计划/动作以卡片嵌入对话流)    │ [查看计划] [确认]  │
└──────────────────────────────┴───────────────────┘
输入框: [_________________________________] [发送]
```

- 输入即目标:引擎可反问澄清;凡产出可执行任务走同一计划→确认→执行管道。
- 动作卡片实时反映 agent_actions 状态;失败卡片带 [查看日志] 与 [重试]。

### 2.4 环境

```text
[运行时 (3)] [AI 工具 (4)] [其他 (1)]        [立即体检]
▾ 运行时
  ● Node.js 22.11.0   managed · active   [设为默认] [卸载] [重装]
  ○ Node.js 20.18.0   managed · inactive [设为默认] [卸载]
  ○ Node(系统已装 v18)  检测 · 只读 [收编(即将支持)]
▾ AI 工具
  ● dsh 0.1.1-rc.2    managed · active   命令: dsh  [卸载] [重装]
```

- 行数据:installs JOIN component_versions;状态徽标(active/inactive/broken/检测)。
- broken 行红显 + [修复](=重装)与 [查看动作记录]。
- system 组只读,收编按钮置灰 tooltip「即将支持」;kind 分页签;hover 高亮依赖链。

### 2.5 备份

```text
[立即备份]  策略: 自动快照保留 20 份 · 手动永久(设置中改)
┌──────────────┬────────┬──────────┬────────┐
│ 时间          │ 类型    │ 范围      │ 大小    │
│ 11:02 装Node前│ 自动    │ 全部      │ 38 MB  │ [恢复] │
│ 10:40 (手动)  │ 手动    │ 托管目录   │ 21 MB  │ [恢复][删除]│
└──────────────┴────────┴──────────┴────────┘
恢复需二次确认;恢复过程显示步骤与结果(成功/失败+原因)。
```

### 2.6 设置

- **AI 引擎**:provider/model/base_url、API Key(仅存 .env 引用名,[测试连接]),
  auto_apply 开关(关=每次计划需确认)。
- **网络**:npm registry 镜像、下载镜像、代理。
- **策略**:快照保留(20 份/手动永久)、体检提醒。
- **关于**:版本、托管目录大小统计与 [打开目录]。

## 3. 数据绑定与命令面

启动:`db_migrate` → 首检(自动 probe + 识别既有安装入 installs)。

| 视图 | 读命令 | 写命令 |
|---|---|---|
| 首页 | probe_latest, list_ai_tools(含依赖满足度) | probe_start, create_task(goal) |
| 计划/执行 | task_get, actions_list | confirm_task, cancel_task, retry_task |
| AI 助手 | list_sessions, messages_list, list_tasks | session_create, message_send |
| 环境 | list_components(with_installs) | uninstall_install, reinstall_install, activate_install |
| 备份 | list_snapshots | create_snapshot, restore_snapshot, delete_snapshot(manual) |
| 设置 | engine_get_config, settings_all | engine_set_config, settings_set, snapshot_purge_now |

事件:`probe-updated`、`task-updated`(动作状态变化)、`action-log`(底部流水)。
引擎循环为后台任务,UI 只读 DB + 订阅事件,不发命令改状态(除 confirm/cancel/retry)。

## 4. 组件规划

| 组件 | 来源 | 说明 |
|---|---|---|
| AsyncButton / SubmitButton / ToastHost / ThemeToggle / Modal 族 | 现有 | 保留 |
| ProbeStatusCard / ToolPickCard / HandoffReport | 新增 | 首页三件套 |
| PlanReviewDialog / ActionTimeline | 新增 | 核心交互 |
| ChatPanel / TaskCard | 新增 | AI 助手 |
| InstallTree(替代 VersionTable) | 重写 | 环境 |
| SnapshotTable | 新增 | 备份 |
| ActionDock(替代 ConsoleDock) | 改造 | 底部动作流水 |
| SetupWizard | 改造 onboarding 向导 | 三步壳 |

## 5. 交互红线

1. auto_apply=false 时,任何变更动作未经确认不得执行(引擎侧同约束,双保险)。
2. 变更动作前置快照失败 → 该动作不得开始,任务转 failed 并提示手动备份。
3. 卸载/恢复二次确认必须展示影响面(槽位/命令/配置改动数,查 env_edges 计数)。
4. 断网:体检项逐条标注网络失败而非全局失败;下载类动作可重试且不重复落盘。
5. 所有失败可见:动作失败必有 error 文案 + 日志入口;禁止静默吞错。

## 6. 里程碑

| 阶段 | 内容 | 验收 |
|---|---|---|
| UR1 | 16 表迁移 + 探针框架 + 首页只读(体检/工具卡) | 裸机首启:体检出真实结果,依赖求值正确 |
| UR2 | 引擎循环(计划→确认→执行→复检→回滚)+ 首页装机闭环 | 删掉 Node 后「一键装好 Claude Code」走通;中途杀进程可恢复 |
| UR3 | 环境视图 + 卸载/重装/切换 + 备份视图 | 卸载后 shell 无残留(shims/PATH 验证);快照可恢复 |
| UR4 | AI 助手对话 + 设置 + 三步向导壳 | 无 Node 裸机按向导三步装好 dsh;对话可发起装机任务 |
