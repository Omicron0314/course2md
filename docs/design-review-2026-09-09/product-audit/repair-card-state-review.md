# 补做已完成但最近卡片仍显示部分完成

日期：2026-09-09。任务：`task-18d3a0c3143ed5a0-b626-1`。证据：`v3-05-scoped-repair-running.jpg`、当前隔离配置及 `before-repair-*.json`。本次未操作 GUI、未修改隔离配置、未提交。

## 只读比对

- 新任务状态为 `complete`，`error` 为空。操作是 `reprocess`，目标仅 `proofreading`、`summary`，基于 `task-18d39b72aa8c0920-17c71-0` 的已发布版本。
- 绑定新服务版本 `service-version-59b87dc3-74d0-40f3-9cf1-f1e8d8e95e7b`，服务名“恢复校对服务”，模型 `fixture-summary`，地址仍是隔离服务 `http://127.0.0.1:18582/v1/chat/completions`，无需认证。
- 新 `manifest.json` 的 `partial` 为 `false`；正文、截图、校对、摘要 outcomes 均为 `succeeded`，校对为 1/1；未请求额外导出。`document.json` 含一个正文 section 及 summary 字段。课程根的 `current.json` 已指向新版本。
- `generation.json` 与保存前完全相同；服务 `defaults` 与保存前完全相同，仍绑定原默认版本 `service-version-bf9e0bf4-cc91-409c-bd92-56e434e2063b`。所有原任务的 `plan` 与保存前完全相同。

因此，补做没有把正文重新识别，也没有修改默认服务或旧计划。此次残留不是新产物的质量状态。

## 原因与修复

`notes::Course::description` 直接根据卡片持有的 `manifest.partial` 追加“部分内容待补全”。`finish_task` 在保存新终态后马上显示完成通知，但仅调用 `refresh_library` 去异步扫描所有登记位置；`self.courses` 直到整次扫描返回仍保留旧版本。当前截图中的旧文案与这个中间窗口一致。另一位置较慢时，这种不一致可以持续更久；用户没有理由知道这两个区域还在分别更新。

这属于真实的显示状态同步缺口。已在 `task_ui.rs` 的完成路径加入 `update_published_course`：已确认发布的新版本先更新同一课程存储目录的内存卡片，再进行原有异步全库扫描。卡片使用新 manifest、版本目录、内容数量及真实封面；保留该课程已有的用户命名。只按完整存储目录替换，不按标题或来源 ID 跨库替换。

原来的异步扫描代数守卫保留；补做提交语义、默认设置、原任务计划和旧文件均未改。

## 验证

新增回归 `a_published_repair_replaces_the_stale_card_before_a_library_scan`，在全库扫描尚未参与时验证新卡立刻不再显示 partial、阅读目标是新目录、保留用户命名与新封面、另一库的同名同来源课程不受影响、旧快照仍为 partial。`rustfmt` 和 `git diff --check` 已通过。

完整桌面测试使用实际 Apple 工具链 shim：**213 passed，0 failed，3 ignored**。日志 `desktop-unit-tests-v4.log`，包含设置的最新明确拒绝/仅保存测试、reader 最新变更、卡片同步及后台模型阶段文案回归。首次运行时阶段文案回归发现 `Downloading` 的子串会误判为 loading，已改为独立单词匹配后重跑完整集。新版原生仍应观察一次补做完成时通知与最近卡片同时更新，不将重新启动后的新数据当作这个瞬间的验证。
