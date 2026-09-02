# Lessons

<!-- 一条经验一行。格式:当 X 发生时,修复是 Y。skill 没提前警告我。 -->

- 当一个 run_code 程序里串了多个 edit 而中途抛错时,后续 edit 根本没执行:先 read 回文件核对哪些改动真实落了地,再补漏,别按记忆重放全部 edit(会重复或错位)。
- 当用 edit 工具时,old_string 必须逐字取自当前文件:把「想要的新内容」误粘进 old_string 会 not-found;报错先 diff 意图与文件现状。
- 当改造大文件怕切坏:先 `git show HEAD:<path>` 留底,改造后用特征函数名(startProfile 等)做完整性断言再写盘——本会话切 api.ts 时差点把后半段函数全删掉。
- 当写 run_code 程序时,tools.bash 的 `description` 是必填参数,漏了整个程序直接抛异常(白跑一步);每个内嵌调用都带全参数。
- 当仓库有 check-size 门禁(函数≤50行/文件≤200行)时,新组件按「hook 管状态 + 渲染组件」起手,别等门禁红了再拆。