> 归档说明：本文件复制自本轮独立审查记录，只调整图片／日志引用为归档内相对路径。保留逐轮发现与当时的待验证状态；最新结论在末尾更新段。同名最终测试日志按主代理最后一次运行归档，历史段落中的数字保留当时记录；当前结果见主报告测试表。各位审查者未亲自执行的行为仍按原文保留边界。

# 转换流程与任务界面独立审查

审查者：`/root/conversion_flow_review`。日期：2026-09-09。

范围：工作台的空输入、选中文件、来源读取、已有笔记决策、转换过程、完成续接，以及任务的运行、暂停、失败、完成详情。依据为项目 `AGENTS.md`、`.agents/skills/course2md-design/SKILL.md` 及其 system / interaction 参考文档：一次明确开始后，普通内部阶段自动继续；只有影响用户结果的选择才中断；后台完成不抢走用户主动选择的页面；同类状态共用图标槽、标签、状态列及交互原语。

本审查者通过图片工具独立查看了下列静帧，并检查、修改了约定范围内的源码。本审查者没有操作 GUI，也没有运行 Cargo。原生操作及构建由根任务执行。静帧只能证明拍摄时的画面；不能据此认定点击次数、动画流畅度、异步导航边界或失败恢复已实操通过。

## 一、静帧证据与独立判断

截图根目录：本归档目录。`after-*` 属于前一轮新实现；`final-01`、`check03-*` 属于 build03。不能把它们当作含恢复分支修复的 build04 图证。

| 编号 | 已查看图证 | 具体问题与判断 | 修复及当前证据边界 |
| --- | --- | --- | --- |
| F1 | 用户拒绝的运行图 [codex-clipboard-853e00e9-65d8-40e3-9702-128f75ad0ad4.png](codex-clipboard-853e00e9-65d8-40e3-9702-128f75ad0ad4.png) | 标题、状态、来源、阶段和操作挤在左端，进度横跨大卡；阶段图标、名称、数量没有共同阅读轴。问题是信息组织，不只是尺寸或裁切。 | 实现共享任务标题、来源事实和阶段列；新图见 F6。 |
| F2 | 用户拒绝的计划图 [codex-clipboard-4f9b8add-ec3e-4695-a19e-f6d1f83ab556.png](codex-clipboard-4f9b8add-ec3e-4695-a19e-f6d1f83ab556.png) | 用户已经开始转换，准备后还要在计划页再次确认；普通来源和字幕读取成为额外中间步骤。 | 源码删除计划对话框与常规计划页；首次 Start 保存转换意图并自动续接。静帧不能独自证明仅点击一次。 |
| F3 | 用户拒绝的完成图 [codex-clipboard-3988cc1c-dc49-4426-a124-e14bb2877f6a.png](codex-clipboard-3988cc1c-dc49-4426-a124-e14bb2877f6a.png)；[after-17-direct-reader](after-17-direct-reader.jpg) | 旧完成页使已经可读的笔记藏在第二个“阅读”操作后。after-17 已显示真实 reader 与“笔记已生成，已为你打开”提示。 | 源码删除完成中间页。根任务报告真实 Start 后直接进入 reader；本审查者只独立看到了结果静帧，未亲自观察完整点击轨迹。 |
| F4 | 用户拒绝的详情图 [codex-clipboard-2571ffa1-6b46-4aa2-83ac-d7857d865b9b.png](codex-clipboard-2571ffa1-6b46-4aa2-83ac-d7857d865b9b.png)；[after-25-task-details](after-25-task-details.jpg) | 旧页把很多“已完成”文字与英文原始日志叠在一起。after-25 的步骤、状态、实际数量已有清晰列，原始日志也已独立收纳。 | 已完成步骤进入独立详情，日志另设有界滚动区域；静帧证明打开后的结构，未证明其展开动画和滚动操作。 |
| F5 | [after-01-workbench](after-01-workbench.jpg) | 输入、来源标签和标题已有共同左轴，Start 与输入同高。支持信息与高级选项仍散落，多个垂直间隔使关系松散；提示面颜色和背景过近。 | 支持说明合并为一处共享 info_callout，高级选项与其用较小组内间距；根任务统一修正提示面边界。大片剩余空白本身不构成问题，不补无用卡片。 |
| F6 | [after-13-local-input](after-13-local-input.jpg)、[after-14-local-selected](after-14-local-selected.jpg)、[after-15-converting](after-15-converting.jpg) | 空输入时大拖放区域有用途；文件已选中后仍保留大块居中空间没有用途，Start 脱离文件，运行任务被推至约 y416。 | 已选文件改为紧凑文件行，右侧为更换与 Start，空态保留 dropzone；build03 的普通尺寸恢复图 final-01 和窄窗图 check03-13 已显示紧凑文件结构。但恢复态仍有 F10 的重复内容。 |
| F7 | [after-16-conversion-phase](after-16-conversion-phase.jpg) | AI 阶段已有步骤、处理中状态、数量/预计时间与进度条共同阅读轴，优于旧竖堆。已知数量时还同时显示 spinner，形成重复运动信号。 | 源码改为未知量使用 spinner，已知量使用共享平滑 progress；静帧不能证明平滑，也不能替代运行一段时间的观察。 |
| F8 | [after-24-task-complete](after-24-task-complete.jpg)、[after-25-task-details](after-25-task-details.jpg) | 来源/发送/保存三行的值被推到约 800px 之外，标签和值难配对；事实行高和间距比下面阶段表松。 | 根任务将共享 detail_row 改为固定标签列、固定值起点、普通字重和更紧行高；本分支将 facts 改为 gap1，保存位置合入同组。待 build04 任务详情新图验证实际比例。 |
| F9 | [after-25-task-details](after-25-task-details.jpg) | 展开全部完成阶段会把“阅读笔记”或运行中的主要操作推到步骤列表后；完成 AI 阶段再显示 100% 是重复信息。 | 当前进度 → 主要操作 → 已完成步骤 → 原始日志；已完成阶段只保留有意义的截图/转写/模型数量，删除纯百分比。源码已改，尚未以新任务详情图确认。 |
| F10 | [final-01-workbench](final-01-workbench.jpg) | 完成一次后重启恢复输入，顶部文件行/更换/Start，下方又封面/来源标题/更换/已有笔记/生成新版；两套来源身份、两个生成入口。此图不能接受。 | build04 待构建修复：已有笔记时隐藏普通 Start，跳过整套 selected-source 预览，将一句已有笔记提示和打开已有/生成新版直接接在输入下。真实字幕选择仍保留。尚无修复后图证。 |
| F11 | [check03-13-narrow-workbench](check03-13-narrow-workbench.jpg) | 窄窗大字号时，紧凑文件名在第一行，操作自然排第二行，关系清楚；没有退回巨型居中空框。下沿可见的重复 source cover 仍属于 F10。 | 可以确认文件行的这一个换行状态。不能据此认定整张长页、所有任务状态或 build04 恢复态均已通过。 |

用户原始四图已原样存入本归档目录，完整文件名及链接如表所列。另查看过 `check03-11-ai-failure`、`check03-12-ai-running`：它们是 onboarding 的服务检查，**不作为转换任务失败/进度的替代图证**。

## 二、源码检查与行为测试范围

### 自动转换与完成导航

- `import_ui.rs` 删除常规计划/准备确认界面和完成中间页。Start 将输入修订保存在 `pending_conversion`；来源、字幕和环境依次就绪时由 `advance_conversion` 自动继续。
- `ConversionGate` 对正常读取和待就绪状态返回 Wait；换输入或没有有效来源终止旧意图；已有笔记仍保留“打开已有/生成新版”的真实决策。
- `ConversionFollow` 从 Preparing 转移至提交的 Task。完成导航要求仍在工作台、输入修订匹配、任务 ID 与输入及恢复链匹配、状态为 Complete 且已有 artifact。
- `finish_task` 在完成状态写入成功后决定是否打开。用户跳页、改输入或主动打开别的笔记会取消跟随；异步 reader 返回时再次核对来源页、阅读代次和跟随修订，避免延迟结果抢导航。
- 打开完成结果的读取间隙仍显示原任务及“笔记已生成，正在打开”，防止短暂闪回普通输入。全局重复结果提示的抑制由根任务的 views 改动处理。
- 现有输入保存、捕获配置、任务快照、明确字幕选择、取消读取、失败后的重试以及不确定请求重发授权保留；没有修改 core/src 或 CLI 行为。

### 任务信息及状态

- 运行、暂停、等待、失败、不确定、取消和完成阶段共用 `TaskStages`：左侧图标槽及标签、中间状态列、右侧细节。共享 helper 负责字号、图标槽和事实行对齐。
- 完成步骤与原始日志是不同 disclosure。日志保留错误和服务原始证据，使用单独有界滚动区域；没有把它们改成更多竖排产品文案。
- 未知进度使用 spinner，已知进度使用共享 progress。完成打开提示使用共享状态进入动画。上述是源码预期；运动表现需实操或连续帧验证。
- 当前源码还使用 `WorkerWait` 区分“启动”“继续处理”“完成笔记”。中间一个步骤结束不能提前声称整个任务即将完成；这是基于 worker/render 证据的业务状态判断。

### 行为测试证据

本审查者新增/核对的关键测试：

| 测试 | 覆盖的行为边界 | 不覆盖的内容 |
| --- | --- | --- |
| `one_start_continues_metadata_subtitles_and_environment_without_a_confirmation_stage` | 同一个 Start 修订穿过来源读取、字幕及环境等待，最终 Submit；换修订、来源失效、已有笔记决策终止自动提交。 | 真实 UI 点击轨迹、具体网络/引擎时序。 |
| `completion_opens_only_the_result_still_followed_in_the_current_input` | 仅跟随同输入/修订/任务的 Complete+artifact；页面已离开、改输入、其他任务、无 artifact 或 Partial 均不自动打开。 | 原生事件循环中的竞争时序和 reader 实际加载动画。 |
| 既有 `completion_belongs_to_the_submitted_input_and_requires_a_readable_result`、恢复链相关测试 | 当前输入的结果归属、Queued/Running/Complete 恢复链、缺失/环/错误 parent/不同来源/重复 ID/新输入拒绝。 | 视觉布局。 |
| 既有自动字幕 fallback、明确字幕及取消读取测试 | 自动模式的可恢复失败可以识别声音；明确字幕模式或用户取消不能自动改变选择；缓存字幕不能掩盖仍待确认的新选择。 | 真实字幕服务返回。 |
| 当前源码 `a_closed_intermediate_stage_does_not_claim_the_task_is_finishing`、`finishing_uses_worker_evidence_and_stops_with_the_process` | 中间步骤完成只进入继续处理；render/worker 结果才可进入完成中；已有结果复用、仍有活动阶段、进程停止的边界。 | spinner、动画节奏及最终画面。 |

已读取根任务执行的 [tests-desktop.log](tests-desktop.log)：关键两项转换/导航测试及两项 worker 等待测试均为 `ok`；整体记录为 **191 passed / 0 failed / 3 ignored**。另 [tests-task-execution.log](tests-task-execution.log) 记录 **11 passed / 0 failed**。这是日志证据，不是本审查者运行 Cargo，也不是 GUI 验收结果。

本分支已执行 rustfmt 与 `git diff --check`。根任务已报告 build03 成功；F10 的最新恢复分支在其之后修改，等待 build04 的构建/画面证据。

## 三、首次记录时待原生实操与新截图

以下项目仍待根任务提供 build04 图证/操作记录，不能标成已通过：

1. **恢复已有笔记输入**：普通和窄窗大字号；只有当前输入的一份来源身份，无 source cover/重复更换/普通 Start；提示与打开已有/生成新版紧邻；两个操作分别可用。
2. **一键正常转换**：本地与链接输入各从 Start 到运行，不出现常规计划/准备确认页；默认字幕读取/自动 fallback/环境等待完成后不要求再点开始。
3. **未知进度及已知进度**：启动和阶段间等待有真实状态，已知数量的 bar 平滑前进；不同时用重复的无限 spinner；状态切换动画不因每帧进度重置。
4. **暂停/继续**：保留原输入和配置，暂停状态共用阶段列，主要继续操作在完成历史之前；继续后仍是原任务/其授权恢复链。
5. **失败与恢复**：失败摘要可理解，有明确恢复动作；来源/状态/细节列不散；原始日志独立，展开不推走主要操作。服务拒绝、确定失败和不确定请求要区分；不确定重发仍需明确授权。
6. **完成**：仍跟随任务时直达 reader，出现明确成功提示及短过渡，没有完成中间页；完成读取间隙没有普通输入闪烁，也没有重复全局结果提示。
7. **完成导航边界**：转换中手动去设置/笔记/其他任务，或更换输入，旧任务完成不得抢导航。reader 正在异步打开期间再导航也不得被延迟结果拉回。
8. **任务完成详情**：普通尺寸和窄窗大字号下，来源/发送/保存值起点固定，阶段状态与数量可配对；Read/Pause 等主操作位于完成历史之前；无无意义 100% 文字。
9. **日志与详情实操**：点击展开、收起与长日志滚动；布局自然扩展，控件可达。单张展开静帧不能证明这些交互已工作。

首次记录结论：已找到并修复具体的流程和信息结构问题，当时已有静帧证明阶段表和文件行方向有效；恢复已有笔记、暂停/失败、完成导航边界及运动表现仍需要后续原生证据。以下 check06 复核更新这一验证状态。不能用“没有裁切”或单一 reader 截图替代全流程验收。

## 四、check06 原生截图复核与进度语义修复

本节是首次记录后的更新。以下截图由根任务实际操作 native app 后生成，本审查者已逐张独立查看。图片尚不包含本节新增的 AI 进度/暂停/失败分类修复；这些源码已冻结，等待根任务统一构建与测试。

| 编号 | 图证 | 独立复核结果 | 状态与动作 |
| --- | --- | --- | --- |
| F12 | [check06-03-empty-workbench](check06-03-empty-workbench.jpg)、[check06-05-empty-tasks](check06-05-empty-tasks.jpg)、[check06-06-invalid-source](check06-06-invalid-source.jpg) | 空工作台是输入→Start→默认说明→高级选项的单一阅读顺序；空任务页有明确含义和进入输入的操作。无效链接的错误留在输入下面，焦点仍在输入，未切换到另一错误页。 | 这些静态状态的信息层级清楚，不需要额外摘要卡或填满空白。没有用它们代替异步处理状态。 |
| F13 | [check06-07-restored-workbench](check06-07-restored-workbench.jpg) | 恢复已有笔记后，仅有当前文件行和一次更换视频；下面紧接已有笔记说明及打开已有/生成新版，普通 Start 和重复 source cover/标题已消失。 | F10 的具体重复问题已有修复后静帧证据。根任务后续实际点击生成新版并得到 reader，见 F16。 |
| F14 | [check06-08-conversion-running](check06-08-conversion-running.jpg) | 来源事实已固定在相近的值列，当前阶段关系明确；但 AI 校对显示“100% · 收尾中”和满进度条，而根任务的 slow fixture 请求还需等待约 12 秒。该进度陈述不真实。 | 查阅 `src/llm.rs` 发现 `pb.inc(1)` 在 `polish_chunk(...)` 前，表示派发批次，并非收到结果。本轮仅在 task_ui 的展示投影修正：AI 未完成时没有百分比、分母或 ETA，显示 spinner 和“等待服务返回结果”；不改 worker/CLI 事件。待新图。 |
| F15 | [check06-09-conversion-paused](check06-09-conversion-paused.jpg)、[check06-10-conversion-paused-settled](check06-10-conversion-paused-settled.jpg) | 09 实际是暂停中，顶部称保存进度，阶段区却称“正在继续处理”。10 暂停完成后把正常用户操作画成红色双语 worker 错误，顶部又重复同一任务暂停通知，整体内容随之下移。 | 本轮按 Intent 区分暂停/取消等待，活动阶段显示暂停中/停止中；暂停完成用普通信息“转换已暂停，进度已保留”，保留继续按钮，原始双语在日志。根任务已在 views 过滤工作台正在显示的同一任务通知；后台通知保留。待新图。 |
| F16 | [check06-11-background-stays-settings](check06-11-background-stays-settings.jpg)、[check06-12-background-completed-settings](check06-12-background-completed-settings.jpg)、[check06-13-background-result-reader](check06-13-background-result-reader.jpg)、[check06-14-direct-result](check06-14-direct-result.jpg) | 11 处理中留在设置；12 已完成后仍留在相同设置画面，仅通知内容变化；13 的 reader 有返回设置；14 前台完成的 reader 有返回工作台和“笔记已生成，已为你打开”的成功提示。 | 结合根任务的实际操作记录：离开工作台→后台完成不抢导航→点击通知才进入结果，以及前台一次生成新版→直接 reader，均有原生操作与相应静帧证据。未亲自操作 GUI；未将这些截图扩大为“改输入/阅读异步竞争”等所有边界都已实操。 |
| F17 | [check06-15-tasks-complete](check06-15-tasks-complete.jpg)、[check06-16-tasks-stages](check06-16-tasks-stages.jpg)、[check06-17-task-logs](check06-17-task-logs.jpg) | 保存/文字来源/AI 处理的值起点固定且靠近标签；“阅读笔记”等操作处于完成阶段之前；阶段表中的图标、名称、已完成状态、实际截图数量可以配对；日志在独立面中。17 已滚动到下部，顶部随正常页面滚动离开，不判作布局裁切。 | F4/F8/F9 的主要结构修复已有新静帧证据。任务标题下的时间继承了按钮字重，和标题争层级；本轮将时间显式恢复普通字重，待新图。 |
| F18 | [check06-18-conversion-failure](check06-18-conversion-failure.jpg)、[check06-19-partial-result](check06-19-partial-result.jpg) | 18 文件名虽写 failure，画面仍是请求中的生成摘要，不能当失败完成证据。19 才是实际部分结果：有简短中文原因及仅补校对/仅补摘要/阅读操作；但说明称校对未完成，完成历史计数仍包含校对，状态自相矛盾。 | 查阅 pipeline 确认 `llm stage done` 也在失败尝试结束后发送。本轮用已保存 component outcome 校正 AI 阶段：failed/partial 留在未完成区，succeeded 才归完成历史。活动重试只读取当前 pending_done，避免旧结果污染状态。待新图。 |

### 本轮新增源码与针对性测试

- `task_stage_progress` 不把 AI 派发批次当作已处理数量。截图保存等有真实完成数量的阶段仍可显示有分母的平滑进度。
- `ai_stage_outcome` 使用 `proofreading` / `summary` component 结果，避免“尝试结束”被标为成功步骤。无原始结果时不凭计数推断成功。
- `WorkerWait` 检查暂停、退出、取消意图；已暂停的反馈走普通信息语义。失败反馈保留短中文摘要，整段英文或长技术文本收在原始日志中。
- 工作台与任务页复用相同反馈，主动暂停不再称“调整后重试”。通知去重由根任务的 views 修改完成。
- 类型检查过程中根任务发现 `Div` 与 `Stateful<Div>` 分支返回类型冲突，已统一为 `Option<AnyElement>`；尚待修改后的最终构建结果。

新增测试范围：

| 测试 | 行为断言 |
| --- | --- |
| `ai_dispatch_counts_do_not_claim_received_results` | AI 派发 1/1、3/3 等样本不能产生百分比、满进度条或收尾陈述；完成事件才结束处理状态。真实截图数量仍保留 2/6。 |
| `completed_ai_attempts_use_component_results_to_classify_success` | 校对 failed/partial 不能进入完成历史；成功校对和摘要可归已完成；没有结果或其他阶段不臆造 AI 状态。 |
| `pause_and_cancel_intents_do_not_report_continued_generation` | 启动、阶段间、最终保存等等待边界均服从暂停/退出/取消意图，进程停止后不再显示等待。 |
| `user_pause_is_information_and_worker_failures_have_a_localized_summary` | 主动暂停是普通信息；双语原因只显示短中文部分，原始技术英文不进入主错误段。 |

这四项是本轮新增测试源码，**不得沿用前面的 191 passed 结果声称它们已通过**。根任务正在统一重跑测试/构建。本审查者仅执行 rustfmt 和 diff 检查。

### 最新剩余验证

待构建后的 AI 请求处理中、暂停中、暂停完成、部分失败及完成历史新图，重点核对百分比消失、真实措辞、普通暂停提示、同任务通知去重、失败校对未进完成历史。新增测试也待根任务结果。此前尚未实操的链接输入全流程、用户更换输入与 reader 加载竞争边界，以及连续动画表现仍不能声明已验证。

## 五、check08 对修复的最终静帧复核

本审查者已独立查看如下新图，以下结论替换上一节相应的“待新图”状态。

| 图证 | 新画面及判断 |
| --- | --- |
| [check08-01-partial-result](check08-01-partial-result.jpg)、[check08-02-failed-stages-separated](check08-02-failed-stages-separated.jpg) | 校对和摘要分别留在“未完成”行；展开的完成历史只有读取字幕、扫描画面、生成截图、生成笔记四项。失败摘要、状态列、仅补相应组件的操作彼此一致，原先失败校对算成功的矛盾已消失。顶部不再重复当前任务通知。 |
| [check08-03-ai-awaiting-result](check08-03-ai-awaiting-result.jpg) | AI 校对显示“处理中 / 等待服务返回结果”，没有百分比、分母、ETA 或满进度条。图标槽、名称、状态和说明保持共同列。该帧证明等待语义正确；不据此声称观察了完整动画周期。 |
| [check08-04-pausing](check08-04-pausing.jpg) | 顶部是正在保存进度，当前请求行是“暂停中 / 等待服务返回结果”，按钮为正在暂停，支持说明为暂停后可继续。已不再宣称正在继续生成。 |
| [check08-05-paused-settled](check08-05-paused-settled.jpg) | 主任务内为普通信息“转换已暂停，进度已保留”，继续生成是主要操作，调整选项没有再称失败重试。原先红色双语暂停错误已从主任务移走；顶部通知也不再重复。但最近笔记区仍重复同一恢复链任务并露出双语暂停，见下方剩余重复项。 |
| [check08-06-recovery-result](check08-06-recovery-result.jpg) | 仅补校对成功后，只保留摘要未完成说明、仅补摘要和阅读笔记操作；没有重复补成功校对。与根任务报告的实际恢复范围一致。最近笔记区仍重复当前恢复任务。 |

最新独立检查到的两项列表重复并非 import_ui 本体拼接，定位已交由拥有 `course_library.rs` 的 reader 审查者处理：

1. `Course::description` 已带“部分处理未完成”，`course_meta` 再追加一次；最近笔记拼接两者导致 check08-03/04 中同一句重复。只需去重文案，不改变任务状态。
2. `recent_notes_section` 以 `draft.submitted_task` 的原始 ID 排除当前任务，恢复链的新 ID 因而又进入最近待处理列表。改用 `current_input_task(cx)` 解析后的当前 ID，并使用共享 `task_ui::task_attention_summary(&TaskRecord)`，让其他待处理卡也保持短中文摘要。共享函数已提供；本审查者未交叉编辑 course_library。

已重新读取根任务的 [tests-desktop-final.log](tests-desktop-final.log)：本轮四项 AI 进度/结果、暂停意图及反馈语义测试均为 `ok`，整体 **195 passed / 0 failed / 3 ignored**。日志中的测试结果不替代 GUI，也不替代后来最近笔记去重改动的最终构建。

现有图证已支持本轮转换/任务主要结构和状态语义修复。列表重复由相应文件负责人收尾，等待修复后的图或构建确认。保留前述未亲自实操/未观察连续动画的边界，不扩大验收范围。

## 六、提交后的行为反查与边界修复

对 flow 提交 `59ae6c9` 只读反查时，额外发现并交由根任务确认了以下实质边界；根任务随后授权局部修复。

1. **冷启动已有笔记的扫描竞态**：输入/来源和 Complete 任务先恢复，而 `courses` 初始为空、课程库扫描异步返回。快速 Start 在扫描前可能判定没有已有笔记，且任务去重只排除未结束任务，因而直接建立新版。已在 `conversion_gate` 增加库扫描就绪条件，`self.loading` 时保留第一 Start 并 Wait；扫描完成既有回调继续判断已有笔记。显式生成新版仍直接提交并跟随新任务，不会自动打开旧结果。新增测试 `first_start_waits_for_library_scan_before_existing_note_decision` 与 `explicit_new_version_follows_new_task_instead_of_previous_result`。本修复尚待根任务修改后的测试/原生验证结果。
2. **取消误称暂停且取消落定无反馈**：[check08-13-cancel-request](check08-13-cancel-request.jpg) 显示当前阶段已停止中，按钮却仍写正在暂停；[check08-14-cancel-settled](check08-14-cancel-settled.jpg) 回到原输入与已有笔记决策，主块被移除、顶部又因当前任务去重被抑制，没有明确取消状态。本审查者已看图，根任务实际 native 取消也确认。局部修复：Intent::Cancel 的停止阶段只保留禁用的正在取消操作/loading 图标；取消落定在当前输入下保留一条普通信息，不保留整张任务卡。扩展已有测试，验证 Cancelled 是普通信息，取消恢复链仍属于原输入用于反馈、不会选旧结果自动打开，换输入后不再匹配。待新图和测试结果。

手动导航的只读反查未发现绕过点：导航、换源和手动打开笔记分别使 follow 或阅读代次失效，异步回调还检查原页面及来源修订。不过“大笔记正打开时立即跳页/换输入”的原生竞争时序仍不能由源码检查替代。

## 七、check09 冷启动与取消复核

- [check09-01-restored-cancelled](check09-01-restored-cancelled.jpg) 已显示普通取消信息，输入身份只保留一份。但实测冷启动时“正在检查生成笔记需要的组件，请稍候”是红色错误文本：正常异步检测被 `submission_issue` 的统一错误呈现误分类。新增局部修复只处理这一个精确等待状态，使用 `Role::Status`、原说明字号、普通灰色文字及共享 spinner；实际缺组件、偏好文件等阻塞错误仍保留 Alert/DANGER。不是根据图片无裁切而通过，已记录为新的状态语义缺陷。
- [check09-02-cancelling](check09-02-cancelling.jpg) 的停止中与正在取消动作已一致；[check09-03-cancelled](check09-03-cancelled.jpg) 落定后原输入下有普通取消信息，取消操作本身的两个缺陷已有新静帧支持。
- 09-02/03 又出现上一条同名取消任务的延迟顶部通知：该旧终态先在工作台被看到，显式生成新版替换输入任务关联后才重新冒出。新增最小修复放在 `enqueue_current` 已有的成功入队事务内，在替换 `submitted_task` 前仅确认当前 PageNew 输入关联的 finished 任务为已读。其他后台任务不受影响，失败提交不误确认，也不在 render 写盘。待最后构建/原生图证确认这一通知不再延迟出现。
- 最新 [tests-desktop-final.log](tests-desktop-final.log) 已记录库扫描和显式新版两项新增测试通过，整体 **197 passed / 0 failed / 3 ignored**。本节之后的普通等待呈现和终态通知确认是较晚的局部修改，不把这份旧日志扩大为修改后全部 GUI 验证。

## 八、e217 release 最终连续图复核

根任务提供 release 的 check11-07～12 后，本审查者已逐张独立查看。未修改源码、未运行 Cargo、未操作 GUI。本节为转换/任务流程的最新独立结论。

| 图证 | 可以确认的画面与流程意义 |
| --- | --- |
| [check11-07-startup-preparation](check11-07-startup-preparation.jpg) | 实际已就绪的恢复输入：仅一份文件输入，普通取消信息和已有笔记决策清楚，打开已有/生成新版紧邻，最近笔记数量、版本和日期没有重复串。文件名含 preparation，但画面没有组件准备状态；**不作为 loading 视觉图证**。 |
| [check11-08-new-conversion-start](check11-08-new-conversion-start.jpg) | 生成新版后直接出现当前任务和 AI 校对等待结果；没有假百分比、满进度条或普通计划确认步骤。来源、AI 发送对象、当前阶段与操作有稳定阅读顺序。 |
| [check11-09-cancelling](check11-09-cancelling.jpg) | 当前阶段为停止中，仅有正在取消的 loading 操作，没有正在暂停误文案，也没有同名旧取消任务的顶部通知。 |
| [check11-10-cancelled](check11-10-cancelled.jpg) | 取消落定回到原输入，普通取消信息仍在；旧可读结果和打开/生成新版决策保留，没有失败红字或额外完整任务卡。任务导航此时有当前未读计数 1。 |
| [check11-11-restart-after-cancellation](check11-11-restart-after-cancellation.jpg) | 再次开始后只有新任务进度，上一取消信息不再出现，导航原未读计数 1 已去除，也没有延迟弹出的旧取消通知。为成功入队时定向确认旧终态提供了修复后的原生静帧证据。 |
| [check11-12-conversion-opens-reader](check11-12-conversion-opens-reader.jpg) | 结果进入版本 7 的真实 reader，有“笔记已生成，已为你打开”成功提示和返回工作台。结合根任务记录的一次开始到结果操作，完成没有再经过完成中间页，也没有打开此前保留的版本 6。 |

最新结论：这组六张原生图与根任务的连续操作记录支持从恢复输入、显式生成新版、运行、取消、再次开始到直接阅读的修复已闭环；此前复现的假 100%、取消误称暂停、取消无反馈和旧终态延迟通知，在对应最终画面中未再出现。版本 7 的结果也与明确的新版本意图一致。判断依据是状态与信息关系一致，不是只判断无裁切。

仍保留的验证边界：

- 组件准备的中性 Status/普通字重/GRAY/spinner 已在源码实现，但本次冷启动过快，07 没拍到该状态。记录为源码已核对、原生静帧未捕获，不人为拖慢应用制造截图。
- 阅读加载期间立即跳页/换输入的极短竞争窗口仍以修订、阅读代次和页面保护的源码及行为测试为据，没有额外原生重放记录；此前后台完成停留设置的原生流程已有 check06-11～13 支持。
- 本审查者只看到了连续流程的若干静帧，没有独立量测每帧动画性能或完整 spinner 周期。不要将上述结论扩展成所有动画或所有网络/磁盘时序都已经原生测试。
- reader 的文件暂不可用后重试成功残留提示由 reader 审查者收尾，未改动本审查者的转换流程区域，不计为此处已完成的独立验证。

## 九、200% 小窗任务页补充复核

本审查者已独立查看根任务提供的以下四张原生截图；本节只补充图证与判断，源码继续冻结。

| 图证 | 具体观察与判断 |
| --- | --- |
| [check10-45-small-tasks](check10-45-small-tasks.jpg) | 两行任务标题和普通字重时间共用左轴，状态徽标仍从属于任务标题。三条元信息的图标、标签和值各有固定起点，值没有被推到窗体最右侧；AI 发送对象仍能读成完整信息。标题、时间和事实的层级清楚。 |
| [check10-46-small-task-actions](check10-46-small-task-actions.jpg) | 调整并生成新版与阅读笔记同高、同基线；阅读笔记保留主要操作的强调，调整操作次之。处理详情与原始日志在操作之后，用户不必先读完历史才能打开结果。 |
| [check10-48-small-task-stages](check10-48-small-task-stages.jpg) | 六个完成阶段共用图标槽、标题起点和已完成状态列。扫描画面与生成截图的实际数量共用右侧说明列，只有存在实际数量的行显示数量；没有为凑齐表格而重复百分比或无意义说明。扫描数量、截图数量与对应阶段能直接配对，200% 下仍保持三个信息层次。 |
| [check10-51-small-task-log-content](check10-51-small-task-log-content.jpg) | 展开的原始日志有独立收起入口和明确边界的面板，且与完成阶段留出组间距离。当前可见英文诊断留在该面板内部，没有回流成任务主摘要或失败说明，日志与用户状态的层级清楚。 |

结合这四张图，完成任务的小窗元信息、阶段状态和数量列、主次操作及诊断日志分组没有发现新的阻断问题，不需要继续改动转换/任务源码。页面在截图间向下滚动，顶部阶段或开关部分离开视口是正常滚动上下文，未作为新的版式问题。

这些图证确认的是所示完成状态的可读性与结构。截图中的短日志面板不能单独证明任意长日志滚动行为；也不据此宣称已测量动画或补齐所有 200% 运行、暂停和失败状态。第八节仍保留的原生时序与准备中未捕获边界不变。

## 十、9ec2f6e release 部分失败与最近笔记去重闭环

本审查者已独立查看根任务最后提供的三张 release 原生截图。根任务报告此轮使用本地 auth-error fixture，让校对和摘要都返回 401；图中发送对象为 `127.0.0.1`。本审查者只读截图并更新本记录，没有调用真实服务、访问凭据、操作 GUI 或修改源码。

| 图证 | 具体观察与判断 |
| --- | --- |
| [check12-10-final-partial-running](check12-10-final-partial-running.jpg) | 本地来源直接进入当前任务启动状态，有 spinner、暂停与取消操作，AI 发送对象明确为本地地址；最近笔记仍是既有可读结果。该帧是任务启动，不将其算作此前组件准备等待状态的图证。 |
| [check12-11-final-partial-result](check12-11-final-partial-result.jpg) | 顶部状态说明笔记已保存但部分处理未完成；正文已保存的说明与校对、摘要两个未完成行分开。两项失败共用图标、标签和状态列，分别提供仅补校对、仅补摘要，并保留阅读笔记。完成历史计数为四项，没有把两个失败的 AI 步骤纳入成功历史，也没有将局部失败误写为整个笔记不可读。原始日志仍独立收纳。 |
| [check12-12-final-partial-recent-notes](check12-12-final-partial-recent-notes.jpg) | 下滚后可见当前部分结果对应版本 8 的最近笔记，元信息按笔记数、截图数、部分处理未完成、版本和日期排列，其中“部分处理未完成”仅出现一次。当前任务主块下面没有再插入同任务或恢复链的额外“需要处理”卡；最近项仍有阅读笔记操作。 |

第五节留下的两项最近笔记验证缺口现已关闭：元信息重复文案和当前输入恢复任务重复卡均有修复后的最终原生画面支持。本轮两个 AI 失败的状态、可读正文及按组件恢复操作也保持一致，没有新的必须修改项。

这是根任务所述本地 401 情境及截图所示状态的独立视觉复核，不扩展为所有服务错误类型的测试。第八、九节关于组件准备画面未捕获、极短导航竞争、连续动画和任意长日志的验证边界仍保留；其中“最近笔记去重待新图”已由本节替换。
