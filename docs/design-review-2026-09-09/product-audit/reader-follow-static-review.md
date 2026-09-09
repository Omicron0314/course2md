# 阅读页补做自动跟随：独立静态复核

范围：`desktop/src/import_ui.rs` 的 `ConversionFollow::ReaderTask` 与相邻测试；`task_ui.rs` 的补做提交、恢复、完成；`main.rs` 的异步读入守卫和版本位置交接，并读取相关 reader 保存/查看器入口以确认集成后果。未修改产品代码、运行 cargo、操作 GUI 或提交。以下是修复前初审；后续补丁复核另行追加，不用原生构建成功代替判断。

最新状态：v8 常规同页与离开设置路径已有原生证据；v9 已独立核对“补做运行时打开查看器 → worker 完成仍保留旧查看器 → 返回旧正文 → 显式打开新版”四张原图、AX 和实际任务产物，**R2 的这条主路径已原生闭环**。R1 的慢文件读入期间继续滚动，以及 R2 的读入期间打开又关闭查看器，已补必要源码守卫并有边界单测，但本次没有人为注入慢磁盘，不能写成原生触发通过。修前观察保留如下，最终证据和限制见末节。

## 初审发现

### R1 · P2 · 新版异步读入期间继续阅读，会恢复到读入开始时的位置

路径：`main.rs::open_course_with_conversion_guard` → `reader_ui.rs::save_reading_position` → 回调里的 `state.positions` 复制与 `restore_reading_position`。

自动读入开始前会保存旧版位置，之后 `reading = true`。此时旧 preview 仍在页面里显示，滚动/目录操作仍可发生；但 `save_reading_position` 对 `reading` 直接返回。异步读完后，现有实现仅把工作区里已经保存的旧版位置复制给新版，未在替换 preview 前重取当前可见位置。因此在慢存储/较大笔记上，用户等待时继续向下读，切新版会回退到读入开始时的锚点。

修法范围：回调的代次、页面、follow 身份和原版本路径守卫全部通过之后、替换 preview 之前，以当前旧阅读视图的位置更新迁移来源。保留原版位置与两个 tab 的各自状态，不为了回避竞态冻结正文阅读。

所需复验：延迟新版读入，开始后在旧文滚到另一段/章节，再等结果；新版应落在最近阅读位置的相应锚点，不能回到刚开始读入的段落。位置不能在已离开页面或已改打开对象时被晚回调写入。

### R2 · P2 · 查看器持有旧版本时，自动换版会使“正文”动作失效

路径：`task_ui.rs::finish_task` 的 reader completion 判定 → `main.rs` 替换 preview → `reader_ui.rs::ImageViewer` 与 `image-to-body`。

截图查看器保存了打开时的 `version` 和 frames，打开它不会改变 Page::Result 或 read_generation。当前自动完成判定没有检查查看器，所以补做完成时可以在弹层后方换成新版。`ensure_reader_data` 不会替换已有查看器；其“在笔记中查看”动作要求 `viewer.version == preview.course.dir` 才寻找锚点，换版后条件必然失败，动作只会关闭弹层，不能跳到截图对应正文。

同一问题还有异步间隙：完成时查看器未开，但新版读取期间用户打开截图，同样可能在晚回调时替换其背后的笔记。只在 task 完成瞬间检查一次不够；若只在回调看当前 viewer，还会漏掉“等待期间打开又关闭”的明确阅读选择。

修法范围：查看器开启时停止 ReaderTask 自动跟随，保持它和旧 preview 属于同一版本，让已生成的新结果通过现有入口手动打开；完成判定与晚回调也检查该边界。不得通过自动关闭用户正在看的弹层解决。

所需复验：补做运行时打开截图，以及新版异步加载期间打开截图，各自等待完成；查看器图片/文字/原视频和“正文”继续针对旧版工作，不被强行换版。再补一条等待期间打开后关闭查看器，确认晚结果不会随后抢位置。

两项已经交给主代理和 Rawls，约定最小修复：回调守卫通过后保存最新旧位置；查看器打开时清除 ReaderTask，完成和读入回调排除查看器。本人不改代码。

## 其余检查成立的范围

| 目标 | 静态证据 | 结论 |
| --- | --- | --- |
| 从原笔记发起补做 | `reader_reprocess` 仅接受 Reprocess、非空 components、base_version_dir 等于当前 preview.dir，且排除 exports-only；记录 task id、完整版本路径和 reader revision。普通补做和修复服务补做都在新任务建立后调用 `follow_resumed_input_task`。 | 跟随绑定到了实际新任务和当前原版本，不是只按课程标题。 |
| 同页完成打开新版 | `finish_task` 要求当前没有另一份读取、跟随任务 id 相符、Page::Result、read_generation 相符、仍为原路径；只有 Complete/Partial 且 artifact 确实不同于旧版才打开。 | 普通常规同页路径成立，R1/R2 是其相邻交互漏洞。 |
| 离开再返回 | `navigate` 清除 ReaderTask 并增加 read_generation；返回同一页不恢复旧的内存跟随意图。手动 `open_course` 也先清跟随；手选版本增加 read_generation。 | 没有看到离开又返回后被旧意图重新接管的路径。 |
| 切换笔记与晚结果 | 异步回调先核对当次 read_generation 和 origin page，再核对完整 follow 对象与原 preview 路径；失败后不覆盖另一个新读入。 | 正常切换对象的迟到读入被拒绝。回调里的 `reader_revision` 是启动时捕获值，外层已经核对新 read_generation；这种配合是有意的，不应误判为漏用当前代次。 |
| Partial | 只要任务是 Partial 且发布了不同的新 artifact，就可进入新版；Uncertain、Paused、Running 和没有新 artifact 都不触发。 | 可读 Partial 不被 Complete-only 门槛挡住，未确认请求仍不会自动当成功。 |
| exports-only | 注册时由 `reader_reprocess` 排除，完成处再次 `!task.exports_only()`。测试还覆盖伪造不同 artifact 不能把 exports-only 变成导航。 | 仅补导出不会换阅读版本。 |
| 原位置映射 | main 仅在同 course_id 且不同 version_id 时复制两个 tab 的旧位置，新版已有位置不覆盖；正文/图库优先同 anchor，缺失时用时间定位，最后才落回像素 offset。 | 保留了跨版本的位置恢复机制；具体行/图视觉位置还需原生验证，不能把纯数据复制当精准排版通过。 |

## 测试覆盖与证据边界

已读新增的 `reader_repair_opens_the_new_version_only_while_the_original_note_is_still_open` 和 `reader_repair_guard_excludes_exports_and_rechecks_the_note_when_loading_finishes`。它们覆盖同页 Complete/Partial、不同页面、不同代次、不同版本、不同任务、未完成状态和 exports-only 的判定。测试属于有价值的状态约束，但没有经过真实 Desktop 异步换 preview/滚动保存/查看器动作的组合，因此没有覆盖 R1/R2。

本轮没有执行这些测试，构建/测试结果由主代理记录。待补丁后只复核 R1/R2 的实际接线与必要测试；随后对照原生同页补做、离开/切换和查看器证据结案。不扩展未证实的新缺陷。

## v8 常规原生路径追加

已实看四张原图及 AX，并读取 `v8-reader-follow-task-evidence.json`，再只读核对当前 wide 工作区中对应任务、parent、operation 和产物 manifest。

- **同一阅读页直接进入新版：这条常规路径有原生证据。** `v8-reader-partial-before-repair` 是 `BV1UXPRODUCTFOLLOW` 的可读第 1 版，明确正文已保存、AI 校对/摘要未完成并提供修复入口。主代理操作记录是点击修复并保存补做，没有再点“查看新版”。`v8-reader-repair-auto-opened` 已显示版本 2，新增真实摘要和命名主题，原有失败块退出，正文截图仍可读。对应补做任务 `task-18d3a61bf2bbda00-dbfe-2` 的 parent 指向原 partial，operation 仅 proofreading/summary，manifest revision 2 / partial false，两项处理均 succeeded。图片本身证明最终内容和版本；无第二次打开操作由主代理的原生操作记录支持。
- **离开阅读后保持设置：这条常规路径有原生证据。** `v8-repair-left-reader` 显示设置外观页及任务计数 1；`v8-repair-background-completed-settings` 仍是同一设置分类、主题与 125% 字号，任务计数已退下。`BV1UXPRODUCTFOLLOWAWAY` 的补做 `task-18d3a62cebbbdfc8-dbfe-5` 已 complete、revision 2，原 task 是其 parent。任务 created 到 updated 相差 25 秒，与两项慢服务处理后的完成相符，因此不是“工作还没结束所以暂未跳页”的假通过。主代理描述单次服务约 12 秒，本报告不把整份补做误写为只持续 12 秒。

这四张图没有显示等待读入时继续滚动、查看器开启，或离开后又返回的间隙。因此 **R1/R2 不由 v8 关闭**，仍等待补丁和 v9；常规同页及离开留在设置这两条成功证据可以独立保留。

## R1 / R2 补丁静态复核追加

已直接重新读取 Rawls 的四处集成接线与两项新增测试，没有仅依据其消息认定修复。

- **R1 的必要次序已落实。** `open_course_with_conversion_guard` 回调先验证 read_generation/page，恢复 `reading = false`，再验证 active follow 与原版本上下文。仅在全部通过的 Ok 分支、preview 仍是旧版时调用 `save_reading_position`，然后调用 `carry_repaired_reading_positions` 迁移、替换 preview 并恢复。这样慢读期间的已呈现滚动位置能被重新捕获，不再仅依启动前快照。迁移 helper 保持原键、保留目标已有位置，并拒绝跨课程或同版本复制。新增 handoff 测试覆盖这几个数据语义；慢读竞态和真实滚动排版仍不能由该数据测试代替。
- **R2 覆盖了完成帧和异步间隙。** `open_reader_image` 在确认帧有效、确实会建立 viewer 后立即清除 ReaderTask；`finish_task` 与异步回调在发现 viewer 已打开时也清除该意图。回调随后因 active follow 不再等于 captured follow 返回，不替换旧 preview。关闭查看器只恢复旧阅读焦点，不会重新建立跟随，因此“读入中打开后又关闭”也不会让旧回调重新生效。它没有自动关弹层，也没有清除不同的 workbench Task 意图。新增测试覆盖清除后不能按原票据完成以及工作台意图保持。

该补丁复核时，两项在源码上的缺口已补，但尚无查看器的原生完成链路。以下 v9 追加区分实际发生的查看器路径与本次没有触发的慢读竞态。未执行 cargo/GUI/提交。

## v9 查看器原生闭环与最终限制

本轮独立实际查看了以下四张原图及各自 `.ax.txt`，读取 `v9-reader-viewer-task-evidence.json`，并只读交叉核对 wide 工作区中的对应任务计划、parent、完成状态和两版 manifest。GUI 操作顺序来自主代理的执行记录；本审查者没有操作 GUI。

| 原始证据 | 实际可观察结果 | 能证明的范围 |
| --- | --- | --- |
| `v9-repair-viewer-open-processing.jpg` | `BV1UXPRODUCTVIEWERFOLLOW` 的截图查看器打开，00:00、1 / 1、100%，蓝色截图与同期人工字幕完整；背景任务计数为 1。 | 补做处理阶段，用户已进入旧版截图查看器；不只是任务详情页等待。 |
| `v9-repair-viewer-completed.jpg` | 背景任务计数退出，查看器仍是同一 00:00 截图和同期文字，“在笔记中查看”“从此处观看”“收起文字”“关闭”仍可见；AX 的查看器内容和动作相同。 | 结合任务 complete 及新产物证明，这不是 worker 尚未结束的假通过。结果完成没有自动关掉弹层或在这里展示新版文字。 |
| `v9-repair-viewer-return-old-note.jpg` | 点击“在笔记中查看”后，旧版仍显示原有“正文已保存，AI 校对、摘要未完成”，页面出现“这份笔记已有新版 / 查看新版”；正文保留 00:00 蓝图和对应人工字幕。 | 查看器关联的旧正文仍可到达，没有因背后换版而变成只关闭弹层；用户保有明确的新版入口。这份夹具只有一个 00:00 锚点，不能据此声称长笔记中任意远处锚点的像素位置已验证。 |
| `v9-repair-viewer-open-new-version.jpg` | 随后显式点击“查看新版”，页面标示版本 2，出现已生成摘要、命名章节“分散投资”及相应目录，旧版的未完成提示退出。 | 停止自动跟随没有丢失新结果，手动打开新版路径成立；不把这个手动结果误记成自动换版。 |

实际任务链也与原图一致：原任务 `task-18d3a64afc4a18a0-dbfe-7` 为 partial，manifest revision 1，transcript/screenshots succeeded、proofreading/summary failed。后续 `task-18d3a66103456d38-e415-1` 的 parent 指向该任务，冻结 operation 是从原版目录补 proofreading/summary；它为 complete，created 1788956412、updated 1788956436，相差 24 秒。新 artifact 是另一个版本目录，manifest revision 2、partial false，两项 AI outcome 均 succeeded。因此“完成时仍保留旧查看器”和“旧版提供新结果入口”确实面对已经生成的新版本。

已读取主代理的 `desktop-unit-tests-v9.log`：**223 passed、0 failed、3 ignored**，并逐项确认以下相关测试均为 `ok`；本审查者没有重新运行它们。

- `reader_repair_opens_the_new_version_only_while_the_original_note_is_still_open`
- `reader_repair_guard_excludes_exports_and_rechecks_the_note_when_loading_finishes`
- `opening_a_reader_viewer_ends_repair_following_even_if_it_closes_before_the_read_finishes`
- `repair_position_handoff_uses_the_latest_old_position_and_keeps_existing_versions`

最终判定：**R2“运行中打开查看器，完成后仍能返回其旧正文，再手动进入新版”的实证路径关闭，没有发现该链路的新实质缺陷。** R1 已完成必要实现与位置交接数据测试；本次小夹具没有人为延迟文件读取，没有原生触发“读取中继续滚动”的竞态。R2“新版异步读入期间打开又关闭查看器”由入口清除意图、完成/回调守卫的静态接线及状态单测覆盖，同样没有原生触发该极窄时间窗。两者保留为明确的验证限制，不宣称全面原生竞态覆盖。

v9 没有再次点击原视频、缩放等其他查看器动作，也未复跑 Partial 新结果、离开又返回、切换不同笔记或 exports-only；这些分别沿用本报告已有的静态检查、此前原生证据或相关测试范围。本轮仅更新审查报告，没有修改产品代码、运行 cargo、操作 GUI 或提交。
