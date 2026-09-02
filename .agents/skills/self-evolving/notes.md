# Session Notes

## 2026-09-02 统一工具台定位升级(TR1)

- 哪个坑浪费了最多时间?api.ts 拆分时按 marker 切掉文件后半段,靠 `git show HEAD` 恢复;以及 check-size 门禁事后拆组件(约 4 个来回)。
- skill 有没有提前警告我?没有——本 skill 当时还是空的;本轮回填了 run_code edit 批次失败、marker 切割、门禁先行拆分三条。
- 重来一次会怎么做?先跑一遍 `pnpm check:size` 看规则再动手;对接外部 API 前先 grep 服务端路由表(spec 可能没跟上实现);所有改文件操作单文件单程序,出错面小。
