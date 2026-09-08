# 桌面设计改版验收 · 2026-09-08

本轮用户要求的设计规范、主页整改、可配置主题、Material 图标、状态动画及逐页独立审查均已完成。最终产品代码提交为 `e1fc3066406c`；文档提交在其后。历史报告按轮次保留发现和修复过程，早期“待复查”以各报告末尾及本页的最终结果为准。

## 交付结果

| 用户要求 | 实现 | 入口 / 证据 |
| --- | --- | --- |
| 项目设计规范与 skill | 采用 Material Design 3 的语义颜色、交互状态和动效原则，结合 macOS 原生窗口行为；明确 course2md 的间距、层级、输入、图标和主题规则。AGENTS.md 要求后续桌面工作先读 skill | [项目 skill](../../.agents/skills/course2md-design/SKILL.md)、[AGENTS.md](../../AGENTS.md) |
| 修正主页批注 | 单一集成标题栏导航；移除重复品牌；来源类型位于输入区外；输入有固定标签和边界；语言显示“自动”；使用真实平台 logo；最近笔记有明确阅读动作，徽标图标与文字在同一容器内 | [最终窄窗主页](evidence/47-new-workbench-narrow-dark.jpg)、[已安装优化版](evidence/54-installed-preview-workbench.jpg) |
| 深浅色配色主题 | Paper、Ink、Nord Snow、Nord、Tokyo Day、Tokyo Night、Catppuccin Latte、Frappé、Macchiato、Mocha，共 10 套；支持跟随系统与独立明暗偏好，立即应用并持久保存 | [外观设置](evidence/33-appearance-final-dark.jpg)、[窄窗深色](evidence/37-appearance-narrow-dark.jpg)、[窄窗浅色](evidence/38-appearance-narrow-light.jpg) |
| Material Icons | 内嵌 79 个 Google Material Icons Rounded SVG，覆盖导航、表单、操作、状态及组件图标；另有 YouTube / Bilibili 品牌资源，附来源和许可 | [图标资源](../../desktop/assets/material/README.md)、[配色来源](../../desktop/assets/themes/SOURCES.md) |
| 流畅状态反馈 | 页面和结果入场、选项展开 240ms；选择和实际进度变化 200ms；主题颜色 280ms；读取、打开、测试、下载、导出、迁移等真实忙态有循环指示。减少动态效果保持原行为 | [共享动效](../../desktop/src/motion.rs)、[真实读取中](evidence/49-reading-narrow-dark.jpg)、[真实生成中](evidence/23-task-running-light.jpg) |
| 每页 subagent 审查 | 三个独立审查代理覆盖工作台/来源/计划、笔记库/阅读/图片，以及任务/全部设置子页/账号/存储/诊断/关于，并复看原生截图后修复发现 | [导入审查](import.md)、[笔记与阅读审查](library-reader.md)、[任务与设置审查](tasks-settings.md) |

## 最终原生验证

设备为 Apple M3 Max、macOS 27。真实 GPUI 应用在普通窗口和 860×620 最小窗口操作；截图由原生窗口采集，归档为实际 JPEG 格式。蓝色、红色封面来自合成教学视频，不是产品封面素材。

- 10 套主题逐一点击并核对选中态；界面、表单、菜单及阅读状态使用语义颜色。明暗设置分别保存为 Catppuccin Latte / Tokyo Night，退出 design03、启动 design04 后保持；design05 再切浅色确认 Latte 未被深色选择覆盖。
- 新输入、来源确认、导出选项、生成计划、任务运行/完成、内部正文、查找高亮和图片查看通过实际操作。合成来源通过真实 CLI、ffmpeg、内部发布及 Markdown 导出生成了“主题与动画验收”健康笔记。
- 最小窗口的库自动三列，外观页双列；[正文图片居中](evidence/41-reader-narrow-light.jpg)，[图片说明与底部动作](evidence/42-image-narrow-light.jpg)完整可见；实际缩放 91%→114%→适合窗口 91%，关闭后返回同一篇笔记。[深色图片查看器](evidence/44-image-narrow-dark.jpg)同样通过。
- [任务完成页](evidence/45-tasks-narrow-dark.jpg)只有一个页标题，普通完成卡保留结果和下一步，已完成步骤收进技术详情。进入选中任务后标题栏未读标记消失。
- [本地输入空态](evidence/48-local-input-narrow-dark.jpg)、[导出格式图标](evidence/51-export-options-narrow-dark.jpg)和[窄窗生成计划](evidence/52-plan-narrow-dark.jpg)完成最终复查；计划返回按钮完整可见。
- 25 秒受控读取期间，输入与取消入口保持可见。打开另一应用遮挡验证窗口后，无额外输入即可更新到[已确认来源](evidence/50-source-ready-narrow-dark.jpg)，来源详情保持默认收起。
- 已安装预览的原生窗口缩放按钮可放大并恢复普通窗口，[放大](evidence/56-preview-window-zoom.jpg)和[恢复](evidence/57-preview-window-restored.jpg)后标题栏和设置内容正常，随后返回工作台。
- 设置中的生成选项、服务编辑/测试、账号二维码、存储、应用和诊断均有本轮原生观察，范围与各轮截图限制详见分组报告。二维码仅验证匿名获取和取消，没有模拟账号登录成功。

动画验收发现并修复两个 macOS 问题：同步 AppKit 重绘会在加载动画中形成绘制循环；单纯丢弃绘制中的请求又会让被遮挡窗口停在旧画面。最终使用每窗口合并的 16ms 延迟唤醒，保留暂时无法处理的需求；启动、遮挡后的异步结果及连续交互已复验。

组件浮层现在会在鼠标或快捷键导航前关闭已显示及等待显示的提示。5 项组件测试通过，最后各次页面导航没有残留；本轮未用自动操作稳定重现“纯悬停后移除触发器”的原生条件，不把截图当成该条件的独立通过证明。

## 自动验证

| 检查 | 结果 |
| --- | --- |
| 桌面完整测试 | 115 通过，0 失败，3 项原有忽略 |
| macOS 延迟重绘测试 | 3 通过：合并及延迟、不可用回调重试、重入需求保留 |
| 组件 tooltip 生命周期测试 | 5 通过，包括鼠标导航、已处理快捷键、延迟请求取消和窗口隔离 |
| 源码补丁准备脚本测试 | 6 通过；`sources.py --locked` 重复准备成功 |
| 打包脚本测试 | 3 通过 |
| 构建 | debug 与 optimized release 均成功；预览包签名验证成功 |
| 静态检查 | 本轮修改文件 rustfmt、Git diff 空白检查、79 SVG 解析、81 内嵌资源路径及 skill 格式验证通过 |
| 原子提交中间状态 | 语义主题基础、阅读/任务、设置阶段的归档源码分别 `cargo check` 通过；只检查桌面 Rust 接口，关闭额外 Apple 模块编译 |

本轮没有改动转换引擎的处理逻辑，旧验收中的网络故障、数据迁移和完整后端组合不重复冒充新一轮实机验证。原生运行与构建在 macOS 完成，没有声称本轮已在 Windows / Linux 实机测试。编译仍有上游弃用/未来兼容提示及 8 项已有桌面未使用代码警告。页面专项可访问性审计按用户要求不在本轮范围。

## 本机预览与提交

已安装独立、优化构建的 `~/Applications/course2md Design Preview.app`，关于页确认[构建 e1fc3066406c](evidence/55-installed-preview-about.jpg)。随包引擎为已发布的 `2.0.0-alpha.1 (2f08a2f2865f)`；这次没有变更引擎源码。预览使用独立配置 `~/Library/Application Support/course2md-design-preview`，默认输出 `~/Documents/course2md-preview`，不包含合成源拦截器。

这是本机设计预览，不是重新发布 Alpha 1。现有 GitHub tag、发布资产和 Homebrew 安装保持原状。

代码按关注点提交：设计 skill `a78069d`、图标及许可 `5d4a962`、动画绘制修复 `baaa0ec` / `e1fc306`、浮层生命周期 `3f4db7d`、主题配置 `7a6a9b5`、语义颜色与动效 `0943889`、笔记/阅读 `15cf9f8`、任务 `f2a0f6b`、设置 `bf06af8`、标题栏与工作台 `76665f9`。最终审查记录与截图单独提交。
