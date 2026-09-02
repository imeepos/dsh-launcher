# UI 设计(基于已确认的 DESIGN-DB.md)

> 状态:待确认。库表模型已于 2026-09-02 确认,本文档定义信息架构、视图与数据绑定、
> 组件规划与交互规则;确认后进入实现(UR1–UR4)。

## 1. 信息架构:四个主视图 + 一个专属分组

```text
顶栏: CLI 工具台 | [工具库] [目录] [运行记录] [DSH] [设置]        主题切换
内容区: 视图面板
底部: ConsoleDock(日志停靠台,全局)
```

| 视图 | 服务域(表) | 默认 |
|---|---|---|
| 工具库 | tools / tool_installations / tool_tags / runs(实时态) | **是**(定位主表面) |
| 目录 | rp_connections / rp_products / rp_releases / rp_artifacts | |
| 运行记录 | runs / run_log_lines | |
| DSH(code='dsh' 专属) | dsh_homes / dsh_profile_configs + runs | |
| 设置 | settings / rp_connections(复用) / 日志保留策略 | |

现有视图映射:启动台+Homes → 合入「DSH」;版本库 → 演进为「工具库」;目录 → 保留改数据源;
新增「运行记录」「设置」。首跑向导保持全屏接管,UR4 分模式。

## 2. 视图明细

### 2.1 工具库(默认视图)

```text
[搜索框___________] [标签▾] [全部来源▾]        [+ 手动登记] [从目录安装]
┌──────────────────────────────────────────────┐
│ ▾ dsh  (DSH · 3 个安装 · 1 运行中)      [标签] │  ← 工具卡(可折叠)
│   ├ v0.1.1-rc.2  npm    指纹 0.1.1  ● 运行中  │  ← 安装行
│   │   [启动台…] [指纹] [删除]                 │
│   └ dev-repo     dev    未采集                │
├──────────────────────────────────────────────┤
│ ▸ releasectl (2 个安装)                [标签] │  ← 折叠态
└──────────────────────────────────────────────┘
```

- **工具卡**:display_name + code 徽标 + 安装数 + 运行中数(实时查 runs ended_at IS NULL);
  标签 chips(来自 tool_tags,点击即筛选)。
- **安装行**:version_label + source 徽标(npm/dev/manual/platform)+ fingerprint;
  操作按 run_mode:`service` → [启动/停止](长驻,起停即走);`oneshot` → [运行…](弹参数对话框);
  通用操作:[指纹] [删除](级联删 runs,二次确认提示影响面)。
- **空态**:无工具 → 引导「从目录安装 / 手动登记」;无网络连接不影响本地工具。
- 来源 `platform` 的行显示「可升级」徽标位(TR3:比对 rp_artifacts.fetched 版本)。

### 2.2 目录

沿用 TR1 已实现三步流,数据源从实时 API 改为**缓存表 + TTL 刷新**:

- 连接配置(RpSettingsForm)不变;连接成功 → 写 rp_connections + 触发缓存刷新。
- 浏览读 rp_releases / rp_artifacts;面板角标显示缓存龄(`x 分钟前`,>10 分钟自动重拉,
  失败显示旧数据 + 黄条提示——失败路径必须可观测)。
- 安装动作不变(download-url → 校验 → 登记 → tool_installations.artifact_id 溯源)。

### 2.3 运行记录

```text
[工具▾] [类型: 全部|工具|DSH] [状态: 全部|运行中|已结束] [日期____]
┌────────┬──────────┬──────┬──────────┬────────┐
│ 时间    │ 工具/安装 │ 类型  │ 参数      │ 结果    │
│ 11:02  │ releasectl│ 工具  │ --help   │ exit 0  │
│ 10:55  │ dsh@rc.2  │ DSH   │ —        │ ● 运行中│
└────────┴──────────┴──────┴──────────┴────────┘
点击行 → 展开日志面板(run_log_lines,带 is_err 红染,加载上限 500 行+加载更多)
```

- 运行中行实时更新(process-exit 事件驱动);[导出日志] 落文件。

### 2.4 DSH(专属分组)

- 子标签:[启动台] [Homes];内容 = 现有 LaunchPad / HomesPanel 原样迁入,
  数据源改 dsh_homes + tool_installations(tool.code='dsh')。
- 启动即写 runs(context='dsh_profile');Home 绑定/最后成功启动写回 FK。
- 视图入口仅在存在 code='dsh' 工具时显示(未装 dsh 不出现空分组)。

### 2.5 设置

- 通用:日志保留(30 天/万行,只读展示策略 + [立即清理])、主题。
- 安装源:npm registry 覆盖、node 镜像、系统 Node 开关(settings KV)。
- 连接:rp_connections 编辑(复用 RpSettingsForm,从目录抽离)。
- 关于:版本、registry.json 导入状态(已导入/未导入/无旧文件)。

## 3. 数据绑定与命令面(前后端契约)

启动时:`db_migrate`(自动建表/升级)+ `registry_import`(检测旧文件一次性导入)。

| 视图 | 读 | 写(命令) |
|---|---|---|
| 工具库 | list_tools(with_tags, running_counts), list_installations | upsert_tool, delete_tool, set_tool_tags, add_manual_installation, remove_installation, fingerprint_installation, start_run / stop_run |
| 目录 | rp_list_* (读缓存,后台刷新) | rp_connect / rp_refresh_cache / rp_install_artifact |
| 运行记录 | list_runs(filter), get_run_log(run_id, after_seq) | export_run_log, delete_run |
| DSH | list_homes, list_profiles(动态) | add/create/clone/bind/remove_home, start_profile(内部写 runs), stop_run |
| 设置 | settings_all, rp_get_config | settings_set, rp_set_config, logs_purge_now |

事件沿用:process-log / process-exit / install-progress;新增 `runs-changed`(历史落库后通知
运行记录视图刷新)。

## 4. 组件规划(复用优先)

| 组件 | 来源 | 动作 |
|---|---|---|
| AppShell / ConsoleDock / ToastHost / 主题切换 | 现有 | 保留 |
| ToolCard / InstallationRow | 新增(替代 VersionTable) | 按 §2.1 |
| RunDialog | ToolRunDialog 演进 | run_mode='oneshot' 用 |
| RunHistoryTable / RunLogView | 新增 | §2.3 |
| HomesPanel / LaunchPad | 现有 | 迁入 DSH 分组,数据源换 SQL |
| RpSettingsForm / ReleaseTable / ArtifactTable | TR1 现有 | 数据源换缓存表 |
| SettingsPane | 新增 | §2.5 |

## 5. 交互红线

- 破坏性操作(删工具/删安装/清日志)二次确认并显示级联影响行数。
- 缓存过期/刷新失败不得白屏:显示旧数据 + 黄条;失败打点 toast。
- 运行态以 DB(runs.ended_at)+ 实时事件双源判定,事件丢失可由 list_runs 兜底纠正。
- 视图切换保持挂载(沿用现有显隐方案),保留筛选与表单状态。

## 6. 里程碑

| 阶段 | 内容 | 验收 |
|---|---|---|
| UR1 | rusqlite 接入 + 13 表迁移 + registry 导入 + 工具库主视图 | 删 launcher.db 裸启可建库;旧 registry.json 导入后数据一致 |
| UR2 | 运行记录视图 + 日志保留策略 + 设置页 | 起停一次工具后运行记录可查可导出 |
| UR3 | 目录切缓存表 + TTL 刷新 + DSH 分组迁移 | 断网仍可浏览缓存;DSH 启停写 runs |
| UR4 | 首跑向导分模式 + 可升级徽标(TR3 合流) | 无 dsh 环境装任一平台工具不触发 DSH 向导 |
