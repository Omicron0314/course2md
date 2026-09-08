# 设计体系重置与交付复盘

2026-09-09。安装构建：`238802797b49`，`2.0.0-alpha.1+design.238802797b49`。

用户批评的核心是缺少整体设计判断。我把批注当成逐项修改指令，没有先判断整页要让人完成什么、信息如何分组、哪个动作值得突出。修掉一处后又补一条局部规则，页面之间逐渐形成不同的排版和控件习惯。这些判断本应由实现者完成，不该依靠用户逐处指出。

## 为什么之前的设计反复失败

**只有样式要求，没有组件职责。** “用胶囊”“加图标”“加动画”不能构成设计体系。导航、互斥选择、主要动作、媒体和内容容器各有用途。此前让它们互相借用尺寸和边框，导致页面处处强调，却看不出哪里值得注意。

**逐条照改，没有整体取舍。** 我追求箭头对应的局部变化，忽略了控件与整页的关系。旧 skill 一面要求共同对齐，一面保留独立的 520px 子列；一面要求复用，一面允许组件默认值和页面覆盖决定最终尺寸。让规则越来越多，没有让设计更统一。用户的批注应当是问题证据，实现者仍需要判断解决方案。

**没有审查组件内部和完整状态。** 调用同一个 helper，不保证内部 padding、实际字号和媒体比例正确。首次空页、已有内容、展开、读取中、失败与完成，都可能改变视觉重心。过去只围绕修改清单看终态，又把“构建成功、没有越界、测试通过”扩大成设计验收；这使明显的排版问题继续交给用户发现。

## 选定的指南，以及项目里的具体取舍

采用 [Material Design 3](https://m3.material.io/) 作为共同的视觉与交互语言，遵循其[互斥选择](https://m3.material.io/components/segmented-buttons/guidelines)与[状态层](https://m3.material.io/foundations/interaction/states/overview)原则；窗口、标题栏、菜单和键盘行为参考 Apple 的[设计原则](https://developer.apple.com/design/human-interface-guidelines/design-principles)、[布局](https://developer.apple.com/design/human-interface-guidelines/layout)与[动效](https://developer.apple.com/design/human-interface-guidelines/motion)。Material Icons 与项目现有配色系统也与这一方向一致。

这些来源用于约束判断，不用于给任意数值背书。40px 默认控件高度、28/18/14/12px 文字角色、随字号缩放的约 360px 控件槽是本产品的桌面适配，不是官方统一规定，也不声称实现了完整的 M3 Expressive。

规则已落入 [course2md-design skill](../.agents/skills/course2md-design/SKILL.md)、[共享设计体系](../.agents/skills/course2md-design/references/system.md)和[设置布局约定](../.agents/skills/course2md-design/references/settings.md)。项目 AGENTS.md 要求后续桌面设计先读取该 skill。关键取舍是：

- 先确定页面任务、信息层级和共同对齐，再选择组件。标题、分组与留白服务阅读顺序，不再给每节叠加标题、分隔线和大卡片。
- 选择使用统一的低强调胶囊；强填色留给当前主要动作。选中、悬停、按下和键盘焦点叠加，不互相抹掉。媒体与多行内容采用自然高度，不能继承普通按钮的固定高。
- 表单处于共同网格。控件保持适合阅读和操作的宽度，必要时移到标签下方；不因为外层很宽就把选择项拉散。宽窗设置用侧栏，窄窗用类别条，按真实可用宽度重排。
- 工作台是持续导入流程。先读视频，成功后显示摘要及开始生成；普通偏好按需展开。字幕正在读取、未确认和失败属于必须处理的信息，直接显示取消和恢复入口。没有新建、命名或管理空草稿的步骤；已生成笔记仍作为阅读结果保留。
- 每个意图有一个主要动态反馈。选择指示连续移动、配色平滑变化、后台工作显示实际进度；去掉整页和类别切换的叠加淡入。选择标签不塞入会使轨道变宽的过程文案。
- 外观页保留稳定的浅色、深色两个预设入口，分别选择四套和六套实际配色。修改另一种模式的预设不切换当前模式；保存失败以重试为主动作。

## 本轮独立审查发现的额外问题

三个 subagent 从原始拒绝图、完整原生画面和源码分别审查，没有只照着我的修复清单确认。

| 发现 | 修正与证据 |
| --- | --- |
| 纯图标按钮继承文字 padding，图标被挤小或消失 | 文字 padding 留在带文字的变体；[库页](design-review-2026-09-09/70-narrow-library-200-fixed.png)与[阅读页](design-review-2026-09-09/93-final-generated-reader.png)实际显示对应图标 |
| Reader 媒体继承 40px 高度，被压成横条 | 图片与媒体按钮明确使用自然高度；[正文](design-review-2026-09-09/35-current-reader.png)、[截图](design-review-2026-09-09/36-current-reader-images.png)、[放大查看](design-review-2026-09-09/37-current-image-viewer.png)均重新观察 |
| 模型标签测量和绘制取整不一致，差不到 1px 也会截字 | 按实际字体测量取整，纳入固定边框和图标空间；增加 Qwen 两个标签切换后的 GPUI 几何回归 |
| 窄窗大字号顶栏截字，刷新独占工具栏一行 | 导航根据实际宽度使用完整短标签；整库刷新位于标题行右端；[860 窗口、200%](design-review-2026-09-09/70-narrow-library-200-fixed.png)补拍确认 |
| 任务卡多行标题仍继承按钮高；部分完成提醒把成功项目也列成补做 | 标题采用自然高度；补做只读取 Failed/Partial，混合成功、失败及未请求结果有独立测试；[完成任务](design-review-2026-09-09/74-narrow-task-natural-height-200.png)复查 |
| 精简导入时隐藏了字幕读取和恢复入口 | 必要字幕状态脱离普通偏好；[读取中](design-review-2026-09-09/65-subtitle-reading-visible.png)、[取消后](design-review-2026-09-09/66-subtitle-cancelled.png)、[失败](design-review-2026-09-09/63-subtitle-wait-visible.png)、[恢复](design-review-2026-09-09/68-subtitle-read-recovered.png)实际走查 |
| 空名称提交只聚焦隐藏字段；手动滚走后重复提交不再定位 | 展开对应选项并按每次错误提交触发可见定位；[首次](design-review-2026-09-09/89-final-title-error-first.png)与[重复提交](design-review-2026-09-09/90-final-title-error-repeat.png)均看到完整输入框与焦点 |
| 服务单选协议没有实际选择；空字段缺少填写线索；失败时完成压过重试 | AI 服务移除单选项协议块，补地址与模型格式提示；[服务表单](design-review-2026-09-09/80-final-service-form.png)、[保存失败](design-review-2026-09-09/84-final-theme-save-failure.png)、[恢复](design-review-2026-09-09/85-final-theme-save-retry.png)补拍 |

完整独立记录：[设置与配色](design-review-2026-09-09/independent-final-design-review.md)、[库／阅读器／任务](design-review-2026-09-09/independent-library-reader-task-review.md)、[连续导入](design-review-2026-09-09/final-import-review.md)。各自保留中间发现和未覆盖状态，后续补审以附录记录闭合，不能把中间失败截图重新称为最终通过。

## 实际验证与交付

已观察正常、860 窄窗和 1800 宽窗，100/125/150/200% 字号及十套配色。图像编号对应实际构建，见 [验证清单](design-review-2026-09-09/verification.json)。固定窗口初始尺寸使用同源调试构建，性能测量使用优化构建，最终安装使用关闭性能记录功能的普通优化构建。

- 设置各分类重新观察：[宽窗外观](design-review-2026-09-09/86-final-wide-settings.png)、[生成选项](design-review-2026-09-09/88-final-recognition-options.png)、[服务底部动作](design-review-2026-09-09/81-final-service-form-actions.png)、[存储](design-review-2026-09-09/79-final-storage.png)、[关于](design-review-2026-09-09/75-final-optimized-application.png)、[设备与模型](design-review-2026-09-09/77-final-model-device-details.png)、[长缓存路径](design-review-2026-09-09/78-final-cache-details.png)。
- 主题实际逐一切换。隔离配置暂设不可写，失败保留旧选择，恢复权限并重试后检查磁盘；另检查独立深色预设、跟随系统和重启后的保留值。
- 隔离课程实际走过读取、12 秒字幕慢读、取消、重新读取、切页、名称校验、生成及阅读。只有提取器网络响应和视频素材是合成 fixture，字幕解析、任务执行、ffmpeg、存储和阅读使用真实实现。最终 [Final design validation 已生成](design-review-2026-09-09/92-final-task-generation.png)，[同名阅读结果](design-review-2026-09-09/93-final-generated-reader.png)有正文和图片、无缺失警告。
- 原生回归测试 138 项通过、0 失败、3 忽略，包括选择控件几何、字幕必要状态和混合任务结果。测试不能证明审美质量。最终错误提交和输入恢复另有原生操作记录。
- 优化构建的本轮采样包含 55 个实际动画中间值；725 次绘制的 p95 为 5.76ms，20 次输入到画面更新的 p95 为 31.28ms，没有超过 100ms 的记录。空闲样本不再持续绘制。详见 [采样摘要](design-review-2026-09-09/optimized-motion-summary.json)。这只是本机本轮切换与设置操作，不是所有负载下的性能保证，也不能从静态截图证明动画观感。

已安装 `~/Applications/course2md Design Preview.app`，关于页确认[构建 238802797b49](design-review-2026-09-09/95-installed-build.png)。旧应用备份至 `~/Library/Application Support/course2md-design-preview/backups/2026-09-09-020749/`。代码签名验证通过；应用偏好文件安装前后 SHA-256 完全一致，保留[深色 Ink、125% 字号](design-review-2026-09-09/94-installed-preferences.png)。普通安装版短时空闲采样稳定到约 0.4% CPU、114MiB RSS。应用留在[工作台](design-review-2026-09-09/96-installed-workbench.png)，[空库](design-review-2026-09-09/97-installed-empty-library.png)仅显示导入动作。

本轮没有重新发布预发行版、变更 Homebrew 或替换已发布资产。没有重新验证所有真实平台网络、外部付费 AI 服务、账号登录及不同硬件的语音识别；本报告不把上述 UI 与隔离生成验证扩大成这些业务能力全部通过。

源码按关注点提交：设计规范 `1bb7e57`，任务补做准确性 `5165e84`，共享控件与页面结构 `0382e6f`，导入状态与错误定位 `2388027`。本复盘与原生证据单独提交。今后的交付应报告具体判断、发现及证据边界，不再用“设计已验收”替用户作接受决定。
