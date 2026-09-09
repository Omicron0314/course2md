# 本轮原生构图证据

主报告见[构图复盘与页面矩阵](../../UX-COMPOSITION-REVIEW-2026-09-09.md)。原图复制保留编码字节；SHA-256、像素尺寸、构建组及排除说明见 [manifest.json](manifest.json)。

当前归档 242 张原图；另有 3 张小型文件选择器调试 JPEG 仅记入 manifest，未复制。历史失败与修复复拍一并保留。

工具日志按原始字节保存，包含格式检查输出中的差异上下文空格和测试工具的末尾空行；本目录的 `.gitattributes` 仅对这些 `.log` 文件关闭 Git 空白告警，不修改日志内容。

**旧失败图不是最终结果。** before／用户拒绝 PNG 是失败基线，after／check03／check04／check06 包含随后修复的问题，final-01 文件名也不代表最终通过。check07／check08 只按逐图记录中的状态和范围使用；性能仪器构建与最后安装身份分开记录。

- [转换／任务独立审查](conversion-review.md)
- [库／阅读／图库独立审查](reader-review.md)
- [设置／引导独立审查](independent-settings-guide-check04.md)
- [桌面最新已归档测试日志](tests-desktop-final.log)、[核心任务测试](tests-task-execution.log)
- [最终构建身份](final-build.json)、[安装记录](installation.json)、[最终 release 构建日志](build-release-final.log)
- [本轮 15 个 Rust 模块格式检查](format-changed-final.log)、[未通过的全仓格式检查原始差异](format-final.log)
- [性能构建身份](performance-build.json)、[性能摘要](performance-summary.json)、[该 PID 原始 trace](release-performance-trace.jsonl)

需要纠正的文件名：check03-18 是二维码就绪，check04-05 是图库，check06-01 是库，check06-18 是处理中，check08-07 是阅读入场中间帧。详情在 manifest 的 interpretation 字段。

## user-rejected

historical rejected task/plan/completion/detail composition

| 原图 | 像素尺寸 | 特别说明 |
| --- | --- | --- |
| [codex-clipboard-853e00e9-65d8-40e3-9702-128f75ad0ad4.png](codex-clipboard-853e00e9-65d8-40e3-9702-128f75ad0ad4.png) | 3818×2192 | rejected running-task composition; never final acceptance evidence |
| [codex-clipboard-4f9b8add-ec3e-4695-a19e-f6d1f83ab556.png](codex-clipboard-4f9b8add-ec3e-4695-a19e-f6d1f83ab556.png) | 3950×2324 | rejected redundant plan/confirmation page; never final acceptance evidence |
| [codex-clipboard-3988cc1c-dc49-4426-a124-e14bb2877f6a.png](codex-clipboard-3988cc1c-dc49-4426-a124-e14bb2877f6a.png) | 3950×2324 | rejected completion intermediary page; never final acceptance evidence |
| [codex-clipboard-2571ffa1-6b46-4aa2-83ac-d7857d865b9b.png](codex-clipboard-2571ffa1-6b46-4aa2-83ac-d7857d865b9b.png) | 3950×2324 | rejected task-detail composition; never final acceptance evidence |

## before

historical baseline; contains rejected composition

| 原图 | 像素尺寸 | 特别说明 |
| --- | --- | --- |
| [before-01-onboarding.jpg](before-01-onboarding.jpg) | 1068×768 | 见主报告及独立记录 |
| [before-02-engines.jpg](before-02-engines.jpg) | 1068×768 | 见主报告及独立记录 |
| [before-03-ai.jpg](before-03-ai.jpg) | 1068×768 | 见主报告及独立记录 |
| [before-04-account.jpg](before-04-account.jpg) | 1068×768 | 见主报告及独立记录 |
| [before-05-model.jpg](before-05-model.jpg) | 1068×768 | 见主报告及独立记录 |
| [before-06-workbench.jpg](before-06-workbench.jpg) | 1068×768 | 见主报告及独立记录 |
| [before-07-library.jpg](before-07-library.jpg) | 1068×768 | 见主报告及独立记录 |
| [before-08-tasks.jpg](before-08-tasks.jpg) | 1068×768 | 见主报告及独立记录 |
| [before-09-settings-appearance.jpg](before-09-settings-appearance.jpg) | 1068×768 | 见主报告及独立记录 |
| [before-10-generation.jpg](before-10-generation.jpg) | 1068×768 | 见主报告及独立记录 |
| [before-11-services.jpg](before-11-services.jpg) | 1068×768 | 见主报告及独立记录 |
| [before-12-storage.jpg](before-12-storage.jpg) | 1068×768 | 见主报告及独立记录 |
| [before-13-application.jpg](before-13-application.jpg) | 1068×768 | 见主报告及独立记录 |

## after

intermediate; known failures retained

| 原图 | 像素尺寸 | 特别说明 |
| --- | --- | --- |
| [after-01-workbench.jpg](after-01-workbench.jpg) | 1068×768 | 见主报告及独立记录 |
| [after-02-appearance.jpg](after-02-appearance.jpg) | 1068×768 | 见主报告及独立记录 |
| [after-03-application.jpg](after-03-application.jpg) | 1068×768 | 见主报告及独立记录 |
| [after-04-guide-engine.jpg](after-04-guide-engine.jpg) | 1068×768 | 见主报告及独立记录 |
| [after-05-guide-engines.jpg](after-05-guide-engines.jpg) | 1068×768 | 见主报告及独立记录 |
| [after-06-guide-ai.jpg](after-06-guide-ai.jpg) | 1068×768 | 见主报告及独立记录 |
| [after-07-model-discovery.jpg](after-07-model-discovery.jpg) | 1068×768 | 见主报告及独立记录 |
| [after-08-model-menu.jpg](after-08-model-menu.jpg) | 1068×768 | 见主报告及独立记录 |
| [after-09-ai-checking.jpg](after-09-ai-checking.jpg) | 1068×768 | 见主报告及独立记录 |
| [after-10-ai-check-state.jpg](after-10-ai-check-state.jpg) | 1068×768 | 见主报告及独立记录 |
| [after-11-guide-account.jpg](after-11-guide-account.jpg) | 1068×768 | 见主报告及独立记录 |
| [after-12-guide-model.jpg](after-12-guide-model.jpg) | 1068×768 | 见主报告及独立记录 |
| [after-13-local-input.jpg](after-13-local-input.jpg) | 1068×768 | 见主报告及独立记录 |
| [after-14-local-selected.jpg](after-14-local-selected.jpg) | 1068×768 | 见主报告及独立记录 |
| [after-15-converting.jpg](after-15-converting.jpg) | 1068×768 | 见主报告及独立记录 |
| [after-16-conversion-phase.jpg](after-16-conversion-phase.jpg) | 1068×768 | 见主报告及独立记录 |
| [after-17-direct-reader.jpg](after-17-direct-reader.jpg) | 1068×768 | 见主报告及独立记录 |
| [after-18-reader.jpg](after-18-reader.jpg) | 1068×768 | 见主报告及独立记录 |
| [after-19-gallery.jpg](after-19-gallery.jpg) | 1068×768 | 见主报告及独立记录 |
| [after-20-viewer.jpg](after-20-viewer.jpg) | 1068×768 | 见主报告及独立记录 |
| [after-21-library.jpg](after-21-library.jpg) | 1068×768 | 见主报告及独立记录 |
| [after-22-reader-multisection.jpg](after-22-reader-multisection.jpg) | 1068×768 | 见主报告及独立记录 |
| [after-23-gallery-six.jpg](after-23-gallery-six.jpg) | 1068×768 | 见主报告及独立记录 |
| [after-24-task-complete.jpg](after-24-task-complete.jpg) | 1068×768 | 见主报告及独立记录 |
| [after-25-task-details.jpg](after-25-task-details.jpg) | 1068×768 | 见主报告及独立记录 |

## final

historical failed restored-input layout; final in filename is not final acceptance

| 原图 | 像素尺寸 | 特别说明 |
| --- | --- | --- |
| [final-01-workbench.jpg](final-01-workbench.jpg) | 1068×768 | Historical build03 restored input with duplicated source identity and actions; not a final state. |

## check03

intermediate; individual unchanged states can be referenced, not a final pass

| 原图 | 像素尺寸 | 特别说明 |
| --- | --- | --- |
| [check03-02-appearance.jpg](check03-02-appearance.jpg) | 1068×768 | 见主报告及独立记录 |
| [check03-03-application.jpg](check03-03-application.jpg) | 1068×768 | 见主报告及独立记录 |
| [check03-04-generation.jpg](check03-04-generation.jpg) | 1068×768 | 见主报告及独立记录 |
| [check03-05-generation-lower.jpg](check03-05-generation-lower.jpg) | 1068×768 | 见主报告及独立记录 |
| [check03-06-services.jpg](check03-06-services.jpg) | 1068×768 | 见主报告及独立记录 |
| [check03-07-storage.jpg](check03-07-storage.jpg) | 1068×768 | 见主报告及独立记录 |
| [check03-08-guide-engine.jpg](check03-08-guide-engine.jpg) | 1068×768 | 见主报告及独立记录 |
| [check03-09-guide-engines.jpg](check03-09-guide-engines.jpg) | 1068×768 | 见主报告及独立记录 |
| [check03-10-guide-ai.jpg](check03-10-guide-ai.jpg) | 1068×768 | 见主报告及独立记录 |
| [check03-11-ai-failure.jpg](check03-11-ai-failure.jpg) | 1068×768 | 见主报告及独立记录 |
| [check03-12-ai-running.jpg](check03-12-ai-running.jpg) | 1068×768 | 见主报告及独立记录 |
| [check03-13-narrow-workbench.jpg](check03-13-narrow-workbench.jpg) | 860×620 | 见主报告及独立记录 |
| [check03-14-ai-result.jpg](check03-14-ai-result.jpg) | 1068×768 | 见主报告及独立记录 |
| [check03-15-guide-bilibili.jpg](check03-15-guide-bilibili.jpg) | 1068×768 | 见主报告及独立记录 |
| [check03-16-narrow-library.jpg](check03-16-narrow-library.jpg) | 860×620 | 见主报告及独立记录 |
| [check03-17-narrow-reader.jpg](check03-17-narrow-reader.jpg) | 860×620 | 见主报告及独立记录 |
| [check03-18-guide-login-loading.jpg](check03-18-guide-login-loading.jpg) | 1068×768 | Actual state: Bilibili QR is ready with countdown and scan/cancel actions; not initial loading. |
| [check03-19-narrow-search.jpg](check03-19-narrow-search.jpg) | 860×620 | 见主报告及独立记录 |
| [check03-20-guide-model.jpg](check03-20-guide-model.jpg) | 1068×768 | 见主报告及独立记录 |
| [check03-21-narrow-search-ready.jpg](check03-21-narrow-search-ready.jpg) | 860×620 | 见主报告及独立记录 |
| [check03-22-model-download.jpg](check03-22-model-download.jpg) | 1068×768 | 见主报告及独立记录 |
| [check03-23-narrow-gallery.jpg](check03-23-narrow-gallery.jpg) | 860×620 | 见主报告及独立记录 |
| [check03-24-model-progress.jpg](check03-24-model-progress.jpg) | 1068×768 | 见主报告及独立记录 |
| [check03-25-model-paused.jpg](check03-25-model-paused.jpg) | 1068×768 | 见主报告及独立记录 |
| [check03-26-narrow-viewer.jpg](check03-26-narrow-viewer.jpg) | 860×620 | 见主报告及独立记录 |
| [check03-27-wide-initial.jpg](check03-27-wide-initial.jpg) | 1330×768 | 见主报告及独立记录 |
| [check03-28-wide-library.jpg](check03-28-wide-library.jpg) | 1330×768 | 见主报告及独立记录 |
| [check03-29-wide-reader.jpg](check03-29-wide-reader.jpg) | 1330×768 | 见主报告及独立记录 |
| [check03-30-wide-gallery.jpg](check03-30-wide-gallery.jpg) | 1330×768 | 见主报告及独立记录 |
| [check03-31-wide-appearance.jpg](check03-31-wide-appearance.jpg) | 1330×768 | 见主报告及独立记录 |
| [check03-32-dark-theme-menu.jpg](check03-32-dark-theme-menu.jpg) | 1330×768 | 见主报告及独立记录 |
| [check03-33-light-theme-menu.jpg](check03-33-light-theme-menu.jpg) | 1330×768 | 见主报告及独立记录 |
| [check03-34-wide-150.jpg](check03-34-wide-150.jpg) | 1330×768 | 见主报告及独立记录 |
| [check03-35-services-wide.jpg](check03-35-services-wide.jpg) | 1330×768 | 见主报告及独立记录 |
| [check03-36-service-editor.jpg](check03-36-service-editor.jpg) | 1330×768 | 见主报告及独立记录 |
| [check03-37-service-editor-actions.jpg](check03-37-service-editor-actions.jpg) | 1330×768 | 见主报告及独立记录 |
| [check03-38-keyboard-focus.jpg](check03-38-keyboard-focus.jpg) | 1330×768 | 见主报告及独立记录 |

## check04

intermediate and targeted rechecks; later geometry fixes are not present

| 原图 | 像素尺寸 | 特别说明 |
| --- | --- | --- |
| [check04-01-restored.jpg](check04-01-restored.jpg) | 1068×768 | 见主报告及独立记录 |
| [check04-02-narrow-library.jpg](check04-02-narrow-library.jpg) | 860×620 | 见主报告及独立记录 |
| [check04-03-narrow-search-empty.jpg](check04-03-narrow-search-empty.jpg) | 860×620 | Actual state: empty search query, not a completed zero-match search. |
| [check04-04-narrow-search-result.jpg](check04-04-narrow-search-result.jpg) | 860×620 | 见主报告及独立记录 |
| [check04-05-narrow-viewer.jpg](check04-05-narrow-viewer.jpg) | 860×620 | Actual page: screenshot gallery; viewer has not been entered. |
| [check04-06-narrow-viewer-image.jpg](check04-06-narrow-viewer-image.jpg) | 860×620 | 见主报告及独立记录 |
| [check04-07-narrow-viewer-text.jpg](check04-07-narrow-viewer-text.jpg) | 860×620 | 见主报告及独立记录 |
| [check04-08-narrow-viewer-scroll.jpg](check04-08-narrow-viewer-scroll.jpg) | 860×620 | 见主报告及独立记录 |
| [check04-09-generation.jpg](check04-09-generation.jpg) | 1068×768 | 见主报告及独立记录 |
| [check04-10-generation-lower.jpg](check04-10-generation-lower.jpg) | 1068×768 | 见主报告及独立记录 |
| [check04-11-storage.jpg](check04-11-storage.jpg) | 1068×768 | 见主报告及独立记录 |
| [check04-12-about.jpg](check04-12-about.jpg) | 1068×768 | 见主报告及独立记录 |
| [check04-13-about-idle.jpg](check04-13-about-idle.jpg) | 1068×768 | 见主报告及独立记录 |
| [check04-14-guide-engine.jpg](check04-14-guide-engine.jpg) | 1068×768 | 见主报告及独立记录 |
| [check04-15-guide-ai.jpg](check04-15-guide-ai.jpg) | 1068×768 | 见主报告及独立记录 |
| [check04-16-guide-ai-result.jpg](check04-16-guide-ai-result.jpg) | 1068×768 | 见主报告及独立记录 |
| [check04-17-guide-account.jpg](check04-17-guide-account.jpg) | 1068×768 | 见主报告及独立记录 |
| [check04-18-model-start.jpg](check04-18-model-start.jpg) | 1068×768 | 见主报告及独立记录 |
| [check04-19-model-phase.jpg](check04-19-model-phase.jpg) | 1068×768 | 见主报告及独立记录 |
| [check04-20-model-paused.jpg](check04-20-model-paused.jpg) | 1068×768 | 见主报告及独立记录 |
| [check04-21-narrow-library-cards.jpg](check04-21-narrow-library-cards.jpg) | 860×620 | 见主报告及独立记录 |
| [check04-22-narrow-library-card-body.jpg](check04-22-narrow-library-card-body.jpg) | 860×620 | 见主报告及独立记录 |
| [check04-23-narrow-library-card-actions.jpg](check04-23-narrow-library-card-actions.jpg) | 860×620 | 见主报告及独立记录 |
| [check04-24-service-editor.jpg](check04-24-service-editor.jpg) | 1068×768 | 见主报告及独立记录 |
| [check04-25-service-editor-actions.jpg](check04-25-service-editor-actions.jpg) | 1068×768 | 见主报告及独立记录 |

## check06

mixed states, including failures corrected in check08

| 原图 | 像素尺寸 | 特别说明 |
| --- | --- | --- |
| [check06-01-narrow-reader-title.jpg](check06-01-narrow-reader-title.jpg) | 860×620 | Actual page: library, not reader/title evidence. |
| [check06-02-narrow-reader-title.jpg](check06-02-narrow-reader-title.jpg) | 860×620 | 见主报告及独立记录 |
| [check06-03-empty-workbench.jpg](check06-03-empty-workbench.jpg) | 1068×768 | 见主报告及独立记录 |
| [check06-04-empty-library.jpg](check06-04-empty-library.jpg) | 1068×768 | 见主报告及独立记录 |
| [check06-05-empty-tasks.jpg](check06-05-empty-tasks.jpg) | 1068×768 | 见主报告及独立记录 |
| [check06-06-invalid-source.jpg](check06-06-invalid-source.jpg) | 1068×768 | 见主报告及独立记录 |
| [check06-07-restored-workbench.jpg](check06-07-restored-workbench.jpg) | 1068×768 | 见主报告及独立记录 |
| [check06-08-conversion-running.jpg](check06-08-conversion-running.jpg) | 1068×768 | 见主报告及独立记录 |
| [check06-09-conversion-paused.jpg](check06-09-conversion-paused.jpg) | 1068×768 | Actual state: pausing request is still pending, not settled pause. |
| [check06-10-conversion-paused-settled.jpg](check06-10-conversion-paused-settled.jpg) | 1068×768 | 见主报告及独立记录 |
| [check06-11-background-stays-settings.jpg](check06-11-background-stays-settings.jpg) | 1068×768 | 见主报告及独立记录 |
| [check06-12-background-completed-settings.jpg](check06-12-background-completed-settings.jpg) | 1068×768 | 见主报告及独立记录 |
| [check06-13-background-result-reader.jpg](check06-13-background-result-reader.jpg) | 1068×768 | 见主报告及独立记录 |
| [check06-14-direct-result.jpg](check06-14-direct-result.jpg) | 1068×768 | Completed note is open; top capture edge includes only the lower part of global navigation. Not full navigation-bounds evidence. |
| [check06-15-tasks-complete.jpg](check06-15-tasks-complete.jpg) | 1068×768 | 见主报告及独立记录 |
| [check06-16-tasks-stages.jpg](check06-16-tasks-stages.jpg) | 1068×768 | 见主报告及独立记录 |
| [check06-17-task-logs.jpg](check06-17-task-logs.jpg) | 1068×768 | 见主报告及独立记录 |
| [check06-18-conversion-failure.jpg](check06-18-conversion-failure.jpg) | 1068×768 | Actual state: summary request still processing, not settled failure. |
| [check06-19-partial-result.jpg](check06-19-partial-result.jpg) | 1068×768 | 见主报告及独立记录 |

## check07

targeted card/progress/title rechecks

| 原图 | 像素尺寸 | 特别说明 |
| --- | --- | --- |
| [check07-01-engine.jpg](check07-01-engine.jpg) | 1068×768 | 见主报告及独立记录 |
| [check07-02-engine-capabilities.jpg](check07-02-engine-capabilities.jpg) | 1068×768 | 见主报告及独立记录 |
| [check07-03-model-start-fixed.jpg](check07-03-model-start-fixed.jpg) | 1068×768 | 见主报告及独立记录 |
| [check07-04-model-progress-fixed.jpg](check07-04-model-progress-fixed.jpg) | 1068×768 | 见主报告及独立记录 |
| [check07-05-narrow-search-title.jpg](check07-05-narrow-search-title.jpg) | 860×620 | 见主报告及独立记录 |

## check08

release performance capture set; per-frame status and later fixes must be read in report

| 原图 | 像素尺寸 | 特别说明 |
| --- | --- | --- |
| [check08-01-partial-result.jpg](check08-01-partial-result.jpg) | 1068×768 | 见主报告及独立记录 |
| [check08-02-failed-stages-separated.jpg](check08-02-failed-stages-separated.jpg) | 1068×768 | 见主报告及独立记录 |
| [check08-03-ai-awaiting-result.jpg](check08-03-ai-awaiting-result.jpg) | 1068×768 | 见主报告及独立记录 |
| [check08-04-pausing.jpg](check08-04-pausing.jpg) | 1068×768 | 见主报告及独立记录 |
| [check08-05-paused-settled.jpg](check08-05-paused-settled.jpg) | 1068×768 | 见主报告及独立记录 |
| [check08-06-recovery-result.jpg](check08-06-recovery-result.jpg) | 1068×768 | 见主报告及独立记录 |
| [check08-07-recovered-reader.jpg](check08-07-recovered-reader.jpg) | 1068×768 | Actual state: reader entry animation intermediate frame, not settled final reader. |
| [check08-08-release-dark.jpg](check08-08-release-dark.jpg) | 1068×768 | 见主报告及独立记录 |
| [check08-09-dark-settled.jpg](check08-09-dark-settled.jpg) | 1068×768 | 见主报告及独立记录 |
| [check08-10-reader-settled.jpg](check08-10-reader-settled.jpg) | 1068×768 | Reader settled capture. The separate generation-information defect is retained in 11/12 and its repaired normal/narrow states are check09-04/05 and check10-35/37. |
| [check08-11-reader-information.jpg](check08-11-reader-information.jpg) | 1068×768 | Historical failed generation-information layout with clipped boundary and weak hierarchy; later normal/narrow rechecks close the reported issue. |
| [check08-12-reader-information-settled.jpg](check08-12-reader-information-settled.jpg) | 1068×768 | Historical settled image of the failed generation-information layout; not a final success merely because motion settled. |
| [check08-13-cancel-request.jpg](check08-13-cancel-request.jpg) | 1068×768 | Historical failed cancellation UI: pending-cancel action wrongly says pausing. Repaired in 6c2e997 and rechecked in check09/11. |
| [check08-14-cancel-settled.jpg](check08-14-cancel-settled.jpg) | 1068×768 | Historical failed cancellation UI: settled cancellation feedback is missing. Later check09/11 provides the repaired ordinary cancellation message. |

## check09

01-08 retain intermediate findings; later e217 rechecks are check11, not a retroactive upgrade of these images

| 原图 | 像素尺寸 | 特别说明 |
| --- | --- | --- |
| [check09-01-restored-cancelled.jpg](check09-01-restored-cancelled.jpg) | 1068×768 | Cancellation information is present, but neutral component-check waiting is incorrectly red; corrected later in d9a9bda source. No native cold-start waiting frame was captured; check11-07 is already ready. |
| [check09-02-cancelling.jpg](check09-02-cancelling.jpg) | 1068×768 | Cancellation action wording is repaired, but an already-seen older terminal-task notification reappears; later notification acknowledgement fix is supported by check11-09 to 11. |
| [check09-03-cancelled.jpg](check09-03-cancelled.jpg) | 1068×768 | Settled cancellation information is present, but an older same-name cancellation notification repeats above it; not final notification acceptance. |
| [check09-04-reader-information.jpg](check09-04-reader-information.jpg) | 1068×768 | Normal-size generation information top; independent reader review confirms complete area and result hierarchy. A global cancellation message is visible above the reader. |
| [check09-05-reader-information-bottom.jpg](check09-05-reader-information-bottom.jpg) | 1068×768 | Normal-size generation information scrolled to its bottom; independent reader review confirms internal scrolling and complete border. The earlier global message is absent; whole-screen vertical displacement is not attributed to information scrolling. |
| [check09-06-library-failure.jpg](check09-06-library-failure.jpg) | 1068×768 | Actual isolated fault: fixture classification JSON was damaged, producing duplicate recovery/diagnostic boxes and exposed raw parse text; corrected later, not final acceptance. |
| [check09-07-reader-file-missing.jpg](check09-07-reader-file-missing.jpg) | 1068×768 | Actual isolated fault: fixture artifact directory was temporarily renamed after listing, making reader open fail. Old prompt is unhelpful raw I/O English; corrected later. Does not prove permanent file loss. |
| [check09-08-reader-restored.jpg](check09-08-reader-restored.jpg) | 1068×768 | Reader can open after fixture files are restored. This is a recovery result, not proof that the earlier error copy was repaired. |

## check10

01-03 are normal 1140x820 at 200%; 04-51 uses e217d0d source in debug to set exact 860x620 at 200%; individual frames require independent review

| 原图 | 像素尺寸 | 特别说明 |
| --- | --- | --- |
| [check10-01-narrow-appearance.jpg](check10-01-narrow-appearance.jpg) | 1068×768 | Actual profile: release normal 1140x820 at 200%, JPEG 1068x768. Not 860x620 narrow evidence. Operator confirmed validation-window override is debug-only. |
| [check10-02-narrow-appearance-resized.jpg](check10-02-narrow-appearance-resized.jpg) | 1068×768 | Actual profile: release normal 1140x820 at 200%, JPEG 1068x768. Attempted edge resize did not alter the window. Not 860x620 narrow evidence. |
| [check10-03-native-resize.jpg](check10-03-native-resize.jpg) | 1068×768 | Actual profile: release normal 1140x820 at 200%, JPEG 1068x768. Attempted native edge resize did not alter the window; this alone is not a production resize-defect conclusion. |
| [check10-04-exact-small-appearance.jpg](check10-04-exact-small-appearance.jpg) | 860×620 | 见主报告及独立记录 |
| [check10-05-small-appearance-controls.jpg](check10-05-small-appearance-controls.jpg) | 860×620 | 见主报告及独立记录 |
| [check10-06-small-generation-top.jpg](check10-06-small-generation-top.jpg) | 860×620 | 见主报告及独立记录 |
| [check10-07-small-generation-middle.jpg](check10-07-small-generation-middle.jpg) | 860×620 | 见主报告及独立记录 |
| [check10-08-small-generation-bottom.jpg](check10-08-small-generation-bottom.jpg) | 860×620 | 见主报告及独立记录 |
| [check10-09-small-services.jpg](check10-09-small-services.jpg) | 860×620 | 见主报告及独立记录 |
| [check10-10-small-service-actions.jpg](check10-10-small-service-actions.jpg) | 860×620 | 见主报告及独立记录 |
| [check10-11-small-service-editor.jpg](check10-11-small-service-editor.jpg) | 860×620 | Actual page: Application/About with the Application tab selected, not the service editor. Do not use it as service form or save-action evidence. |
| [check10-12-small-about.jpg](check10-12-small-about.jpg) | 860×620 | 见主报告及独立记录 |
| [check10-13-small-service-list.jpg](check10-13-small-service-list.jpg) | 860×620 | 见主报告及独立记录 |
| [check10-14-small-service-visible-actions.jpg](check10-14-small-service-visible-actions.jpg) | 860×620 | 见主报告及独立记录 |
| [check10-15-small-ai-card.jpg](check10-15-small-ai-card.jpg) | 860×620 | 见主报告及独立记录 |
| [check10-16-small-editor-top.jpg](check10-16-small-editor-top.jpg) | 860×620 | 见主报告及独立记录 |
| [check10-17-small-editor-auth.jpg](check10-17-small-editor-auth.jpg) | 860×620 | 见主报告及独立记录 |
| [check10-18-small-editor-actions.jpg](check10-18-small-editor-actions.jpg) | 860×620 | 见主报告及独立记录 |
| [check10-19-small-storage-top.jpg](check10-19-small-storage-top.jpg) | 860×620 | 见主报告及独立记录 |
| [check10-20-small-storage-bottom.jpg](check10-20-small-storage-bottom.jpg) | 860×620 | 见主报告及独立记录 |
| [check10-21-small-about-actions.jpg](check10-21-small-about-actions.jpg) | 860×620 | 见主报告及独立记录 |
| [check10-22-small-about-health.jpg](check10-22-small-about-health.jpg) | 860×620 | 见主报告及独立记录 |
| [check10-23-small-guide-engine.jpg](check10-23-small-guide-engine.jpg) | 860×620 | 见主报告及独立记录 |
| [check10-24-small-guide-engine-choices.jpg](check10-24-small-guide-engine-choices.jpg) | 860×620 | 见主报告及独立记录 |
| [check10-25-small-guide-ai-top.jpg](check10-25-small-guide-ai-top.jpg) | 860×620 | 见主报告及独立记录 |
| [check10-26-small-guide-ai-model.jpg](check10-26-small-guide-ai-model.jpg) | 860×620 | Actual visible area: AI-use switches, not a fully visible model field. Shows pre-fix small switch geometry at 200%. |
| [check10-27-small-guide-bilibili.jpg](check10-27-small-guide-bilibili.jpg) | 860×620 | 见主报告及独立记录 |
| [check10-28-small-guide-model.jpg](check10-28-small-guide-model.jpg) | 860×620 | Actual state: model resources are incomplete; not an active download or a fresh paused-worker test. |
| [check10-29-small-guide-model-actions.jpg](check10-29-small-guide-model-actions.jpg) | 860×620 | Incomplete model resources with file-reuse consequence and visible prepare/recheck/details actions; not proof that a new download or verification completed. |
| [check10-30-small-library.jpg](check10-30-small-library.jpg) | 860×620 | 见主报告及独立记录 |
| [check10-31-small-library-row-actions.jpg](check10-31-small-library-row-actions.jpg) | 860×620 | 见主报告及独立记录 |
| [check10-32-small-reader.jpg](check10-32-small-reader.jpg) | 860×620 | Actual page remains library list; does not show a reader or a successful switch to cards. |
| [check10-33-small-library-aligned-actions.jpg](check10-33-small-library-aligned-actions.jpg) | 860×620 | Actual display mode is list, not cards. Supports list auxiliary-action alignment only. |
| [check10-34-small-reader.jpg](check10-34-small-reader.jpg) | 860×620 | 见主报告及独立记录 |
| [check10-35-small-reader-information.jpg](check10-35-small-reader-information.jpg) | 860×620 | 见主报告及独立记录 |
| [check10-36-small-reader-information-bottom.jpg](check10-36-small-reader-information-bottom.jpg) | 860×620 | The capture did not scroll inside generation information; do not use as bottom-of-information evidence. Use 37 for the actual internal scroll. |
| [check10-37-small-reader-information-scrolled.jpg](check10-37-small-reader-information-scrolled.jpg) | 860×620 | 见主报告及独立记录 |
| [check10-38-small-gallery.jpg](check10-38-small-gallery.jpg) | 860×620 | 见主报告及独立记录 |
| [check10-39-small-gallery-caption-actions.jpg](check10-39-small-gallery-caption-actions.jpg) | 860×620 | 见主报告及独立记录 |
| [check10-40-small-gallery-complete-caption-actions.jpg](check10-40-small-gallery-complete-caption-actions.jpg) | 860×620 | 见主报告及独立记录 |
| [check10-41-small-gallery-action-baseline.jpg](check10-41-small-gallery-action-baseline.jpg) | 860×620 | 见主报告及独立记录 |
| [check10-42-small-library-cards.jpg](check10-42-small-library-cards.jpg) | 860×620 | 见主报告及独立记录 |
| [check10-43-small-library-card-footer.jpg](check10-43-small-library-card-footer.jpg) | 860×620 | 见主报告及独立记录 |
| [check10-44-small-library-card-actions.jpg](check10-44-small-library-card-actions.jpg) | 860×620 | 见主报告及独立记录 |
| [check10-45-small-tasks.jpg](check10-45-small-tasks.jpg) | 860×620 | 见主报告及独立记录 |
| [check10-46-small-task-actions.jpg](check10-46-small-task-actions.jpg) | 860×620 | 见主报告及独立记录 |
| [check10-47-small-task-stage-list.jpg](check10-47-small-task-stage-list.jpg) | 860×620 | 见主报告及独立记录 |
| [check10-48-small-task-stages.jpg](check10-48-small-task-stages.jpg) | 860×620 | 见主报告及独立记录 |
| [check10-49-small-task-log-trigger.jpg](check10-49-small-task-log-trigger.jpg) | 860×620 | 见主报告及独立记录 |
| [check10-50-small-task-logs.jpg](check10-50-small-task-logs.jpg) | 860×620 | 见主报告及独立记录 |
| [check10-51-small-task-log-content.jpg](check10-51-small-task-log-content.jpg) | 860×620 | 见主报告及独立记录 |

## check11

normal/fault rechecks; 06 is a newly discovered stale-notice failure, 07 is already ready, not preparation loading

| 原图 | 像素尺寸 | 特别说明 |
| --- | --- | --- |
| [check11-01-library-recovery-start.jpg](check11-01-library-recovery-start.jpg) | 1068×768 | Early view of the isolated classification-record recovery state; later frames show settled recovery and diagnostics. |
| [check11-02-library-recovery.jpg](check11-02-library-recovery.jpg) | 1068×768 | One classification recovery group with actionable recovery choices; isolated fixture JSON fault, not damage to real user data. |
| [check11-03-library-recovery-diagnostics.jpg](check11-03-library-recovery-diagnostics.jpg) | 1068×768 | Raw technical parsing evidence appears only after diagnostic disclosure is expanded inside the recovery group. |
| [check11-04-library-recovered.jpg](check11-04-library-recovered.jpg) | 1068×768 | Library has returned to its ordinary state after recovery/refresh; does not independently prove both recovery choices were each exercised. |
| [check11-05-reader-file-unavailable.jpg](check11-05-reader-file-unavailable.jpg) | 1068×768 | Temporary fixture artifact unavailability produces a localized actionable prompt while keeping the library. Does not assert permanent file loss. |
| [check11-06-reader-retry-restored.jpg](check11-06-reader-retry-restored.jpg) | 1068×768 | Known historical failure: readable note retains old unavailable-file banner. Corrected in 5740837; final release check12-07/08 supplies failure-to-success evidence. |
| [check11-07-startup-preparation.jpg](check11-07-startup-preparation.jpg) | 1068×768 | Actual state: startup is already ready and shows the restored input. No component-preparation/loading state is visible, despite the filename. |
| [check11-08-new-conversion-start.jpg](check11-08-new-conversion-start.jpg) | 1068×768 | 见主报告及独立记录 |
| [check11-09-cancelling.jpg](check11-09-cancelling.jpg) | 1068×768 | 见主报告及独立记录 |
| [check11-10-cancelled.jpg](check11-10-cancelled.jpg) | 1068×768 | 见主报告及独立记录 |
| [check11-11-restart-after-cancellation.jpg](check11-11-restart-after-cancellation.jpg) | 1068×768 | 见主报告及独立记录 |
| [check11-12-conversion-opens-reader.jpg](check11-12-conversion-opens-reader.jpg) | 1068×768 | 见主报告及独立记录 |

## check12

targeted final switch, restored-reader and partial-result rechecks; scope comes from independent image review

| 原图 | 像素尺寸 | 特别说明 |
| --- | --- | --- |
| [check12-01-small-switch-scaled.jpg](check12-01-small-switch-scaled.jpg) | 860×620 | 见主报告及独立记录 |
| [check12-02-small-switch-on.jpg](check12-02-small-switch-on.jpg) | 860×620 | 见主报告及独立记录 |
| [check12-03-small-switch-keyboard-off.jpg](check12-03-small-switch-keyboard-off.jpg) | 860×620 | Native state snapshot reports focused/off after Space; pictured thumb remains near its previous endpoint. Do not call this a settled animation; 05 shows the off thumb settled left. |
| [check12-04-small-guide-switch-scaled.jpg](check12-04-small-guide-switch-scaled.jpg) | 860×620 | 见主报告及独立记录 |
| [check12-05-small-appearance-state.jpg](check12-05-small-appearance-state.jpg) | 860×620 | Settled off switch at exact 860x620 / 200%, with scaled track/knob geometry. Not a complete appearance-page view. |
| [check12-06-final-library.jpg](check12-06-final-library.jpg) | 1068×768 | 见主报告及独立记录 |
| [check12-07-final-reader-unavailable.jpg](check12-07-final-reader-unavailable.jpg) | 1068×768 | Final release isolated fixture read failure after temporary artifact unavailability; localized next steps and original library retained. |
| [check12-08-final-reader-restored-no-error.jpg](check12-08-final-reader-restored-no-error.jpg) | 1068×768 | Final release successful retry after restoring fixture files: readable version 6 and no previous unavailable-file banner. Root reports no manual dismissal. |
| [check12-10-final-partial-running.jpg](check12-10-final-partial-running.jpg) | 1068×768 | Task startup with local 127.0.0.1 auth-error fixture, not cold-start component-preparation evidence. |
| [check12-11-final-partial-result.jpg](check12-11-final-partial-result.jpg) | 1068×768 | Saved readable note with proofreading and summary both unfinished after local 401 responses; only four successful stages belong to completed history. Per-component recovery actions are visible. |
| [check12-12-final-partial-recent-notes.jpg](check12-12-final-partial-recent-notes.jpg) | 1068×768 | Recent version 8 shows partial-processing status once; no redundant current/recovery-chain needs-attention card appears under the current task. |

## installed

actual installed Design Preview About, user library and reader; installation.json supplies identity, independent review supplies pictured composition

| 原图 | 像素尺寸 | 特别说明 |
| --- | --- | --- |
| [installed-01-about.jpg](installed-01-about.jpg) | 1068×768 | 见主报告及独立记录 |
| [installed-02-library.jpg](installed-02-library.jpg) | 1068×768 | 见主报告及独立记录 |
| [installed-03-reader.jpg](installed-03-reader.jpg) | 1068×768 | Actual user note: shared reader axes, normal-weight body and separate table of contents. Summary heading exists in native state text but is not visible in this static frame; not proof of absolute top on first opening. |
| [installed-04-gallery.jpg](installed-04-gallery.jpg) | 1068×768 | Installed real 40-image gallery entry frame; use 05 for the settled gallery. |
| [installed-05-gallery-settled.jpg](installed-05-gallery-settled.jpg) | 1068×768 | Settled gallery of the actual user note with 40 screenshots. Independent reader review confirms three columns and three-line summaries with full aligned actions in the pictured first row; not the exact 200% three-line case. |
| [installed-06-reader-anchor-entry.jpg](installed-06-reader-anchor-entry.jpg) | 1068×768 | Installed reader entry frame while returning to the 00:27 note anchor; use 07 for the settled layout. |
| [installed-07-reader-anchor-settled.jpg](installed-07-reader-anchor-settled.jpg) | 1068×768 | Installed reader settled at the 00:27 anchor after returning from the gallery; does not by itself prove every possible navigation/resize sequence. |

## 配色原图

ten actual palette selections; scoped appearance evidence

| 原图 | 像素尺寸 | 特别说明 |
| --- | --- | --- |
| [palette-01-ink.jpg](palette-01-ink.jpg) | 1330×768 | 见主报告及独立记录 |
| [palette-02-nord.jpg](palette-02-nord.jpg) | 1330×768 | 见主报告及独立记录 |
| [palette-03-tokyo-night.jpg](palette-03-tokyo-night.jpg) | 1330×768 | 见主报告及独立记录 |
| [palette-04-frappe.jpg](palette-04-frappe.jpg) | 1330×768 | 见主报告及独立记录 |
| [palette-05-macchiato.jpg](palette-05-macchiato.jpg) | 1330×768 | 见主报告及独立记录 |
| [palette-06-mocha.jpg](palette-06-mocha.jpg) | 1330×768 | 见主报告及独立记录 |
| [palette-07-paper.jpg](palette-07-paper.jpg) | 1330×768 | 见主报告及独立记录 |
| [palette-08-nord-snow.jpg](palette-08-nord-snow.jpg) | 1330×768 | 见主报告及独立记录 |
| [palette-09-tokyo-day.jpg](palette-09-tokyo-day.jpg) | 1330×768 | 见主报告及独立记录 |
| [palette-10-latte.jpg](palette-10-latte.jpg) | 1330×768 | 见主报告及独立记录 |
