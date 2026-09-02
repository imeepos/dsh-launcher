# Known Issues

<!-- 格式:症状 → 原因 → 修法。排查超过 5 分钟的 bug 才值得记。 -->

- 2026-09-02 release-platform 目录对接:按 OpenAPI 找 `GET /v1/versions`、`GET /v1/components` 想列版本/组件 → 实际两路由只有 POST(创建),GET 返回 405(102 实测),OpenAPI 也确实未定义 → 浏览改走 `GET /v1/releases`(带 version_id/artifact_ids)+ `GET /v1/artifacts?version_id=&os=&arch=`;平台将来补 GET 端点再升级。
- 2026-09-02 对 102 调 release-platform API:带 `X-Tenant-ID`/`X-Subject` dev headers 得 `UNAUTHENTICATED: authorization header required` → 102 部署没开 RELEASE_DEV_AUTH,必须 JWT;用 `IDENT_ISSUER_URL=http://192.168.0.102:38086/oauth scripts/issuer-token.sh admin admin-pw` 换 JWT(见 techniques)。
- 2026-09-02 ureq 2.x:请求级 `.timeout_connect()` 编译报 no method → 只有 `ureq::AgentBuilder::new().timeout_connect(..)` 可用;单请求要不同超时就另建 agent(下载用连接超时 10s、无整体超时)。
- 2026-09-02 本机 cargo 不在 PATH(brew 装在 ~/.cargo/bin)→ `PATH=$HOME/.cargo/bin:/opt/homebrew/bin:$PATH cargo test`。