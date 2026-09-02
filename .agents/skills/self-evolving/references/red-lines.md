# Red Lines

<!-- 格式:禁止 X,因为 Y 发生过。真的付出过代价才记。 -->

- 禁止在 run_code 里用「从某 marker 切到文件尾」的方式删除大段代码,除非先确认 marker 之后没有任何要保留的内容——本会话按 marker 切 api.ts 把 process/profile 封装全切没了,靠 git HEAD 底稿才恢复。
- 禁止在多步骤 edit 批次抛错后凭记忆重放整批:未执行与已执行的 edit 混着重放会造成错位重复。