# Red Lines

<!-- 格式:禁止 X,因为 Y 发生过。真的付出过代价才记。 -->

- 禁止在 run_code 里用「从某 marker 切到文件尾」的方式删除大段代码,除非先确认 marker 之后没有任何要保留的内容——本会话按 marker 切 api.ts 把 process/profile 封装全切没了,靠 git HEAD 底稿才恢复。
- 禁止在主树里执行属于 feature 分支的 git 写操作(git rm/git add 等):本会话在主树 git rm 两份文档,删除只进暂存区未提交,造成主树悬挂状态 + worktree 里旧文件仍存的分裂,直到 ff-merge 前核对才暴露;worktree 命令一律带 workdir 指到对应 worktree,merge 前必查主树 `git status` 干净。
- 禁止在多步骤 edit 批次抛错后凭记忆重放整批:未执行与已执行的 edit 混着重放会造成错位重复。
- 禁止对密钥文件(.env 等)运行任何会输出内容的提取命令,即使宣称「只取 key 名」:行级过滤挡不住多行带引号的值,2026-09-04 sed 去 value 实际漏出明文敏感行;核对密钥文件只允许 sha/cmp/存在性判断,永不打印内容或切片。