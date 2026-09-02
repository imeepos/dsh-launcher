# 统一工具台设计(dsh-launcher 定位升级)

> 2026-09 定位变化:本应用不再只是「DSH 启动器」,而是**为所有命令行工具、无界面工具、
> 非客户端 GUI 工具提供统一可视化管理的桌面应用**。DSH 是其中第一个被完整管理的工具,
> 而不是应用的全部。
>
> 持久化模型升级:registry.json → SQLite,库表/关系/字段见 **DESIGN-DB.md(待确认)**;
> UI 设计以 DB 模型确认为前置。

## 1. 概念模型(两层管理链路)

```text
通用链路(本设计新增):
  Tool(工具,如 dsh / releasectl /任意 CLI)
    └─ Installation(安装/登记,复用 VersionEntry,新增 tool 字段)
         └─ Run(运行:start_tool,任意 args + cwd,单实例锁)

DSH 专用链路(保留,不动):
  Home(profile 容器) ─ Profile ─ start_profile(SIGTERM/运行锁/日志流)
```

- `VersionEntry.tool`(可选,缺省视为 `dsh`):工具归属名。旧 registry.json 无此字段,零迁移。
- 通用运行键:`__tool__/<versionId>`,与 DSH 的 `homeId/profile` 共用 ProcessMap、
  process-log / process-exit 事件与日志停靠台;停止复用 stop_profile 语义(SIGTERM)。
- 通用运行锁:`~/.dsh-launcher/run/tool-<versionId>.lock`(fs4,同 (home,profile) 锁语义)。

## 2. release-platform 对接

平台是「版本、发布与分发控制面」(102 部署:http://192.168.0.102:38080)。
本应用作为**客户端侧的可视化安装器/管理器**接入:

### 2.1 目录浏览(只读)

| 平台 API | 用途 |
|---|---|
| `GET /v1/products` | 工具目录(产品 = 对外发布的工具) |
| `GET /v1/components?product_id=` | 产品下组件 |
| `GET /v1/versions?component_id=` | 可安装版本(按时间倒序) |
| `GET /v1/artifacts?version_id=&os=&arch=` | 匹配本机平台的制品 |

列表信封 `{items,total,limit,offset}`,limit≤500。os/arch 本机默认值由
`std::env::consts` 映射:`macos→darwin`、`aarch64→arm64`、`x86_64→amd64`,UI 可改。

### 2.2 安装流(写路径,唯一)

```text
选制品 → POST /v1/artifacts/{id}/download-url → {url}
       → 流式下载到 ~/.dsh-launcher/tools/<tool>/<文件名>(install-progress 事件逐行)
       → chmod +x(unix)
       → 登记 VersionEntry{kind:manual, tool:<产品名>, spec:<semver>, bin:<落盘路径>}
       → 用户点「指纹」实测 <bin> --version 写回(复用现有命令)
```

付费门控(`DEVICE_REQUIRED` / `ENTITLEMENT_REQUIRED`)错误原样透传 UI,本期不做设备注册。

### 2.3 认证(三模式,存 Settings.rp)

102 部署强制 JWT(`UNAUTHENTICATED: authorization header required`),dev-header 仅本地
`RELEASE_DEV_AUTH=true` 的服务可用:

| mode | 字段 | 行为 |
|---|---|---|
| `password`(推荐) | issuerUrl + username + password | `POST {issuer}/token` OAuth password grant 换 JWT,token 仅驻内存 |
| `bearer` | token(rpat_ 或 JWT) | 直接 `Authorization: Bearer` |
| `devHeaders` | tenant + subject | `X-Tenant-ID` + `X-Subject`,仅本地联调 |

password/bearer 凭据明文存 `~/.dsh-launcher/registry.json`(本机自用工具,与 .env 同级风险,
文档明示);JWT 不落盘。

### 2.4 HTTP 客户端

`ureq`(default-features=false,纯 http)。102 为内网 http;如需 https 再开 tls feature。
超时:API 15s;下载连接 10s、读不设全局超时(大制品)。

## 3. 品牌与命名

- 窗口/页面标题:CLI 工具台;「版本库」更名「工具库」。
- identifier/productName 本期不变(`com.imeepos.dsh-launcher`),避免打包/升级路径断裂;
  更名作为独立事项另起。
- DSH 专属表面(首跑向导、Homes、启动台)保留原样,标「DSH」前缀以免与通用能力混淆。

## 4. 里程碑

| 阶段 | 内容 | 验收 |
|---|---|---|
| TR1(本期) | tool 字段 + rp 客户端(目录浏览/下载安装)+ start_tool + 目录视图 | 对 102 密码授权拉取产品/版本/制品并完成一次安装登记,cargo test 过 |
| TR2 | 通用运行 UI(运行对话框:args/cwd/停止/日志)+ 工具库按 tool 分组筛选 | 任意登记的 CLI 从 UI 起停,日志进停靠台 |
| TR3 | 更新检查:本地登记 spec/指纹 vs 平台 versions;一键升级=重新安装覆盖 | 平台发新版后工具库出「可升级」徽标 |
| TR4 | 首跑向导分模式(DSH 深度链路 vs 通用工具即装即用)+ 设备注册/付费门控 | 萌新装任一平台工具不走 DSH 向导 |

## 5. 红线

- 平台错误码透传不吞(INVALID_* / UNAUTHENTICATED / DEVICE_REQUIRED / ENTITLEMENT_REQUIRED)。
- 下载失败不得留半截文件冒充安装:先写 `.part`,校验 sha256(登记里有)通过后改名。
- 不在主分支直接改代码;registry 变更走 with_registry 串行化。
