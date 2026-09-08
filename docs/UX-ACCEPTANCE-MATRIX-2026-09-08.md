# UX 完整验收覆盖表

目标：完成 [规格](UX-REDESIGN-SPEC-2026-09-07.md) 中的核心产品流程。2026-09-08 用户明确取消可访问性专项：不再将 VoiceOver、全页面纯键盘及 200% 字号专项作为本次完成条件；正常窗口下的可用性和核心操作仍需验证。历史证据见 [实施与验收记录](UX-VALIDATION-2026-09-07.md)，本表记录本轮核心功能的最终验收结果。

验收采用原生操作、真实转换和自动回归组合：原生证据验证页面行为，自动测试验证交叉组合、持久化和故障边界。表中的“通过”不表示每个排列都做了手工点击，也不代表真实平台、所有硬件或所有浏览器已验证；具体范围以证据栏和下方限制为准。AT33 取消，AT34 按调整后的正常窗口范围验收。

| 编号 | 核心覆盖范围 | 状态 | 证据/结果 |
| --- | --- | --- | --- |
| AT01 | 全新配置、无 ASR/云服务、同名字幕直接生成和阅读 | 通过 | `at01-fresh` → `at01-plan-visible` → `at01-internal-reader`；真实转换完成，0 服务请求、无模型缓存、原文件哈希不变；按能力缺工具另见 AT18 |
| AT02 | 延迟 A/B、在线/本地 C、独立选项与文件夹、重启 | 通过 | 第七轮原生迟到 A/B、在线 B/本地 C 和重启；`54ec122` 的复合测试补齐两份草稿的独立文件夹、JSON/网页、AI 覆盖、默认位置变化和冻结队列。 |
| AT03 | 分集、多语言人工/自动字幕、本地附加字幕、真实执行来源一致 | 通过 | 第四/五轮原生单选 p=2、法语人工字幕和中文/自动字幕标记；第九轮附加法语字幕真实生成，第十轮 p=2 仅 JSON 生成；来源身份与正文核对一致。 |
| AT04 | 无字幕、超时、登录受限及同一草稿恢复，无隐式音频外发 | 通过 | 第七轮 `v23-login-recovered`、`v23-timeout-recovered` 与三态证据；恢复后任务/请求计数不增加，未把失败当作无字幕。 |
| AT05 | 缺字段的云识别在派发前阻断，字幕路径不受其影响 | 通过 | 原生半填识别服务保存及生成均阻断，模型/凭据有缺项提示、任务数不变、0 请求；更换外部字幕后成功生成。核心集成测试另覆盖在线完整下载前缺凭据阻断 |
| AT06 | 本地内嵌标题、手工名称、外部字幕、原文件哈希与路径 | 通过 | 内嵌标题预览后手工改名，附加法语字幕成功生成；草稿、任务、阅读页标题一致，9 份源文件哈希和原路径不变，本地无下载视频保留选项 |
| AT07 | 最小/宽窗完整表单、折叠计划、自定义 AI/识别参数 | 通过 | 第七至九轮最小窗/宽窗工作台与设置实机；第十轮实际 AI 组合请求及旧隐藏模型/规则导入，选项说明、有效参数和输出一致。 |
| AT08 | 零附加导出、仅网页、仅 JSON，三次内部阅读与复制 | 通过 | `at01-internal-reader`、`at08-html-reader`、`at08-json-reader` 与复制反馈；三次真实转换分别为零附加导出、仅 HTML、仅 `structured.json`，内部正文与图片独立保存。 |
| AT09 | A/B 默认服务、独立覆盖/归属、仅本次保存与明确设默认、返回焦点 | 通过 | 第七轮原生返回原草稿与焦点；`preferences::tests::task_only_publication_and_key_rotation_do_not_change_default_or_old_snapshot` 验证仅本次/默认版本隔离；`54ec122` 补齐归属、覆盖和重启组合。 |
| AT10 | 半填服务、独立应用偏好、写盘失败与重试 | 通过 | 第五轮只读目录失败/重试实机；偏好组独立提交、未发布服务草稿和恢复意图的自动回归通过。 |
| AT11 | 未等防抖立即提交、保存失败时零派发及就地恢复 | 通过 | 第五轮立即提交失败的任务/服务/提取器计数均不增加，就地恢复成功；`failed_commit_does_not_publish_an_unpersisted_task` 验证持久化失败不发布任务。 |
| AT12 | 覆盖/继承、排队快照与后续默认更改 | 通过 | `54ec122` 及 `source_revisions_and_default_overrides_are_independent`、`frozen_task_survives_other_drafts_and_defaults_without_storing_credentials`：覆盖保持、继承更新、排队快照不变。 |
| AT13 | 有效甲、未发布乙、开始任务和重启、明确发布范围、凭据隔离 | 通过 | 第一/七轮真实服务甲和未发布乙的请求核对；偏好测试验证重启保留草稿、明确发布与凭据隔离，后台输出脱敏测试通过。 |
| AT14 | 用途契约有效/格式错误/401/限流/未知、改模型失效、无隐式测试 | 通过 | 第一/七轮有效/错误 200/401/429/未知实机；`service_test` 用途契约、无自动重发和样本内容测试；更改字段使旧测试失效。 |
| AT15 | 校对/摘要/截图/规则及旧配置组合，说明与实际请求一致 | 通过 | `ffd5a8c` 真实 HTTP 校对请求含截图及 UX-RULE-42，摘要请求无图/音频；第五轮仅摘要成功补做；第十轮旧规则、模型和仅摘要组合导入并保留。 |
| AT16 | 模型缺失、已知/未知体积、下载中断恢复、离线、设备与模型分开 | 通过 | 真实 Apple 缓存和推理；`919e40a` 验证截断、已知/未知体积与离线下载边界。`714dccd` 后完整缓存离线推理约 1.09 秒、0 Hub 请求；缺缓存离线准备明确失败，不标就绪。 |
| AT17 | 已保存 GPU 变为不可用，保留选择、明确切 CPU、不转云 | 通过 | `at17-gpu-unavailable` → `at17-restart-keeps-gpu` → `at17-explicit-cpu`：受控 CPU-only 能力环境保留 GPU 选择，明确切 CPU 后仍区分模型缺失，0 服务请求。 |
| AT18 | 本地/在线缺工具的不同阻断范围、完整诊断与修复入口 | 通过 | `at18-application-diagnostics`、`at18-local-subtitle-no-ytdlp`、`at18-online-submit-blocked`；仅本地任务成功，在线派发前阻断，界面给安装/重检入口。诊断协议另由 `cli_ux` 自动测试验证。 |
| AT19 | 损坏设置：有备份、无备份、备份不可读，保全与恢复 | 通过 | `at19-corrupt-legacy-actions`、`at19-backup-restored`、`at19-reset-write-blocked`、`at19-reset-retry-succeeded`；损坏原件先保全，可用备份恢复；备份不可读及目录只读失败均保留原件。 |
| AT20 | 分段识别中断续做、成功空结果、未知云请求准确授权范围 | 通过 | 第一/五轮未知请求与明确补做实机；ASR 检查点测试含成功空结果；`real_http_uncertainty_requires_exact_authorization_and_reprocessing_never_repeats_asr` 核对真实请求次数和用途范围。 |
| AT21 | 关窗/退出/崩溃/冷启动、暂停与确定未发/未知请求 | 通过 | `8070190` 真实记录冷启动组合覆盖 Run/Pause/Quit/Cancel/Uncertain；既有进程树取消测试、原生多次退出重开。`284900c` 保留退出前失败/未知状态和未读提示，同时禁止自动派发。未重启宿主操作系统，系统重启等价的磁盘恢复边界由测试验证。 |
| AT22 | A 运行时 B 草稿；失败后继续与调整 A，不改写 B | 通过 | 第一轮后台独立 C 草稿；`8070190` 将界面调整操作放入任务状态层并测试：A 运行时不能修订，继续/调整均来自 A 的完整快照，B 不变，修订重启恢复，后续提交幂等。 |
| AT23 | 设置/其他阅读页上后台成功、失败、部分成功，提示与任务入口 | 通过 | 第十一轮真实成功 A/截图部分失败 B/来源变化失败 C；设置和 A 阅读页不跳转，提示含来源及动作，任务入口显示数量；查看/关闭才清提示，重启保留记录。 |
| AT24 | 校对/摘要/截图/导出各阶段失败与仅补失败项，旧正文保留 | 通过 | 第五轮原生仅补摘要；真实 HTTP 集成测试覆盖校对/摘要失败及恢复；`screenshots-retry/evidence.json` 验证仅补截图不再识别/下载、旧文件哈希不变；导出恢复测试在原素材已移除时只补失败格式。 |
| AT25 | 同源改名/归属、人工编辑旧导出、新版发布前失败/崩溃/空间不足 | 通过 | `77781b4` 模拟新版已写入但 current 提交不可写，旧指针与人工修改导出不变，重试发布一次，重放旧版不能回滚；同源改显示名/文件夹不创建重复工作由工作区测试覆盖，实际法语新版保留旧版哈希。 |
| AT26 | 失败/损坏记录、孤立正文、旧好新坏，无假统计/空阅读 | 通过 | 第二至四轮原生历史 Markdown/HTML/JSON 与无正文材料；第十轮人工笔记和失败目录重启发现；legacy_import、artifact、backend 测试覆盖损坏/孤立正文、旧版保留与无伪统计。 |
| AT27 | 两位置默认/筛选/文件夹新建、旧队列、迁移各阶段中断 | 通过 | `54ec122` 的两位置普通新建/文件夹新建/旧队列组合；storage 测试覆盖复制中断、取消、重试和原件保护；`5bc8a6f` 实际提交不可写后冷启动仍用旧根，登记已提交但日志落后时仍用新根。 |
| AT28 | 局部无权限、缺正文、位置离线、分类损坏，健康笔记可读 | 通过 | 第五轮原生整个位置离线/恢复；storage 多位置可读覆盖和 organize 分类备份/重建测试，legacy_import 只读/坏资料隔离测试，健康正文不被误判为空库或覆盖。 |
| AT29 | 生成新文件夹、库内重命名/删除、弹层范围与返回归属 | 通过 | 第九轮原生创建和空名称错误；`at29-renamed-folder`、`at29-delete-explained`、`at29-delete-keeps-note`；删除明确保留笔记，18 个原文件哈希不变，文件夹元数据单独更新。 |
| AT30 | 搜索折叠组内结果，清空恢复折叠状态，无结果可清空 | 通过 | `at30-grouped` → `at30-search-opens-group` → `at30-clear-restores-collapse` → `at30-no-results`：折叠组命中直接可见、清空恢复折叠、无结果可清空。 |
| AT31 | 长文中段、截图、离开/重启、版本独立锚点、实际来源/时间 | 通过 | 第八轮同版中段/截图/设置/退出重开；`5fb3b95` 后 60 秒长文两个版本分别保留 30 秒/10 秒锚点，五张截图独立，20 秒图片回跳正文；实际段落/网格几何测试通过。 |
| AT32 | 两种复制、Markdown 包/内嵌图片网页、移走原库后独立打开 | 通过 | 第八轮原生两种复制与保存面板导出，移开原库后文本/图片哈希验证；`portable::tests::all_exports_survive_without_the_original_library_and_never_overwrite` 通过。独立网页视觉打开沿用第四轮 Chrome 证据，本轮 file URL 访问被工具策略拒绝，未重做该视觉检查。 |
| AT33 | 实际 VoiceOver 专项 | 本轮取消 | 用户明确调整范围；不记为通过，不再安排专项验证 |
| AT34 | 正常窗口核心操作、加载与空态 | 通过（调整后范围） | 按调整后的范围通过：860×620 与宽窗的普通生成/设置/任务/阅读、加载和无结果状态已有原生证据；第十一轮补全结果提示与任务入口。取消的专项不计通过。 |
| AT35 | 旧配置/隐藏选项/人工文稿/失败目录升级，多次重启幂等 | 通过 | `4d9f7e2` 恢复实际旧配置入口并在导入前校验备份；`at35-imported-options`、`at35-manual-note-readable`、`at35-restart-library`；隐藏 0.6B、仅摘要、自定义规则和人工图文保留，重启无重复导入。 |

新一轮使用 `/tmp/course2md-ux-complete-20260908` 独立验证根目录。配置、媒体、服务和课程状态均为合成验收数据；旧验证根目录保留供证据回查。

## 第八轮：首次生成、阅读与导出

使用构建标签 all01–all10，窗口 860×620。all01 从全新配置开始；后续复用同一份合成课程库核对阅读位置。下表文件名位于验证根目录的 `screenshots/`。

| 范围 | 操作与观察 | 证据 |
| --- | --- | --- |
| 首次生成 | 空库进入工作台，选本地 `lecture-a.mp4`，自动读取同名字幕，AI 与附加导出均关闭。真实转换完成，任务为 complete，截图和正文成功；没有模型目录和服务请求；9 份原媒体/字幕哈希不变 | `at01-fresh`、`at01-plan-visible`、`at01-internal-reader`、`at01-evidence.json` |
| 两种复制 | 分别执行复制纯文本和 Markdown 文本，再在两个新建 TextEdit 文档粘贴；正文到第 18 段完整，格式与反馈对应，未带图片引用 | `at32-copy-plain-feedback`、`at32-pasted-plain`、`at32-copy-markdown-feedback`、`at32-pasted-markdown.ax.txt` |
| 带图导出 | 通过原生保存面板导出 ZIP 与 HTML；ZIP 有相对图片引用，HTML 有内嵌图片，两者图片 SHA256 一致。临时移开原库后检查正文和图片仍完整；随后恢复原库 | `at32-md-export.ax.txt`、`at32-html-export.ax.txt`、`at32-portable-structure.json` |
| 导出限制 | 浏览器安全策略拒绝访问本地 file URL；未改用其他浏览器或间接地址绕过。当前轮只记录结构检查，独立打开的视觉验证保持待验 | `at32-portable-structure.json` |
| 正文锚点 | 滚至正文中段，切到截图再返回、进入设置再返回、明确退出后重开笔记，均恢复同段；此处只有一个版本、一个截图时间，不能推定跨版本与多时间点通过 | `at31-middle`、`at31-return-from-screenshots`、`at31-return-from-settings`、`at31-restart` |
| 200% 阅读修复 | 原头部完全挤走正文和导出入口；限制头部高度后正文可见。Tab 能到达笔记页签，左右键切换截图；查找完整匹配“第 18 段”；导出按钮自动滚入视口；Page Down 实际移动正文 | `at34-reader-200-before`、`at34-reader-200-after`、`at34-reader-tabs-fixed.ax.txt`、`at34-reader-find-200`、`at34-reader-export-tab-settled` |
| 图片与弹层修复 | 原图片容器被长转录文字压缩为零高；修复后正文和弹层图片可见。激活原生窗口后，从正文图片用 Return 打开弹层，再以 Tab 到底部“在笔记中查看”，操作有可见焦点并能返回原文；最终构建已复验 | `at34-image-viewer-200`、`at34-inline-image-200-fixed`、`at34-image-viewer-200-fixed`、`at34-image-clean-keyboard` |
| 开关焦点 | 设置应用页在 200% 字号下 Tab 到减少动态效果，开关有清楚的深色外框，未增加额外 Tab 停靠点 | `at34-switch-focus-after` |

原生工具只发送 AX/键盘事件时，出现过 AX 布局已更新而窗口画面仍停在旧帧的情况。验收时先用窗口内点击激活原生窗口，再执行纯键盘路径；未把未激活窗口的截图当作通过证据。排查中的临时日志和额外重绘尝试已移除。

验证：桌面完整测试 103 通过、3 忽略（`tests-reader-complete.log`）；后续阅读相关测试 10 通过（`tests-images-reader.log`），焦点几何测试 1 通过（`tests-focus-final.log`）；最终干净构建成功（`build-images-clean.log`）。核心转换未修改，沿用本轮首次生成的真实执行与前轮 165 通过、2 忽略的核心测试记录。

## 第九轮：外部字幕与弹层确认键

使用 all10–all13，沿用同一隔离配置与合成媒体。服务编辑器只填写名称和本机受控地址，留空模型及凭据；保存与生成均显示缺项，仍为 1 个任务、0 个已发布服务版本、0 次服务请求（`at05-incomplete-service`、`at05-submit-blocked.ax.txt`、`at05-preflight-evidence.json`）。核心集成测试 `missing_frozen_cloud_credentials_fail_before_download_and_ignore_environment` 覆盖在线下载前相同的阻断边界。

本地 `lecture-external.mp4` 先附加无效字幕，页面保留视频并提供重选入口（`at06-invalid-subtitle`）。随后附加 `different-name.fr.srt`，改名为“外部字幕验收：风险与分散”，仅选网页导出，真实任务成功。正文包含两条法语字幕，当前版本仅有 `exports/course.html`；未完成的识别服务草稿不影响字幕生成。证据：`at08-html-reader`、`at06-html-evidence.json`；3 个标题位置一致，9 份媒体与字幕哈希全部不变、0 服务请求。

此过程中复现了弹层 Enter 键错误：焦点位于“创建文件夹”时，通用 Dialog Confirm 在按下时先关闭弹层，按钮在释放时的实际动作未执行。取消该通用 Enter 绑定后，由当前聚焦控件处理确认，Escape 保持原行为。all13 复验空名称时弹层保留并显示错误；有效名称创建“金融学验收”并返回正确归属。图片弹层焦点在“放大”时 Return 将 100% 改成 125% 且弹层保持打开；焦点在“关闭截图”时 Return 正常关闭。证据：`at29-create-keyboard-before`、`at29-empty-keyboard-fixed`、`at29-create-keyboard-fixed`、`at34-dialog-image-enter-fixed.ax.txt`。构建通过（`build-dialog-enter.log`）。

原生文件选择面板关闭后出现自动化读取超时，进程采样显示主事件循环正常。通过明确退出并以新验证标识启动恢复连接，持久化草稿保留有效字幕；未据此声称应用崩溃。法语句子合并缺少句间空格已在 `8d75f37` 修复：合并拉丁文字时识别扩展拉丁字母和句末标点，保持中日文及连字符、撇号片段原有行为；时间线 11 项测试通过。all14 真实重新生成的法语正文包含正确句间空格，旧版 8 个文件哈希不变。工作台末尾按钮的焦点滚动专项按用户后续要求停止，其未提交改动已撤回；all14 的相关焦点截图不作为最终构建证据。

## 第十轮：核心生成、阅读、模型与旧资料

沿用 `/tmp/course2md-ux-complete-20260908`；原生应用 all14–all16、capability01–03、legacy01–03 分别用于正常生成、受控能力缺失和旧资料升级。截图仍在根目录 `screenshots/`；能力与旧配置各用自己的配置和课程库。

| 范围 | 结果 | 证据 |
| --- | --- | --- |
| 仅 JSON | 受控在线第二集使用已确认中文字幕，实际生成 `exports/structured.json`；内部正文和图片可读，复制有反馈，没有 ASR 或 AI 请求 | `at08-json-reader`、`at08-json-copy-feedback` |
| 多版本长文 | 真实转换生成 60 秒长文、5 张时间不同的截图和两个版本。修复按外层块误算段落坐标、切版覆盖旧锚点的问题；V2 保存约 30 秒、V1 保存约 10 秒，回到 V2 仍为 30 秒。20 秒截图可回到对应正文 | `at31-versions-positions.json`、`at31-v2-independent-restored`、`at31-grid-restored`、`at31-frame-20-body`；`tests-reading-layout.log` |
| 文件夹与搜索 | 创建后重命名、删除文件夹，提示明确保留笔记；原文件哈希不变。搜索显示折叠组内命中，清空后恢复折叠，无结果有清空入口 | `at29-delete-explained`、`at29-delete-keeps-note`、`at29-files-before-delete.json`、`at30-search-opens-group`、`at30-clear-restores-collapse`、`at30-no-results` |
| 能力与工具 | 受控环境模拟只剩 CPU 和缺少 yt-dlp；保存的 GPU 选择重启仍可见且标出不可用。明确切 CPU 后仍说明模型缺失。本地已确认字幕任务成功；在线任务在派发前阻断，任务总数仅增加本地一项 | `at17-restart-keeps-gpu`、`at17-explicit-cpu`、`at18-application-diagnostics`、`at18-online-submit-blocked`、`at18-task-boundary.json` |
| 旧配置修复 | 发现设置重构后遗失修复入口，已接回实际诊断、恢复备份、保全原件后重建、查看路径和重检。可读备份恢复成功；备份不可读且保全目录只读时失败，原件不变，恢复权限后重试成功 | `at19-corrupt-legacy-actions`、`at19-backup-restored`、`at19-reset-write-blocked`、`at19-reset-retry-succeeded` |
| 旧配置导入 | 正常导入也先生成经校验的原件备份；导入保留旧 0.6B 模型、仅摘要、规则与截图辅助原配置，实际仅摘要不发送截图。人工 Markdown 及图片哈希不变，失败目录单列，重启没有重复导入和重新生成 | `at35-import-with-backup`、`at35-imported-options`、`at35-manual-note-readable`、`at35-restart-library`；`legacy-case/preferences-before-restart.json` |
| AI 组合与局部恢复 | 真实 ffmpeg 和本机 HTTP 测试校验校对请求包含图片和自定义规则，摘要无图片/音频；截图失败后只补图成功，旧版 5 个文件不变，未重新识别或下载；导出恢复在原媒体/字幕移除后仍能进行 | `tests-ai-combinations.log`、`screenshots-retry/evidence.json`；`tests/task_execution.rs` |
| 本机与云端识别 | 实际音频提取、受控 API 上传和文稿发布成功；Apple Qwen3 1.7B 实际识别出合成英文讲解。发现完整缓存仍联网上游，首次需约 235 秒；修复后新工作目录完整推理约 1.09 秒，模型准备和推理均为 0 Hub 请求 | `asr-end-to-end/api-evidence.json`、`asr-end-to-end/coreml-evidence.json`、`asr-end-to-end/coreml-offline-verified/evidence.json` |

Apple 加载现在使用已核验的本机缓存；缺失文件留给准备阶段下载，并尊重 `HF_HUB_OFFLINE`。明确离线且缺文件时返回失败，保留缓存检查原因。Qwen3 0.6B 和 Whisper 的相同离线参数已编译验证，本轮实际推理只覆盖 Qwen3 1.7B。CPU-only 是能力探测夹具，不能当作真实 CPU 推理性能测试。

补充自动回归按独立提交保留：`54ec122`（草稿/默认/归属/队列/重启组合）、`77781b4`（发布中断与旧导出保护）、`ffd5a8c`（实际 AI 请求）、`919e40a`（下载完整性）、`8070190`（任务修订与冷启动）、`5bc8a6f`（迁移登记提交失败及日志落后）。迁移测试实际写入并重新打开记录，取消和恢复只处理登记的临时文件，保留旧资料；不通过重启宿主机器制造故障。

## 第十一轮：后台结果、退出与最后回归

`background-case/` 使用三份已保存的合成任务：A 成功，B 提取截图失败但正文成功，C 的过期来源指纹触发确定失败。background01–04 的工作进程包装器只将 `run-task` 启动推迟 18 秒，随后执行真实转换程序，以便在结果出现前切到设置或阅读页；不替换转换结果。background05–06 使用直接的真实工作进程。

发现并修复两个产品问题：

- `eac75d7`：顶部缺少任务入口、其他页面缺少后台结果提示。现在任务入口显示实际未查看/需要处理数量，结果提示带课程名和操作，只有查看对应结果或关闭提示才消失，状态跨重启保留。
- `284900c`：退出时把失败和未知请求的状态统一改成暂停，导致需要处理数量失真。现在记录退出意图的同时保留这些状态和原因，恢复后不会自动派发。

| 操作 | 实际观察 | 证据 |
| --- | --- | --- |
| A 在后台成功 | 留在设置，提示《后台成功 A》及“查看生成结果”；点击后打开 A 的任务并清除未读 | `at23-success-notice-in-settings`、`at23-unread-success-after-restart` |
| 阅读 A 时 B 部分完成 | 阅读标题、版本和正文仍为 A；提示 B 的未完成部分。关闭提示后仍在 A，任务页可以只补截图 | `at23-reading-a-while-b-runs`、`at23-partial-stays-reader`、`at23-dismiss-keeps-reader` |
| C 确定失败 | 设置页保留，提示准确指向 C；查看任务显示来源指纹变化的原因，未创建伪成功笔记 | `at23-failure-stays-settings`、`at23-failure-opens-c` |
| 退出及重启 | 未查看的失败提示、原因和需要处理状态保留，860×620 下导航、提示和按钮可见；已查看/关闭的提示不重新出现 | `at21-failure-before-quit`、`at21-failure-after-restart`、`background-case/quit-failure-evidence.json` |
| 运行中明确退出 | B 的仅补图任务保存退出意图；重启显示已暂停并提供继续，没有自动重做或创建第二个补做任务 | `at21-reprocess-before-quit`、`at21-running-quit-restores-paused`、`background-case/quit-running-evidence.json` |
| 仅关闭窗口 | 明确继续同一个补图任务后关闭窗口，原进程仍在；后台成功保存截图，重新显示窗口仍为同一任务，无额外补做记录 | `at21-close-window-completes-same-task`、`background-case/closed-window-evidence.json` |

最终全量回归暴露测试用本机 HTTP 服务的 `WouldBlock` 偶发失败，已显式将接收的连接设为阻塞读取，并避免清理时二次 panic；修复后重新运行完整核心测试通过。该修改仅在测试夹具中。

验证根目录中的 `tests-core-final-verified.log` 记录核心全量 169 通过、2 忽略；`tests-desktop-final-verified.log` 记录桌面全量 108 通过、3 忽略。最终桌面构建通过；`tests-quit-outcomes.log` 另核对失败/未知请求在退出后的持久化。忽略项为真实公共平台连接及显式外部工具场景；本轮本地媒体已有独立实机与集成验证。

### 完成边界

本轮核心实现、发现的问题修复及组合回归已完成。真实平台登录/网络、非本机硬件推理、完整设备矩阵未由受控夹具替代。AT32 的当前完整导出已验证文本、图片、路径独立性和原件保护；第四轮有 Chrome 独立打开的视觉证据，本轮浏览器工具拒绝本地 file URL，因此没有新的浏览器视觉复验，也未绕过限制。AT33 及 AT34 中用户取消的专项不计为通过。
