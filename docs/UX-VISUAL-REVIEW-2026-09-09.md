# 原生视觉与流程复核 · 2026-09-09

前几轮的问题来自设计方法和验收方法同时失效：先拼控件，再按截图逐项补尺寸；把编译成功、状态存在当作视觉成立。结果是普通文字与动作都蓝色、短内容被拉成整窗、按钮远离对应内容，窗口变矮或字号放大后便暴露裁切。用户指出的是完整产品的信息层级和操作负担；给每条批注单独加补丁无法解决它们。

同样的错误也影响流程：把读取、校对、摘要和补做各自的实现边界暴露为用户步骤。用户已经表达的转换意图应贯穿这些阶段。恢复既要保留原先的后续工作，也要区分“从未发出的请求”与“可能已处理的请求”，不能用一个重试按钮掩盖两者差别。

## 本轮使用的设计依据

项目继续统一采用 [Material 3](https://m3.material.io/) 的控件、状态和图标语言，窗口及任务组织参考 [Apple Layout](https://developer.apple.com/design/human-interface-guidelines/layout)、[Onboarding](https://developer.apple.com/design/human-interface-guidelines/onboarding) 和 [Sheets](https://developer.apple.com/design/human-interface-guidelines/sheets)。40px 控件、28/18/14/12px 字号及具体断点是项目适配值，不冒充官方规定。

[course2md-design skill](../.agents/skills/course2md-design/SKILL.md) 已补充任务面板、主题语义、实际绘制边界和独立复核规则。后续 UI 修改仍由 AGENTS.md 要求读取此 skill。具体方案需要说明它帮助用户做什么决定，不能把用户给出的示意箭头直接当成实现方案。

## 修正内容与证据

| 问题 | 修正与原生证据 |
| --- | --- |
| 引导松散，内容与下一步分离 | 面板保持统一上方起点，按内容自然增长；达到可用高度后正文滚动，操作留在同一面板。引擎以本机/在线路线及有取舍说明的模型卡片呈现。[引擎](visual-review-2026-09-09/native69-normal-engine-final.jpg)、[200% 重排](visual-review-2026-09-09/native63-narrow-build12-start.jpg) |
| 表单焦点截断、开关归属不清 | 输入与图标共享尺度，滚动区保留绘制空间；用途开关紧邻所属文字，焦点进入视野。[AI 检查](visual-review-2026-09-09/native71-ai-test-final.jpg)、[200% 焦点](visual-review-2026-09-09/native64-narrow-auth-reveal.jpg)、[服务保存操作](visual-review-2026-09-09/native95-editor-footer-reveal.jpg) |
| 模型 ID 只能手填 | 引导与设置共用模型发现：获取候选、手动填写、刷新及明确错误；列表可读不等于服务用途测试通过。网络请求有总截止时间，旧输入和已关闭窗口的结果不能回写。[模型菜单](visual-review-2026-09-09/44-service-model-menu.jpg)、[语音检查](visual-review-2026-09-09/native97-speech-check.jpg) |
| 登录入口缺失；大字二维码裁掉一半 | 引导增加可跳过的 Bilibili 步骤；登录复用真实 QR 会话，矮宽窗口将完整二维码与说明并排，取消固定可见。[修正后的 860×620 / 200%](visual-review-2026-09-09/native91-qr-860x620-200.jpg) |
| 模型状态堆成技术文字 | 模型身份、实际状态和操作集中；部分缓存、准备中和已加载分别显示。在线服务不要求本机模型。[真实加载中](visual-review-2026-09-09/native74-model-loading.jpg)、[加载结束](visual-review-2026-09-09/native78-model-load-complete.jpg)、[在线就绪](visual-review-2026-09-09/native98-online-ready.jpg) |
| Tokyo Day 普通文字几乎全蓝、选择太弱 | 从主题源色映射到正文/辅助/动作角色，共享选择态保留表面、轮廓及焦点层。十套配色逐一查看；浅深选择仍独立。[来源与适配](../desktop/assets/themes/SOURCES.md)、[150% 外观](visual-review-2026-09-09/native76-wide-150-appearance.jpg)、[主题弹窗](visual-review-2026-09-09/native77-wide-150-picker.jpg) |
| 阶段全绿但任务尚未完成，恢复动作被细节淹没 | 活动阶段单独显示，已完成历史默认收起；失败时先显示原因及恢复动作。补做的任务关联回工作台，避免新结果已生成还展示原错误。[不确定结果](visual-review-2026-09-09/native105-attention-final.jpg)、[已跟随结果](visual-review-2026-09-09/native100-recovery-result-final.jpg) |
| 阅读标题与工具占据正文 | 普通窗口压缩重复元信息；矮窗/大字号使用单行标题与紧凑工具栏，元信息按需展开，保留正文空间。[普通窗口](visual-review-2026-09-09/native104-reader-normal-final.jpg)、[860×620 / 200%](visual-review-2026-09-09/native99-reader-compact-final.jpg)、[实际查找命中](visual-review-2026-09-09/native103-reader-search-result.jpg) |
| 截图查看器叠住标题栏，转录和关闭操作被裁 | 使用标题栏下的实际可用高度，紧凑工具、完整比例的图片和固定操作区；短转录自然高度，长转录达到上限后显示独立滚动条。[短转录](visual-review-2026-09-09/native113-image-viewer-short-final.jpg)、[长转录](visual-review-2026-09-09/native114-image-viewer-long-final.jpg)、[实际滚动后](visual-review-2026-09-09/native115-image-transcript-scrolled.jpg) |

库的真空态、有内容列表/卡片、搜索无结果，任务的空态/运行/失败/完成，以及五个设置类别由分工代理独立审查。Sartre 负责引导、库、主题和设置整体；Lorentz 负责服务表单、请求边界与恢复关联；Ptolemy 负责账户、阅读及任务状态。根代理统一操作原生应用、修正并重拍。详细分批判断保留在[独立记录](visual-review-2026-09-09/independent-review.md)。

## 实际运行与检查范围

- 四次受控在线视频经真实 CLI worker 处理：从粘贴链接、点击开始，到生成文件、课程库和阅读页。普通转换不再要求用户逐步确认读取、字幕与生成内部阶段。
- 第二次转换的本地测试服务在摘要请求处断连，实际触发“结果未确认”。修复测试服务后，明确重发并自动补做成功；这次运行暴露并修复了工作台仍关联原错误任务的问题。断连属于测试服务，旧错误未更新属于产品缺陷，两者分别记录。
- 第三次故意在校对处断连，发现重试遗漏原计划中尚未发送的摘要。第四次原生复现使用修正后的续做计划，一次点击“重新发送以上内容”即完成校对和摘要，并直接进入阅读。[恢复结果](visual-review-2026-09-09/native111-proofread-and-summary-recovered.jpg)、[实际摘要](visual-review-2026-09-09/native112-recovered-note.jpg)。工作区测试核对精确授权与六种已有摘要请求状态；真实 HTTP 回归核对旧版本保留、两个组件完成及重复执行不增加请求。已有不确定的摘要请求不会被校对重试自动重新发送。
- 实际请求模型列表，检查 AI 校对、摘要和语音用途；模拟认证拒绝、慢响应及服务断连。凭据读取/DNS 的总等待边界、迟到结果、输入变更和错误关联由行为测试验证。
- Qwen3 0.6B / Apple 原生使用隔离的已有缓存重新加载，观察到准备进度及最终已验证加载。这不是无缓存下载、所有硬件后端或长视频准确率验证。
- Bilibili 使用真实匿名二维码生成、等待和取消；没有替用户扫码确认账号。语音及 AI 服务使用本机受控端点，不推断所有付费服务兼容。
- 原生 debug 窗口检查 1140×820、860×620、1800×1040，覆盖产品提供的 100/125/150/200% 字号代表场景及十套主题。没有把页面、主题与全部状态的笛卡尔积宣称为已检查。
- 截图查看器另使用短、长两种合成转录：实际点击放大、适合窗口、返回笔记及关闭，并滚动长转录。原 native108 的弹层裁切及 native110 对短转录的错误等分高度均是本轮发现并否决的中间结果，不作为最终效果。

检查与构建数字、安装位置和代码修订记录在 [delivery.json](visual-review-2026-09-09/delivery.json)。本轮常规桌面测试中三个既有外网/本地媒体测试维持显式 ignored；不能将它们算作通过。

最终交付：桌面测试 **186 通过、3 忽略**，真实任务协议回归 **11 通过**。以普通 Release 配置构建代码 `3654cafff794`，安装至 `~/Applications/course2md Design Preview.app`，验证应用签名和应用内构建号，并实际从设置重新进入引导。[安装后的版本页](visual-review-2026-09-09/native117-installed-about.jpg)、[当前打开的引导](visual-review-2026-09-09/native119-installed-onboarding-active.jpg)。旧应用和配置已备份；生成、服务配置和原输入/任务均保留。

## 动画、性能与证据质量

使用 `--release --features performance` 在原生窗口采集选择中间值、帧绘制和输入呈现数据。测量构建采用普通 1140×820 初始窗口；release 会忽略 debug 的窗口尺寸环境变量。两次采样分别使用 100% 和 200% 字号。见[普通字号测量](visual-review-2026-09-09/normal-performance-summary.json)与[大字号测量](visual-review-2026-09-09/narrow-performance-summary.json)。存在实际中间帧，动画最终收敛；测量中的 draw/input 没有超过 100ms 的样本。它们是这台机器和这些交互的有限样本，不代表重负载下永不卡顿。正式 Preview 不启用测量采集。

截图来自 Computer Use 对原生窗口的采集，部分被工具缩小。早期工具文件虽名为 PNG，实际字节是 JPEG，归档统一使用 `.jpg`，未重画、修图或伪造界面。截图的可见层级、状态、业务运行与测试结果分别作为证据；静态图片不能证明动画流畅或真实账号登录。早期文件名含 final/completed 的画面可能仍是被否决的迭代，以上述收口说明及交付记录为准。
