> 归档说明：本文件复制自本轮独立审查记录，只调整图片／日志引用为归档内相对路径。保留逐轮发现与当时的待验证状态；最新结论在末尾更新段。同名最终测试日志按主代理最后一次运行归档，历史段落中的数字保留当时记录；当前结果见主报告测试表。各位审查者未亲自执行的行为仍按原文保留边界。

# 阅读、截图图库与笔记库独立原生审查

审查日期：2026-09-09。审查者：`reader_layout_review` 子代理。范围是原生 GPUI 的笔记库列表/卡片、阅读正文、截图图库、查找与图片查看器。所有下列图片均由主代理操作原生应用后提供，本代理通过 `view_image` 独立查看；没有运行 GUI、Cargo 或 Git 提交。最后一项成功重试提示于不含 performance 的 release 9ec2f6e 完成原生闭环，本代理当前没有保留必须修改源码的已知问题；后文仍明确区分已观察结果与覆盖限制。此记录没有把历史审查关闭状态当成通过证据。

设计依据是仓库 `.agents/skills/course2md-design/` 的当前规则，以及实际的 `src/render.rs::render_html`、`src/summarize.rs::render_html_block` 阅读排版。原生正文采用共同对齐轴、自然图片比例和正常字重长文；用户明确要求正文图片不加卡片边框。主要操作使用共享图标与 semibold 标签，辅助信息与正文保持正常字重。

## 缺陷与验证结果

截图路径均位于本文件所在目录 本归档目录。

| 问题 | 修复与实际证据 | 当前判断 |
| --- | --- | --- |
| 返回、标题、元信息、正文原先各有左边界；图片被卡片边框/留白包住 | [after-22-reader-multisection.jpg](after-22-reader-multisection.jpg)、[check03-29-wide-reader.jpg](check03-29-wide-reader.jpg) 可见标题、元信息、工具栏与正文共左轴，图片完整并采用自然比例。目录作为侧栏，未把正文压成过窄栏 | 正常浅色、宽屏深色 125% 构图通过 |
| 宽屏目录默认关闭、持久选中易被按钮绘制盖掉 | [check03-29-wide-reader.jpg](check03-29-wide-reader.jpg) 展开目录并显示当前 00:00 的持久背景。代码改为自动宽度判断加独立手动选择状态，选中背景在外层容器 | 自动展开和选中外观通过；静态截图不能证明手动选择跨 resize 的全部行为 |
| 图库过多窄列，摘要与动作高低不齐 | [after-23-gallery-six.jpg](after-23-gallery-six.jpg)、[check03-30-wide-gallery.jpg](check03-30-wide-gallery.jpg) 显示三列真实可读图宽，摘要三行区与动作同基线；[check03-23-narrow-gallery.jpg](check03-23-narrow-gallery.jpg) 显示 200% 字号单列自然滚动 | 已观察的正常浅色/宽屏深色/窄屏构图通过 |
| 大字号图库摘要行高取整后可能超过预留高度约 1px | 摘要实际像素行高先取整，再据此留足三行高度，摘要和动作均不收缩。e217 debug 的 [check10-39-small-gallery-caption-actions.jpg](check10-39-small-gallery-caption-actions.jpg) 至 [check10-41-small-gallery-action-baseline.jpg](check10-41-small-gallery-action-baseline.jpg) 连续显示同一卡片；41 完整展示两行长摘要和定位正文/原视频动作 | 200% 已观察的两行摘要与动作同屏通过，动作基线一致、没有重叠；这不是一张恰好占满三行摘要的截图 |
| 本地视频文件名、移动文件动作和元信息重复挤占阅读头部 | 正常来源只展示时长/本地视频与“打开原视频”；文件名和重定位放入生成信息。[check03-29-wide-reader.jpg](check03-29-wide-reader.jpg) 显示头部更清楚；文件夹动作有“在文件夹中显示”标签 | 正常宽屏可见状态通过 |
| 初次打开的摘要标题可能被异步测量推离顶端 | 捕获 offset 为顶端时保留真正顶端；摘要导航测量标题本身；重排恢复保留段落锚点与段内进度，增加三项行为测试 | 修复已写入。测试执行及完整原生“首开顶端/摘要导航/resize 不跳段”记录由主代理汇总，本代理没有独立执行，不能仅据静态图片关闭所有行为风险 |
| 库 metadata 继承粗体，同名笔记难区分；新增数量一度在最近笔记重复 | 库 metadata 改正常字重，列表使用已存在的截图数；卡片/最近笔记已有描述数量时不再重复。[check03-28-wide-library.jpg](check03-28-wide-library.jpg) 显示版本/日期与 1、6 张截图各出现一次 | 宽屏库列表通过；没有新增文件 IO |
| 860×620、200% 库列表缩略图和右侧操作挤到标题只剩“信…” | 小宽度按上下结构排标题与信息、操作另起行，移除列表缩略图占宽。[check04-02-narrow-library.jpg](check04-02-narrow-library.jpg)、e217 debug 的 [check10-30-small-library.jpg](check10-30-small-library.jpg) 标题完整、metadata 数量仅显示一次；[check10-33-small-library-aligned-actions.jpg](check10-33-small-library-aligned-actions.jpg) 完整显示同一条目的文件夹/阅读/更多动作行 | 已观察状态通过，三项操作同高对齐，列表允许正常垂直滚动；33 的选择仍为列表，不能用作卡片模式证据 |
| 窄阅读搜索计数被限高区裁掉半行 | 搜索区移出附加信息的 28% 限高容器，输入行/计数不收缩，窄屏省略重复匹配摘要。[check04-03-narrow-search-empty.jpg](check04-03-narrow-search-empty.jpg)、[check04-04-narrow-search-result.jpg](check04-04-narrow-search-result.jpg) 显示输入与控件完整，计数整行可见，正文匹配高亮正确；最终 [check07-05-narrow-search-title.jpg](check07-05-narrow-search-title.jpg) 在 860×620、200% 字号下同时显示最新完整标题、完整“第 1 处，共 1 处”计数及正文首行命中高亮 | 空查询、一条结果及最新标题组合通过，待复拍项已关闭；这不是“零条结果”状态截图 |
| 200% 标题最后一个“骤”字单独占行，标题左边界仍被返回按钮推开 | check04-03/04 明确失败，随后改为所有宽度均“带标签的独立返回行 + 全宽标题”，窄屏最多两行。[check06-02-narrow-reader-title.jpg](check06-02-narrow-reader-title.jpg) 可见完整中文标题单行、与正文同左轴，返回标签独占一行，正文和下一段 00:15 仍有空间 | 最新标题构图通过。主代理报告原阅读位置未重置；静态图显示当前段落，但不单独证明整个恢复过程 |
| 低矮窗口查看器默认展示转录，图片只剩约 100px 高、14% fit | 高度不足 32rem 时默认收起文字，显式“显示文字/收起文字”保留用户选择。[check04-06-narrow-viewer-image.jpg](check04-06-narrow-viewer-image.jpg) 图像以 35% 完整展示，正文/原视频/显示文字/关闭四项同排 | 图片优先和窄屏 footer 构图通过 |
| 查看器长转录必须独立滚动，不能覆盖底部操作 | [check04-07-narrow-viewer-text.jpg](check04-07-narrow-viewer-text.jpg) 展开文字，[check04-08-narrow-viewer-scroll.jpg](check04-08-narrow-viewer-scroll.jpg) 可见末尾“图片。”，底部操作始终可见；展开后图片重新适配到 14% | 已观察的展开与滚至末尾通过。fit/缩放中心/viewport 测量算法保持原实现；本次截图未独立重放所有手动缩放比例 |
| 窄屏卡片标题、metadata 与操作可能拥挤 | check04-21/22/23 已显示封面、完整标题与可滚达阅读入口。补充的 [check10-42-small-library-cards.jpg](check10-42-small-library-cards.jpg) 明确选中卡片模式，43 显示完整标题与不重复的普通字重 metadata，[check10-44-small-library-card-actions.jpg](check10-44-small-library-card-actions.jpg) 显示文件夹/更多完整辅助动作行与下一行全宽阅读入口 | 标题、metadata、辅助动作共轴与主操作可达通过，卡片辅助行待补项关闭。44 最底部按钮弧边在视口边缘自然裁切，不是叠压；整卡仍需滚动查看 |
| 空库需要指明下一步，不能呈现为失效的列表 | [check06-04-empty-library.jpg](check06-04-empty-library.jpg) 显示“我的笔记”页面标题、居中的“还没有笔记”、生成内容会保存到此处的解释，以及唯一突出的“导入视频”入口。没有无数据的筛选控件堆叠 | 正常浅色 100% 空库构图及用户方向通过 |
| 最新返回/标题结构在正常字号下也需成立 | [check06-13-background-result-reader.jpg](check06-13-background-result-reader.jpg) 中返回、完整标题、元信息、工具栏、摘要和正文采用清楚的共同边界，右侧目录独立。摘要标题可见，当前摘要目录项高亮 | 正常浅色 100% 构图通过。这张证明摘要在可见顶端，不替代所有异步布局恢复行为测试 |
| 自动完成 banner 使用全局壳宽，而阅读内容使用文章宽，是否产生混乱 | [check06-14-direct-result.jpg](check06-14-direct-result.jpg) 顶部绿色“笔记已生成，已为你打开”属于应用级反馈，位置在返回与文章标题上方，带独立关闭动作。更宽的反馈区与收窄文章分层清楚，正文内部对齐轴稳定，没有把状态误包装进正文 | 已观察到的完成反馈与阅读区域关系通过，没有必须修复的宽度冲突 |
| 生成信息只露前四行，卡片底边被截断，标签和值层级不足 | [check08-11-reader-information.jpg](check08-11-reader-information.jpg) 与 [check08-12-reader-information-settled.jpg](check08-12-reader-information-settled.jpg) 显示旧区域底边截断与同层小字。局部修复后，release 6c2e997 的 [check09-04-reader-information.jpg](check09-04-reader-information.jpg) 完整显示四项优先结果，05 显示信息末尾。e217 debug 的 [check10-35-small-reader-information.jpg](check10-35-small-reader-information.jpg) 至 [check10-37-small-reader-information-scrolled.jpg](check10-37-small-reader-information-scrolled.jpg) 在 860×620、200% 下从首项滚至重新定位动作 | 正常字号与窄 200% 的顶部/末尾均通过。图标+semibold 标签和值清楚，边框完整、滚动条不盖动作，35/37 的页头、信息区边框与固定收起入口位置相同；窄窗每次约显示一项信息，保留空间限制 |
| 分类记录故障重复显示两个黄框，第二框默认暴露 JSON 解析英文 | [check09-06-library-failure.jpg](check09-06-library-failure.jpg) 与 AX 确认重复提示。e217 release 的 [check11-02-library-recovery.jpg](check11-02-library-recovery.jpg) 只有一处恢复区，保留备份恢复、保留记录重建和打开位置；03 仅在展开诊断后显示原始英文；04 恢复并刷新后回到常态，文件夹可用 | 故障、诊断展开、恢复后构图通过。此组没有分别证明备份恢复与重建两个动作均已执行；主代理用 fixture 分类 JSON 制造故障，没有操作真实用户数据 |
| 笔记读取失败直接显示英文 I/O，没有说明下一步 | [check09-07-reader-file-missing.jpg](check09-07-reader-file-missing.jpg) 旧提示失败。主代理暂时重命名 fixture artifact 目录触发；e217 release 的 [check11-05-reader-file-unavailable.jpg](check11-05-reader-file-unavailable.jpg) 已显示保存位置/恢复文件后再次阅读或刷新课程库的中文建议，并保留原列表 | 失败文案和原库返回路径通过，没有声称文件永久丢失；其后的成功重试另发现旧提示未清除，见下一项 |
| 恢复文件后已成功阅读，旧“暂时无法访问”提示仍留在顶部 | [check11-06-reader-retry-restored.jpg](check11-06-reader-retry-restored.jpg) 显示旧缺陷。修复后，不含 performance 的 release 9ec2f6e 中，[check12-07-final-reader-unavailable.jpg](check12-07-final-reader-unavailable.jpg) 保留中文失败提示及原库列表；[check12-08-final-reader-restored-no-error.jpg](check12-08-final-reader-restored-no-error.jpg) 显示有效版本 6 的摘要/正文，旧失败提示已消失。主代理记录在同一独立 fixture 中暂移目录、读失败、还原并再次阅读，期间没有手动关闭消息 | 最后原生成功重试闭环通过。通知仅清除读取回调拥有的旧失败或该次跟随完成的捕获提示；4 项回归测试已纳入主代理完整 201 项桌面测试并通过 |

## 实际安装版本复核

补充独立查看 [installed-02-library.jpg](installed-02-library.jpg)、[installed-03-reader.jpg](installed-03-reader.jpg)、[installed-05-gallery-settled.jpg](installed-05-gallery-settled.jpg)、[installed-07-reader-anchor-settled.jpg](installed-07-reader-anchor-settled.jpg) 及对应 AX。`installation.json` 记录安装位置为 `/Users/aac6fef/Applications/course2md Design Preview.app`，bundle 版本 20260909.5、代码 9ec2f6ef89b923226fa178f722c7b6a07fa71423；安装后运行 PID 98163，安装二进制哈希与待安装文件一致。主代理说明该 release 未启用 performance。本代理核对截图、AX 与安装记录，未自行启动应用或重复执行安装检查。

| 所示安装状态 | 独立观察与判断 |
| --- | --- |
| 真实用户笔记库 [installed-02-library.jpg](installed-02-library.jpg) | 现有笔记的完整标题可读，版本、日期、40 段笔记、40 张截图各出现一次，metadata 与标题字重分层。缩略图、文字区、带标签的文件夹选择与阅读操作保持一条清楚的列表结构；搜索、筛选、分组、列表/卡片选择同高，无可见覆盖或挤压。 |
| 阅读正文 [installed-03-reader.jpg](installed-03-reader.jpg) | 返回在独立一行，完整标题、来源/时长、工具栏及正文共左轴；目录作为右侧导航，没有形成第三条漂移的文章轴线。长摘要和项目符号为普通正文，章节/时间标签更突出；图片使用自然比例，没有被额外卡片边框、底栏或留白框包住。工具组同高，“在文件夹中显示”已有清楚标签，AX 对应“在文件夹中显示这份笔记”。当前目录展开，摘要选中可见。 |
| 落定图库 [installed-05-gallery-settled.jpg](installed-05-gallery-settled.jpg) | 真实 40 图笔记稳定显示三列，首行三张卡片有可读宽度、相同图片高度及完整边界。三张摘要在本次窗口中均实际占三行，三行文字与下方操作没有重叠；“定位正文”和“观看”两项在各卡片中同高、跨列共基线。第二行从同一水平线开始；原先六列过密的问题没有复现。这补充的是本次安装窗口的三行证据，不能替代 200% 字号下恰好满三行的压力验证。 |
| 图库定位后阅读 [installed-07-reader-anchor-settled.jpg](installed-07-reader-anchor-settled.jpg) | 主代理记录从 00:27 卡片定位正文；落定图片中正文首段为 00:27，完整 16:9 图片和对应转录出现在该段下，右侧 00:27 有持久选中背景。标题、元信息和工具栏保持位置，图片左边界与正文共轴，截图与目录选中一致；AX 同时确认笔记视图及“目录：开”。 |

此次安装复核没有发现新的必须修改源码的排版问题，所示库、正文、三列图库及 00:27 定位落定状态通过。它使用的是用户原有的 40 段/40 图笔记，不是新执行 GUI 转换的产物。`installed-03` 可见正文从摘要第一段开始，“摘要”标题仅在 AX 中，因此该图只作为当前阅读构图证据，不用于证明首开绝对顶端；目录默认自动展开来自主代理运行步骤与前文代码/原生证据，静帧只能直接证明当前已展开。以上结论不扩展为全部 40 张图、所有滚动位置、所有字号主题或全产品全部状态的保证。

## 明确保留的限制

- 在 860×620、200% 字号的卡片库中，全局导航与标题/搜索/筛选工具区合计约占上方 420px，内容区约剩 200px。封面、标题和主操作需要分段滚动查看；这些图片证明可达与无重叠，并不证明此极限尺寸具有宽裕的浏览空间。列表提供更紧凑的选择。
- 卡片辅助动作行已用 check10-44 独立完成复核；check10-33 仍只作为列表模式证据。200% 下卡片封面、metadata、动作无法在约 200px 内容区同时显示，分段截图反映真实滚动状态，不等于整卡被缩小到同屏。
- 前文六图 fixture 由 CLI 使用相同导出管线及 similarity 0.999 生成，以便原生库/阅读/图库显示；这不能写成“通过 GUI 转换生成六张图”。安装复核另使用用户原有的真实 40 段/40 图笔记，补充实际内容的排版证据，同样不代表本次执行了生成流程。
- 本记录已覆盖正常浅色空库、已加载内容、图库、搜索、查看器、两种库展示、直接完成反馈、分类损坏/诊断/恢复、笔记读取失败及成功重试消除旧提示。其它加载/部分覆盖状态、损坏图片、缺失原视频、所有主题与所有字号组合没有全部独立覆盖。主代理应在最终全产品 matrix 中保留尚无证据的格子。
- 最新全宽标题与带结果查找的组合已由 [check07-05-narrow-search-title.jpg](check07-05-narrow-search-title.jpg) 完成原生复拍，标题、计数和命中首行完整同屏。860×620、200% 字号下，展开搜索后正文首屏约剩一行完整文字，后续正文仍需滚动；没有把这种可达性表述为宽裕的阅读空间。`check06-01` 实际是库页面，未用作阅读证据。
- [check06-14-direct-result.jpg](check06-14-direct-result.jpg) 顶沿仅保留全局导航的下部。它可以验证完成提示、返回、标题和阅读区的相互关系，不能用于确认完整全局导航的外边界；本代理未操作 GUI，未把这一截取限制推断为源码缺陷。
- 生成信息正常字号和 860×620、200% 均完成顶部/内部滚动到底复拍。窄窗每次约能显示一项信息，展开期间下方正文空间很少，固定“收起”用于恢复阅读空间；这不等于截断了信息或隐藏出口。check09-04/05 全局消息状态不同，不能据两张正常窗图声称整体逐像素不动；check10-35/37 没有该状态差异，信息区及页头位置一致。窄窗打开信息会收起查找，打开查找会收起信息，查询文字保留。
- check10-31 为中途滚动；check10-32 实际仍是列表（未成功切换卡片）；check10-36 没有滚进信息内部。这三张未用作相应状态验收。check10-34 另确认小窗阅读标题/返回/正文共轴，check10-38 仅确认单列图库图片区。

本代理的源码修改在 `desktop/src/reader_ui.rs`、获授权的 `desktop/src/course_library.rs`，以及获单独授权的 `desktop/src/main.rs` 读取回调/通知快照及其回归测试。最后 main-only 通知修复于 2026-09-09 08:51:43 UTC 冻结。所有改动经过 Rustfmt 解析/局部格式化及差异检查；最后四项通知回归测试从生产源码原样提取，以 rustc --test 独立执行通过，主代理随后将它们纳入完整桌面 201 项测试并全部通过。逻辑修复提交为 5740837，9ec2f6e 仅格式化本轮文件，其最终 release（未启用 performance）已完成成功重试原生复拍。主代理说明该轮未改配置或真实用户文件。本代理没有运行 Cargo 或 GUI，本文件不替代主代理的完整构建与全产品状态矩阵。
