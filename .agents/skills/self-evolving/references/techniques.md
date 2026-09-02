# Techniques

<!-- 排查技巧、工具命令、调试手法。格式:什么场景 → 怎么用。 -->

- 需要对 102 的 release-platform 做带认证的 API 验证:在 release-platform 仓库跑 `TOKEN=$(IDENT_ISSUER_URL=http://192.168.0.102:38086/oauth scripts/issuer-token.sh admin admin-pw) && curl -H "Authorization: Bearer $TOKEN" http://192.168.0.102:38080/v1/...`;issuer 是 web nginx 同源反代的 jwks-issuer。
- 快速核对 Go 服务的真实路由表(OpenAPI 与实现可能不同步):`grep -n 'mux.HandleFunc' internal/api/server.go`,一行一个方法+路径,比翻 spec 快且准。
- 从 OpenAPI 抽请求/响应形状做对接设计:python3 + yaml.safe_load 打印 paths 的 method/params/200 schema,30 秒拿到字段清单(sha256 必填、download_url 信封等)。