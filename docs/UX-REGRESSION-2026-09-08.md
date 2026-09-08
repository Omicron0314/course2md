# 桌面回归修复 · 2026-09-08

用户指出上一轮设计有错位、越界和卡顿，随后要求完全取消“草稿”概念。旧验收只证明了部分页面能操作，过早写成“全部完成”。本页替代旧记录作为本轮修正的验收依据。

## 本轮修正

| 问题 | 修正后的行为 |
| --- | --- |
| 标题栏偏心、控件对不齐 | 清除标题栏组件隐含的左侧 padding，左右预留相同空间；按整个窗口计算导航中心。保存文件夹按钮的内部标签左对齐；导出选项的图标、勾选框、标题/说明分为三列。 |
| 动画导致内容越界或被裁掉 | 页面入场改为 160ms 透明度变化；展开内容从第一帧起使用正常布局和自然高度。移除位移与缓存高度裁剪，动画不再覆写调用方的宽度、padding。 |
| 窄窗与弹窗布局失真 | 笔记卡片按实际可用宽度和间距计算列数；阅读目录和正文分别约束滚动高度；已打开的图片弹窗随窗口缩小重新计算尺寸，长说明单独滚动；主题名称可换行，账号详情离开短状态徽标。 |
| 页面绘制重复探测磁盘 | 课程库访问、路径校验、恢复信息、标题别名与视频状态改为后台快照，并拒绝旧路径/旧请求结果。未变更的输入与任务镜像不再重复写盘。实际提交仍保留必要校验。 |
| 新建笔记要理解多个“草稿” | 工作台只有当前输入和“新建笔记”。切换页面保留当前输入，新建清空当前表单；已提交任务独立保存。旧版本多输入数据先完整备份再迁移，界面不再出现选择、命名、管理或丢弃草稿。 |
| 服务配置的草稿生命周期 | 普通编辑、保存和取消。未保存的编辑只在内存中，取消直接返回服务列表；保存一次即可生效，不再出现未完成记录或恢复编辑入口。 |
| 偶发画面呈现长停顿 | 增加可选逐帧诊断，将布局绘制与平台呈现分开测量；为 Metal 每帧增加独立 autorelease pool，及时释放有限池中的绘图对象。保留原有 GPU 提交顺序、异步缓冲回收和窗口唤醒机制。因果验证范围见下文。 |

设计 skill 现已明确：保留已有合理构图、从实际组件内部布局检查对齐、展开首帧必须具有正确高度、渲染方法禁止文件与网络访问，以及产品不再引入草稿管理。10 套配色、Material 图标和实际忙态反馈继续使用共享原语。

## 原生界面与流程

实测为 macOS 27 / Apple M3 Max 的优化 GPUI 构建，默认 100% 文字大小。主窗口 1140×820，最小窗口 860×620。图片是原生窗口截图，工具保存时会缩小像素尺寸；没有以 HTML 或源码推断替代运行验证。

- [工作台单一输入](design-review-2026-09-08/regression-evidence/19-single-form-workbench.jpg)、[新建后的空表单](design-review-2026-09-08/regression-evidence/20-new-note-empty.jpg)、[正在读取](design-review-2026-09-08/regression-evidence/21-reading-before-new.jpg)：返回工作台仍保留输入；读取一个受控慢来源时直接新建并读取另一来源，25 秒后旧结果没有覆盖新表单。
- [窄窗导出选项](design-review-2026-09-08/regression-evidence/13-export-aligned-narrow.jpg)：三列对齐，说明与标题同列，展开内容和后续选项边界完整。
- [普通窗口四列卡片](design-review-2026-09-08/regression-evidence/18-library-four-columns.jpg)、[窄窗三列卡片](design-review-2026-09-08/regression-evidence/11-library-narrow.jpg)：列数来自实际内容宽度。
- [长正文与章节跳转](design-review-2026-09-08/regression-evidence/07-reader-chapter-jump.jpg)、[弹窗打开后缩小窗口](design-review-2026-09-08/regression-evidence/09-image-dialog-after-window-shrink.jpg)、[长图片说明起始位置](design-review-2026-09-08/regression-evidence/10-image-caption-at-top.jpg)：正文、目录与说明各自保持边界。这张说明截图没有被当成“长图片标题”的独立视觉证据。
- [深色主题页](design-review-2026-09-08/regression-evidence/14-appearance-narrow.jpg)、[浅色主题页](design-review-2026-09-08/regression-evidence/15-appearance-light-narrow.jpg)、[存储](design-review-2026-09-08/regression-evidence/16-storage-narrow.jpg)、[展开的任务](design-review-2026-09-08/regression-evidence/17-tasks-narrow.jpg)：最小窗口下复查居中、内容边界和首屏间距。
- [服务编辑操作](design-review-2026-09-08/regression-evidence/23-service-edit-actions.jpg)、[保存后的服务](design-review-2026-09-08/regression-evidence/24-service-saved.jpg)：添加服务后取消没有留下记录；另一服务一次保存后出现在列表。本轮保存使用本地测试服务，没有声称完成外部服务联网验证。

工作台操作前后，测试工作区为一个当前输入、五个已提交任务；五个任务 plan 的整体 SHA-256 保持 `eb8273a30e2844e0cca5459198dcba5c35392336b62d61d6deefbe7d0d7b45e1`。迁移前的数据另存为 `workspace-single-input-upgrade-*`，不会为简化界面直接丢掉旧输入。

三个独立代理分别复审[导入与工作台](design-review-2026-09-08/import.md)、[笔记库与阅读](design-review-2026-09-08/library-reader.md)、[任务与设置](design-review-2026-09-08/tasks-settings.md)。各报告末尾记录新增缺陷、修正和实际截图范围。Metal 生命周期改动另经独立只读审查：借用对象不逃逸，命令缓冲持有 GPU 资源，完成回调仍在 GPU 完成后回收实例缓冲。

## 性能测量与限制

对照版本为 `e1c7afa` 与 `7d38083`，使用相同优化构建、同一套可选 GPUI profiler、相同五篇笔记、Tokyo Night 深色和窗口尺寸。每轮依次切换设置、任务、工作台、笔记库，再折叠、展开和上下滚动，六轮共 48 个操作。每步等待 300ms 后读取实际页面状态，确认动作生效。

| 实际操作区间 | 修复前 | 布局/缓存/输入修复后 |
| --- | ---: | ---: |
| GPUI 绘制 p95 | 12.62ms | 11.56ms |
| 首次失效至绘制结束 p95（含调度等待） | 40.63ms | 24.47ms |
| 首次失效至绘制结束最大值 | 126.42ms | 41.83ms |
| 平台呈现调用最大值 | 14.03ms | 7.48ms |

[完整对照数据和动作时间](design-review-2026-09-08/regression-evidence/performance-paired.json)包含逐帧数据及整个进程的累计直方图。表中只取记录的实际操作区间，不把工具截图、激活窗口和等待时间当作应用处理时间。

必须保留的异常：中间构建曾记录约 1017.6ms 的失效至呈现延迟；补充分段记录后，`7d38083` 在正式操作开始前的设置预热又出现一次约 1001.65ms 的平台呈现调用，那个画面的 GPUI 绘制只有 4.33ms。不能因为正式 48 次操作没有异常就抹掉这次停顿，也不能把它当成普通布局绘制太慢。

源码与本机 Apple SDK 检查发现：`nextDrawable` 返回 autoreleased 绘图对象，原 Metal 路径没有每帧独立释放池。`efdcc20` 增加该作用域，及时释放每帧临时对象。原代码显式关闭 drawable timeout，事务呈现路径还会等待调度，系统呈现也可能等待 WindowServer；因此“恰好约一秒”不能证明是默认超时，更不能仅凭代码审查认定唯一根因。

最终 `efdcc20` 再执行 48 个已核实的实际交互及 10 次切回前台，[整个进程的 496 帧记录](design-review-2026-09-08/regression-evidence/performance-final.json)没有超过 100ms 的绘制或失效至呈现样本；绘制 p95 为 10.16ms，输入至呈现 p95 为 29.46ms，最大 44.60ms。启动、预热与切回前台的样本均保留。这证明本次复测没有再出现长停顿，仍不能证明每一种用户环境下的偶发卡顿都已消除，或证明此前两次异常都由同一原因导致。

最初的空闲 CPU/主线程等待采样不是响应速度证据。另一次 45 秒原生物理导航 Time Profiler 的 [potential-hangs 导出](design-review-2026-09-08/regression-evidence/pre-pool-physical-hangs.xml)没有事件，阈值为 250ms；它发生在 Metal 释放改动之前，也不能单独用于宣称修复有效。输入延迟从 GPUI 收到事件开始，平台呈现计时覆盖 CPU 侧编码/提交，不包含完整 OS 输入队列或显示器扫描输出。

## 验证与提交

- 桌面完整测试：130 通过、0 失败、3 项原有忽略。新增覆盖单输入迁移备份、旧异步结果隔离、服务取消/保存边界、任务快照及镜像更新；真实 GPUI 布局测试覆盖展开首帧、宽度/内容变化和收起后的高度。
- 依赖补丁准备：6 项测试通过；`sources.py --locked` 连续应用成功。Metal 新补丁通过优化编译和上述原生交互复验，没有用不运行 Metal 的单元测试冒充呈现验证。
- 本轮普通优化构建保留 Apple 语音模块与 Metal 资源。一次编译等待来自 SwiftPM 下载大型预编译依赖；本机临时使用 `--disable-experimental-prebuilts` 构建已缓存源码，没有删除缓存、关闭 Apple 功能或把构建等待当成应用卡死。
- 代码分关注点提交：诊断 `56ae414` / `7d38083`、自然布局动效 `688b7cf`、页面/弹窗对齐 `6947862`、后台快照 `ffa56da`、单一输入 `94333da`、服务编辑 `909f979`、Metal 每帧释放 `efdcc20`。设计规则和证据另行提交。

本轮按用户要求没有扩展为专项可访问性审计。原有键盘行为和系统动态效果偏好保留。

## 本机交付

已更新并打开 `/Users/aac6fef/Applications/course2md Design Preview.app`。这是未启用 profiler 的普通优化构建，关于页确认[构建 `efdcc20d6eb7`](design-review-2026-09-08/regression-evidence/25-installed-corrected-about.jpg)，实际鼠标导航返回[修正后的工作台](design-review-2026-09-08/regression-evidence/26-installed-corrected-workbench.jpg)。Paper / Ink 跟随系统偏好已恢复，旧任务记录保持一致。

替换前的完整预览应用和配置位于 `~/Library/Application Support/course2md-design-preview/backups/2026-09-08-regression-fix/`。签名通过 `codesign --verify --deep --strict`，随包引擎 SHA-256 保持一致；[安装校验记录](design-review-2026-09-08/regression-evidence/installed-preview.json)与[证据文件清单](design-review-2026-09-08/regression-evidence/manifest.json)已归档。发布的 Alpha 1 与 Homebrew 版本不因这次本机 UI 修正而重打标签或替换资产。
