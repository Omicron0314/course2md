# 原生构图复盘与证据记录 · 2026-09-09

> **后续纠正：** 用户在 17:22–17:29 的实际使用截图中指出了本报告漏判的分组、视觉轻重、操作关联和信息价值问题。下文“没有新的必须修问题”等设计关闭结论已撤回；原始截图、测试和安装记录保留为历史证据。原因见[审查失效复盘](UX-REVIEW-FAILURE-2026-09-09.md)。这些新发现尚未因这份复盘而修复。

本轮把工作台、转换状态、阅读与图库、笔记库、设置及四步引导重新按共同轴线和完整任务组织，并由三个独立审查者查看真实原生截图。已复拍关闭的问题有具体前后图；未实际操作的状态和未拍到的页面组合单列为验证边界；后续修复均对应新的构建与复拍记录。本报告不作“全页面／全状态已验收”的结论，也不沿用旧轮次的通过判断。

原始图片、三份独立记录、测试日志及性能摘要位于[本轮证据目录](design-review-2026-09-09/composition-evidence/README.md)，逐文件尺寸与 SHA-256 见 [manifest.json](design-review-2026-09-09/composition-evidence/manifest.json)。仓库同级旧 PNG 归档属于此前轮次；本报告没有借用它们补本轮未验证的格子。

## 此前判断为何失效

1. **没有先确定共同轴线。** 标题、图标、说明、控件和状态分别拥有自己的内边距；短开关被推到整个内容区右侧，卡片正文回到图标外缘。各个局部都能放下，组合后却没有连续的阅读起点。缺少的是标签列、文本列、控件列和顶部锚点的关系，继续修改单处间距不能建立这些关系。
2. **按截图局部修补，没有按状态和完整任务组织画面。** 字号、图标或提示框只解决当前静帧；展开详情、切换认证、取模型和返回检查结果会改变高度，使整张引导上下移动。来源、准备、执行和打开笔记同属一条任务；把计划页、完成页分别排整齐不能消除多余中断。
3. **把编译和无溢出误当成设计验收。** 编译只证明构建成立；无裁切只证明某一画面能容纳内容。两者均不能证明标签与操作容易配对、层级成立、解释不重复、选择和进度语义正确。此前结论超出了证据。
4. **没有在真实分辨率和动态内容下充分独立看图。** 先给出修复清单、再挑几张图确认，容易重复实现者的判断。不同窗口、文字比例、长内容以及加载／失败／完成前后的连续画面没有形成充分覆盖。零散静帧也不能证明滚动条显隐、焦点、动画中间过程和返回路径。

这次检查先看整屏阅读顺序和分组，再核对共同轴线、实际文字／图标边界与状态变化。每个结论注明证据种类：**源码**表示实现意图；**测试日志**表示所列断言已执行；**静帧**表示拍摄时可见的构图；**主代理实操记录**表示实际执行过的路径。它们相互补充，不能互相替代。独立审查者没有操作 GUI 或运行 Cargo。

## 原生环境与归档边界

以下逻辑尺寸来自本轮 fixture 配置及操作者记录；图片尺寸直接读取原文件，不能由后者反推窗口逻辑尺寸或显示器比例。

| 配置 | 请求的逻辑窗口 | 原始 JPEG 尺寸 | 应用文字比例／外观 | 证据用途 |
| --- | --- | --- | --- | --- |
| normal | 1140×820 | 1068×768 | 100%，浅色 Tokyo Day | 正常页面、设置、四步引导及完整任务 |
| narrow | 860×620 | 860×620 | 200%，浅色 Tokyo Day | 工作台、库、阅读、查找、图库及查看器重排 |
| wide | 请求 1800×1040 | 1330×768 | 125%，深色 Catppuccin Mocha | 宽页面、目录、图库、设置；check03-34 另切 150% |
| empty | 继承 normal 窗口 | 1068×768 | 100%，浅色 Paper | 独立空配置下的工作台、库与任务 |

十张 `palette-*` 是实际切换全部十种配色后的原生画面；check08-08／09 是 normal release fixture 的深色切换。原始拒绝图和 `before-*` 不因像素尺寸相同而被推定使用这些 fixture 偏好。多截图阅读素材由 CLI 使用同一导出管线和 similarity 0.999 生成，用于原生阅读六张截图；不记为“GUI 转换生成六张图”。

| 截图组 | 操作者报告的构建组 | 使用方式 |
| --- | --- | --- |
| 用户四张拒绝 PNG、before | 用户原始拒绝图；before 为 Design Preview，CFBundleVersion `20260909.4`、code `3654cafff794` | 失败基线，不能作为最后结果 |
| after | build02，debug + performance | 第一批改动，包含随后发现的失败 |
| final-01、check03、palette | build03，debug + performance | `final-01` 实际仍有重复来源／操作；文件名中的 final 不代表验收 |
| check04 | build04，debug + performance | 设置等局部复拍；卡片首行和进度位置仍在随后修复 |
| check06 | build06，debug + performance | 正常任务、后台完成、空态与部分失败；AI 进度／暂停语义仍含已知问题 |
| check07 | build07，debug + performance | 卡片、模型进度锚点与窄阅读标题的针对性复拍 |
| check08 | release + performance，PID 43350 | AI 等待／暂停／部分恢复及阅读反馈；每张是否最终落定按下文区分 |
| check09-01～08 | `6c2e997`，release、不含 performance | 中间复拍；04／05 信息区正常字号已闭环，01～03 与 06／07 又发现状态／错误文案问题 |
| check10-01～03 | `6c2e997`，release、不含 performance | **实际正常 1140×820／200%，JPEG 1068×768**，不能按文件名充当 860×620 |
| check10-04～51 | `e217d0d` 源码，debug、不含 performance | 用仅 debug 启用的窗口覆盖机制设置真正 860×620／200%；与 release 正常／性能证据分开 |
| check11-01～12 | `e217d0d`，release、不含 performance | 正常／故障复拍；06 是旧失败提示残留的新发现，07 已就绪而非准备中 |
| check12-01～05 | `5740837`，debug、不含 performance | 共享开关在真实 860×620／200% 下的复拍、鼠标／空格操作和 off 落定端点 |
| check12-06～12（无 09） | `9ec2f6e`，release、不含 performance | 读取故障→恢复无旧提示，以及本地 401 的部分结果／最近笔记去重复拍 |
| installed | `9ec2f6e`，release、不含 performance，Design Preview `20260909.5` | 已实际安装并查看 About、真实用户库和 reader；身份记录见报告末尾 |

最终 release 不启用 `COURSE2MD_VALIDATION_WINDOW`，最初三张 check10 虽名为 narrow，实际仍是正常窗口。原生边缘拖动尝试未改变这些图，不能仅由此断言生产 resize 有缺陷。后续使用该轮源码的 debug 原生渲染路径指定精确小窗，具体修订按上表区分，不往 production 添加验证后门。

构建组来自操作者说明，不能仅靠文件名独立验证二进制身份。最终代码、二进制哈希与安装身份由主代理在报告末尾补充；不把较早截图追溯归属于后来二进制。

归档按原编码字节复制，不缩放、裁切、重压缩或修图。3 张小型文件选择器调试图保留在临时目录，在 manifest 中记录 SHA-256、尺寸和排除原因；其余本轮顶层 JPEG 与四张原始拒绝 PNG 保留审查历史。精确数量以 manifest 为准。

有五处文件名容易造成错误结论：check03-18 实际为 **QR 已就绪**；check04-05 实际仍在 **图库**；check06-01 实际为 **笔记库**；check06-18 实际仍在 **摘要处理中**；check08-07 实际为 **阅读入场中间帧**。它们分别不能充当初始二维码加载、查看器、阅读标题、已结束失败或落定阅读的证据。check04-03 是空查询，也不是“搜索零结果”。这些更正在 manifest 和独立记录中保留。

## 具体问题及关闭依据

第一批六项来自独立查看 before-09～13、after-02～12，之后继续在新图中找未关闭问题。下表的“图证闭环”只关闭所列画面中的问题。

| 编号 | 原问题与原图 | 修改关系 | 最新证据与状态 |
| --- | --- | --- | --- |
| C01 引导顶部锚点随状态移动 | [after-04](design-review-2026-09-09/composition-evidence/after-04-guide-engine.jpg) 与 [after-05](design-review-2026-09-09/composition-evidence/after-05-guide-engines.jpg) 顶部约 y163→76；AI 取模型／检查也整体移动 | 固定面板顶部、步骤和主标题，长表单在正文中滚动 | [check04-15／16](design-review-2026-09-09/composition-evidence/check04-16-guide-ai-result.jpg) 的步骤／标题保持相同起点；[check07-01／02](design-review-2026-09-09/composition-evidence/check07-02-engine-capabilities.jpg) 展开能力时顶部仍固定。**相应静帧闭环**；全部焦点／长表单组合未独立实操 |
| C02 短控件脱离标签与 About 身份组 | [外观](design-review-2026-09-09/composition-evidence/after-02-appearance.jpg)、[应用](design-review-2026-09-09/composition-evidence/after-03-application.jpg) 短按钮／开关贴最右，About 操作脱离名称 | 设置共享有界控件槽，短控件靠列首；About 身份与操作共文本轴 | [check03-31](design-review-2026-09-09/composition-evidence/check03-31-wide-appearance.jpg)、[check04-12／13](design-review-2026-09-09/composition-evidence/check04-13-about-idle.jpg)。**正常／宽外观及正常 About 图证闭环**，真实窄 200% 的对应组已补图，开关比例问题另列 S10-01 |
| C03／R04-01 选择卡首行不共轴 | [after-05](design-review-2026-09-09/composition-evidence/after-05-guide-engines.jpg) 说明回到图标外缘、状态居中；[check04-14](design-review-2026-09-09/composition-evidence/check04-14-guide-engine.jpg) 等高卡里长段落仍使标题上移 | 图标槽、标题和说明文本共轴；内部改顶部对齐 | [check07-01](design-review-2026-09-09/composition-evidence/check07-01-engine.jpg) 三张模型标题及说明首行同轴，[check07-02](design-review-2026-09-09/composition-evidence/check07-02-engine-capabilities.jpg) 能力卡标题／说明／状态各共轴。**图证闭环** |
| C04 同一关系重复包容器 | [账号](design-review-2026-09-09/composition-evidence/after-11-guide-account.jpg)、[模型](design-review-2026-09-09/composition-evidence/after-12-guide-model.jpg) 重复标题与多层帮助面 | 一个状态组及就近解释，普通正文不逐句加框 | [check04-17](design-review-2026-09-09/composition-evidence/check04-17-guide-account.jpg)、[check07-03／04](design-review-2026-09-09/composition-evidence/check07-04-model-progress-fixed.jpg)。**所列账号／准备状态图证闭环** |
| C05 检查结果和操作不连贯 | [after-09](design-review-2026-09-09/composition-evidence/after-09-ai-checking.jpg) 重复忙碌／停止行；[after-10](design-review-2026-09-09/composition-evidence/after-10-ai-check-state.jpg) 结果措辞技术化且底部操作裁切 | 状态与停止同排；显示能力“可用”，检查后正文滚到结果 | [check03-11／12](design-review-2026-09-09/composition-evidence/check03-12-ai-running.jpg)、[check04-16](design-review-2026-09-09/composition-evidence/check04-16-guide-ai-result.jpg) 中错误／处理中／成功各有对应动作。**静帧中措辞与可见性闭环**，不据此证明停止和重试时序 |
| C06 忙碌与焦点外观不一致 | [after-09](design-review-2026-09-09/composition-evidence/after-09-ai-checking.jpg) 用途开关一亮一淡、焦点边界含义不明 | 统一忙碌时受影响控件和根焦点 | [check03-12](design-review-2026-09-09/composition-evidence/check03-12-ai-running.jpg) 外观一致。**视觉差异已收敛，鼠标／键盘能否修改正在检查的值未由本审查者实操** |
| S01 设置节间距／AI 控件列／健康存储空隙 | [check03-04](design-review-2026-09-09/composition-evidence/check03-04-generation.jpg) 节间距压缩，服务值脱离标签；健康存储仍占空状态槽 | 节与固定控件不参与不当 flex 收缩；AI picker 接共享标签列；无状态内容时不布局空槽；解释贴首组 | [check04-09／10](design-review-2026-09-09/composition-evidence/check04-10-generation-lower.jpg)、[check04-11](design-review-2026-09-09/composition-evidence/check04-11-storage.jpg)。**正常配置图证闭环** |
| S02 服务编辑用途缺语义标题 | [check03-36](design-review-2026-09-09/composition-evidence/check03-36-service-editor.jpg) 测试用途像服务启用范围 | “检查服务”将用途／说明／结果关联；认证及用途使用共享有界单选轨道，不再加外层面板 | [check04-24／25](design-review-2026-09-09/composition-evidence/check04-25-service-editor-actions.jpg) 中测试与保存生效说明清楚，取消／重测／保存可见。**正常成功态图证闭环** |
| R04-02 后台后果贴错动作 | [check04-18／19](design-review-2026-09-09/composition-evidence/check04-19-model-phase.jpg) 后台说明在上一步下方，实际关联继续使用 | 后果紧贴继续使用下方、右对齐 | [check07-03／04](design-review-2026-09-09/composition-evidence/check07-04-model-progress-fixed.jpg)。**两状态图证闭环** |
| R04-03 第一条进度挤动动作 | [check04-18](design-review-2026-09-09/composition-evidence/check04-18-model-start.jpg)→[19](design-review-2026-09-09/composition-evidence/check04-19-model-phase.jpg) 插入阶段／数量使暂停和继续下移约 66px | 预留进度区，等待文字替换为真实阶段／数量 | [check07-03](design-review-2026-09-09/composition-evidence/check07-03-model-start-fixed.jpg)→[04](design-review-2026-09-09/composition-evidence/check07-04-model-progress-fixed.jpg) 暂停中心约 y403、分隔 y452、继续中心 y485 不变。**两端几何闭环**；未声称复现或排除丢失点击 |
| F02／F03 普通计划／完成中间页 | [原计划](design-review-2026-09-09/composition-evidence/codex-clipboard-4f9b8add-ec3e-4695-a19e-f6d1f83ab556.png)、[原完成](design-review-2026-09-09/composition-evidence/codex-clipboard-3988cc1c-dc49-4426-a124-e14bb2877f6a.png) 再次要求无必要确认 | 首次开始保存转换意图，内部准备自动续接；仍跟随当前任务时直接打开可读笔记 | 转换意图／完成归属测试已执行；操作者报告本地“生成新版”直达 reader，[check06-14](design-review-2026-09-09/composition-evidence/check06-14-direct-result.jpg) 为结果图。离开工作台后[完成仍在设置](design-review-2026-09-09/composition-evidence/check06-12-background-completed-settings.jpg)，点击通知才打开。**这些实操路径有记录**，链接成功全流程和异步竞态尚未全部实操 |
| F08／F09／F10 来源、详情与恢复输入重复 | [final-01](design-review-2026-09-09/composition-evidence/final-01-workbench.jpg) 同一来源两套身份／开始；旧任务事实值远离标签、操作在长历史下方 | 恢复态仅保留一份输入与打开已有／生成新版；事实固定值列，主操作位于历史前 | [check06-07](design-review-2026-09-09/composition-evidence/check06-07-restored-workbench.jpg)、[check06-15／16／17](design-review-2026-09-09/composition-evidence/check06-16-tasks-stages.jpg)。**正常结构及 [check10-45 小窗](design-review-2026-09-09/composition-evidence/check10-45-small-tasks.jpg) 时间／事实层级均有新图支持** |
| F14／F15／F18 AI 进度／暂停／失败分类失真 | [check06-08](design-review-2026-09-09/composition-evidence/check06-08-conversion-running.jpg) 请求仍待返回却称 100%；[10](design-review-2026-09-09/composition-evidence/check06-10-conversion-paused-settled.jpg) 主动暂停画为红色双语错误；[19](design-review-2026-09-09/composition-evidence/check06-19-partial-result.jpg) 失败校对进入完成历史 | 派发数量不当结果数量；按暂停意图反馈；按保存的组件结果分类成功／未完成 | [check08-01～06](design-review-2026-09-09/composition-evidence/check08-02-failed-stages-separated.jpg) 及四项专项测试支持修复。**主任务状态图证闭环**；最后 [check12-11 部分结果](design-review-2026-09-09/composition-evidence/check12-11-final-partial-result.jpg) 的两个 AI 失败单列、成功历史四项，[12 最近笔记](design-review-2026-09-09/composition-evidence/check12-12-final-partial-recent-notes.jpg) 的部分处理状态仅一次且无重复恢复卡，见转换独立记录第十节 |
| F19 取消中动作与落定反馈缺失 | [check08-13](design-review-2026-09-09/composition-evidence/check08-13-cancel-request.jpg) 的取消中动作错误；[check08-14](design-review-2026-09-09/composition-evidence/check08-14-cancel-settled.jpg) 取消落定后没有对应提示 | 取消意图与落定反馈已随 `6c2e997` 修正 | [check09-02／03](design-review-2026-09-09/composition-evidence/check09-03-cancelled.jpg) 已证实取消本体措辞与普通反馈。**取消本体静帧闭环**；随后发现旧通知再出现，见 F20。check08 仍是旧失败图 |
| F20 等待误作错误、已看过的旧通知再出现 | [check09-01](design-review-2026-09-09/composition-evidence/check09-01-restored-cancelled.jpg) 普通组件检测用红色错误；[02／03](design-review-2026-09-09/composition-evidence/check09-03-cancelled.jpg) 新版任务关联更新后旧取消通知重现 | `d9a9bda` 将精确正常等待态用共享中性状态；成功入队事务确认已看到的当前旧终态 | [check11-09～11](design-review-2026-09-09/composition-evidence/check11-11-restart-after-cancellation.jpg) 与操作者连续取消／再开始支持**旧通知问题闭环**；中性组件等待只有源码证据，11-07 已就绪，未捕获 loading 静帧 |
| RINFO 生成信息区边界与结果层级 | [check08-11／12](design-review-2026-09-09/composition-evidence/check08-12-reader-information-settled.jpg) 展开只露约 112px、边界截断、小字同层 | 独立信息滚动区、固定收起头、共享标签／值行，优先展示四项生成结果 | 阅读独立审查已查看 [check09-04 顶部](design-review-2026-09-09/composition-evidence/check09-04-reader-information.jpg) 与 [05 末尾](design-review-2026-09-09/composition-evidence/check09-05-reader-information-bottom.jpg)。**正常字号及 [check10-35／37 窄 200%](design-review-2026-09-09/composition-evidence/check10-37-small-reader-information-scrolled.jpg) 均闭环**；窄页头／边框／固定收起位置一致，每次约显示一项信息。正常两图全局消息不同，整体位移不归因于信息滚动 |
| LFAULT 库故障重复说明、读取错误无下一步 | [check09-06](design-review-2026-09-09/composition-evidence/check09-06-library-failure.jpg) fixture 分类 JSON 损坏产生重复黄框及原始英文；[07](design-review-2026-09-09/composition-evidence/check09-07-reader-file-missing.jpg) 暂移 artifact 后只报英文 I/O | `e217d0d` 把诊断收在恢复组、保留真实恢复动作；读取错误按链区分未找到／权限／其他原因并给下一步 | [check11-02／03／04](design-review-2026-09-09/composition-evidence/check11-04-library-recovered.jpg) 单一恢复组、展开诊断和恢复常态，[05](design-review-2026-09-09/composition-evidence/check11-05-reader-file-unavailable.jpg) 中文建议已由阅读审查关闭；**成功重试旧提示残留另列 NOTICE** |
| NOTICE 成功重试后旧读取失败提示残留 | [check11-06](design-review-2026-09-09/composition-evidence/check11-06-reader-retry-restored.jpg) 同时出现可读笔记与旧无法访问提示 | `5740837` 仅清除本次读取捕获且当前仍匹配的失败／入队提示，新消息保留；四项状态测试已执行 | [check12-07 读取失败](design-review-2026-09-09/composition-evidence/check12-07-final-reader-unavailable.jpg)→[08 恢复后阅读](design-review-2026-09-09/composition-evidence/check12-08-final-reader-restored-no-error.jpg) 在 release 9ec 中显示可读版本 6 且旧提示消失；操作者未手动关闭消息。**阅读独立复核关闭这一成功重试缺陷** |
| S10-01 大字号 switch 比例失衡 | [check10-05](design-review-2026-09-09/composition-evidence/check10-05-small-appearance-controls.jpg)、[08](design-review-2026-09-09/composition-evidence/check10-08-small-generation-bottom.jpg) 在真正 860×620／200% 下，文字／图标／单选轨道放大，但 switch 绘制仍约 36×20 | 已新增可重放 [Switch 几何补丁](../desktop/patches/component-switch-geometry.patch)，共享适配使用 20/14 rem；引导唯一绕过 caller 也接入，原事件／焦点／spring 保留 | [check12-01](design-review-2026-09-09/composition-evidence/check12-01-small-switch-scaled.jpg) 与 [04 引导](design-review-2026-09-09/composition-evidence/check12-04-small-guide-switch-scaled.jpg) 显示 200% 的 72×40 轨道和同比例圆点；主代理独立看图并执行鼠标 on／空格 off。**该比例问题闭环**，不是全部键盘／动画验收 |
| L01／R01／G01 库、正文及图库各自起点 | 元信息继承粗体；标题被返回图标挤窄；图片被卡片框包裹；图库窄列和摘要／操作不齐 | 库元信息正常字重；返回独立行＋全宽标题；正文自然比例图片不加卡边；图库按实际宽度排列 | [宽库](design-review-2026-09-09/composition-evidence/check03-28-wide-library.jpg)、[宽阅读](design-review-2026-09-09/composition-evidence/check03-29-wide-reader.jpg)、[宽图库](design-review-2026-09-09/composition-evidence/check03-30-wide-gallery.jpg)、[窄标题](design-review-2026-09-09/composition-evidence/check06-02-narrow-reader-title.jpg)。**所列构图有新图支持**，异步首开／重排恢复由测试和实操另记 |
| R02／V01 窄查找计数与查看器图片被挤压 | 200% 查找计数被限高截半行；低矮查看器默认转录使图片约 100px 高 | 查找脱离附加信息限高；查看器低高度默认收文字，文字独立滚动、底部操作保留 | [check07-05](design-review-2026-09-09/composition-evidence/check07-05-narrow-search-title.jpg) 查找标题／计数／命中完整；[check04-06／07／08](design-review-2026-09-09/composition-evidence/check04-08-narrow-viewer-scroll.jpg) 图片优先且可滚到文字末尾。**这些状态图证闭环** |

更多逐图依据及保留限制分别见[转换／任务独立记录](design-review-2026-09-09/composition-evidence/conversion-review.md)、[库／阅读／图库独立记录](design-review-2026-09-09/composition-evidence/reader-review.md)和[设置／引导独立记录](design-review-2026-09-09/composition-evidence/independent-settings-guide-check04.md)。记录保留早期失败与后续更新，不能只摘第一段或最后一句作为全局结论。

## 全产品页面与状态矩阵

矩阵覆盖本轮受共享组件及流程修改影响的全部页面类别；没有图片或操作记录的格子明确留空。某一页的检查失败不能替代另一页的失败状态，服务检查失败尤其不能充当转换任务失败证据。

| 页面／子区 | 本轮正常或宽窗证据 | 窄窗 200% 证据 | 已见状态与验证边界 |
| --- | --- | --- | --- |
| 全局导航／应用通知 | [check03-38 键盘焦点](design-review-2026-09-09/composition-evidence/check03-38-keyboard-focus.jpg)、[check06-12 后台完成](design-review-2026-09-09/composition-evidence/check06-12-background-completed-settings.jpg) | [check04-02 库](design-review-2026-09-09/composition-evidence/check04-02-narrow-library.jpg) | 多页框架与后台通知可见；焦点静帧不代表完整 Tab 顺序，所有 pressed／focus 叠加未逐项实操 |
| 工作台：空输入／本地选择／恢复输入 | [check06-03 空](design-review-2026-09-09/composition-evidence/check06-03-empty-workbench.jpg)、[check06-07 恢复](design-review-2026-09-09/composition-evidence/check06-07-restored-workbench.jpg) | [check03-13 文件行重排](design-review-2026-09-09/composition-evidence/check03-13-narrow-workbench.jpg)，仍含旧恢复重复，不能作为最终整页 | 正常恢复的重复已闭环；最终窄恢复整页、所有高级选项展开尚缺 |
| 工作台：链接／来源读取／真实来源选择 | [check06-06 无效链接](design-review-2026-09-09/composition-evidence/check06-06-invalid-source.jpg)、[after-15 读取／转换](design-review-2026-09-09/composition-evidence/after-15-converting.jpg) | 无本轮最终图 | 无效输入留在字段附近；链接成功→读取→提交→结果完整原生路径尚缺，字幕选择／取消／fallback 的测试不替代这条实操 |
| 工作台：转换／等待／暂停 | [check08-03 等待](design-review-2026-09-09/composition-evidence/check08-03-ai-awaiting-result.jpg)、[04 暂停中](design-review-2026-09-09/composition-evidence/check08-04-pausing.jpg)、[05 暂停落定](design-review-2026-09-09/composition-evidence/check08-05-paused-settled.jpg) | 无本轮最终状态图 | 主任务等待／暂停有图；[check11-09～11](design-review-2026-09-09/composition-evidence/check11-11-restart-after-cancellation.jpg) 取消及再开始无旧通知已闭环；不确定请求未覆盖 |
| 工作台：部分失败／定向恢复／最近结果 | [check08-06 仅补校对](design-review-2026-09-09/composition-evidence/check08-06-recovery-result.jpg)、[check12-11 最后部分结果](design-review-2026-09-09/composition-evidence/check12-11-final-partial-result.jpg)、[12 最近笔记](design-review-2026-09-09/composition-evidence/check12-12-final-partial-recent-notes.jpg) | 无本轮最终图 | 本地 401 的两项 AI 失败、四项成功历史和按组件恢复清楚；最近版本 8 的部分状态仅一次，无额外当前恢复卡。曾实操仅补校对后保留未完成摘要。来源彻底失败、写盘失败、不确定重发授权未全覆盖 |
| 完成导航／后台完成 | [check06-11→12 设置不被抢走](design-review-2026-09-09/composition-evidence/check06-12-background-completed-settings.jpg)、[13 点击后结果](design-review-2026-09-09/composition-evidence/check06-13-background-result-reader.jpg)、[check11-12 新版直达](design-review-2026-09-09/composition-evidence/check11-12-conversion-opens-reader.jpg) | 无本轮最终完成图 | 操作者实际执行上述本地路径；换输入／打开另一笔记／reader 加载竞争只由规则测试覆盖，未全部原生重放 |
| 任务页：空列表／完成／完成详情／日志 | [check06-05 空](design-review-2026-09-09/composition-evidence/check06-05-empty-tasks.jpg)、[15 列表](design-review-2026-09-09/composition-evidence/check06-15-tasks-complete.jpg)、[16 阶段](design-review-2026-09-09/composition-evidence/check06-16-tasks-stages.jpg)、[17 日志](design-review-2026-09-09/composition-evidence/check06-17-task-logs.jpg) | [check10-45 标题／事实](design-review-2026-09-09/composition-evidence/check10-45-small-tasks.jpg)、[46 操作](design-review-2026-09-09/composition-evidence/check10-46-small-task-actions.jpg)、[48 阶段](design-review-2026-09-09/composition-evidence/check10-48-small-task-stages.jpg)、[51 日志](design-review-2026-09-09/composition-evidence/check10-51-small-task-log-content.jpg) | 正常及 200% 完成结构、普通字重时间、固定事实／数量列和操作在历史之前已有图。任务页本身的所有运行／暂停／失败组合、任意长日志未逐一覆盖 |
| 笔记库：空／列表 | [check06-04 空](design-review-2026-09-09/composition-evidence/check06-04-empty-library.jpg)、[check03-28 宽列表](design-review-2026-09-09/composition-evidence/check03-28-wide-library.jpg)、[installed-02 用户库](design-review-2026-09-09/composition-evidence/installed-02-library.jpg) | [check10-30 标题](design-review-2026-09-09/composition-evidence/check10-30-small-library.jpg)、[33 辅助操作](design-review-2026-09-09/composition-evidence/check10-33-small-library-aligned-actions.jpg) | 标题、元信息及换行操作可读；分类故障另列下一行。加载、扫描失败、部分覆盖、库搜索零结果没有本轮完整状态图 |
| 笔记库：分类记录故障／诊断／恢复 | [check11-02 恢复组](design-review-2026-09-09/composition-evidence/check11-02-library-recovery.jpg)、[03 诊断](design-review-2026-09-09/composition-evidence/check11-03-library-recovery-diagnostics.jpg)、[04 恢复后](design-review-2026-09-09/composition-evidence/check11-04-library-recovered.jpg) | 无此故障的窄窗图 | 故障来自隔离 fixture 分类 JSON，单一恢复组及展开诊断已复核，恢复／刷新后常态可见；未逐一执行两个恢复选项，未损坏真实用户数据 |
| 阅读：文件暂不可用／成功重试 | [check12-07 失败](design-review-2026-09-09/composition-evidence/check12-07-final-reader-unavailable.jpg)→[08 恢复](design-review-2026-09-09/composition-evidence/check12-08-final-reader-restored-no-error.jpg) | 无此故障的窄窗图 | 暂移 fixture 目录使读取失败，恢复后再次阅读可读版本 6 且旧提示消失，操作者未手动 dismiss；不称文件永久丢失，不扩展为全部权限／磁盘错误 |
| 笔记库：卡片／更多／重命名 | 正常库早期图 [after-21](design-review-2026-09-09/composition-evidence/after-21-library.jpg) | [check10-42 真卡片](design-review-2026-09-09/composition-evidence/check10-42-small-library-cards.jpg)、[43 内容](design-review-2026-09-09/composition-evidence/check10-43-small-library-card-footer.jpg)、[44 辅助及阅读动作](design-review-2026-09-09/composition-evidence/check10-44-small-library-card-actions.jpg) | 真卡片的封面／标题／元信息、文件夹／更多共轴和下一行全宽阅读动作已复核。整卡仍分段滚动；顶部工具区约占 420px、内容约 200px，不称空间宽裕。重命名菜单及操作错误未覆盖 |
| 阅读：返回／标题／元信息／正文／目录 | [check03-29 宽深色](design-review-2026-09-09/composition-evidence/check03-29-wide-reader.jpg)、[check06-13 正常](design-review-2026-09-09/composition-evidence/check06-13-background-result-reader.jpg)、[installed-03 用户正文](design-review-2026-09-09/composition-evidence/installed-03-reader.jpg) | [check06-02](design-review-2026-09-09/composition-evidence/check06-02-narrow-reader-title.jpg) | 正文正常字重、图片自然比例、标题共轴；宽目录与持久选中可见。首开、摘要导航、resize 恢复的测试不能替代全部原生时序；手动目录选择跨 resize 未完整重放 |
| 阅读：查找 | 正常阅读工具区有图，专门零结果无图 | [check04-03 空查询](design-review-2026-09-09/composition-evidence/check04-03-narrow-search-empty.jpg)、[check07-05 单一命中](design-review-2026-09-09/composition-evidence/check07-05-narrow-search-title.jpg) | 最新标题、计数和命中首行完整；多命中、零命中及逐个跳转未完整实操。200% 展开查找后正文首屏约一行，后文依靠滚动 |
| 阅读：截图图库 | [installed-05 实际 40 图库](design-review-2026-09-09/composition-evidence/installed-05-gallery-settled.jpg)、[check03-30 宽三列](design-review-2026-09-09/composition-evidence/check03-30-wide-gallery.jpg) | [check10-38 单列](design-review-2026-09-09/composition-evidence/check10-38-small-gallery.jpg)、[41 摘要／动作](design-review-2026-09-09/composition-evidence/check10-41-small-gallery-action-baseline.jpg) | 安装窗口首行三卡均为三行摘要，操作完整且跨列共基线；窄 41 的两行长摘要也与操作同屏。200% 恰满三行及损坏／缺失图片未覆盖 |
| 阅读：图片查看器／展开转录 | [after-20 正常查看器](design-review-2026-09-09/composition-evidence/after-20-viewer.jpg) | [check04-06 图片优先](design-review-2026-09-09/composition-evidence/check04-06-narrow-viewer-image.jpg)、[07 文字](design-review-2026-09-09/composition-evidence/check04-07-narrow-viewer-text.jpg)、[08 滚至末尾](design-review-2026-09-09/composition-evidence/check04-08-narrow-viewer-scroll.jpg) | 图片完整适配和底部动作可见；所有缩放比例、中心保持、损坏图片与缺失原视频未独立重放 |
| 阅读：生成信息／来源详情 | [check09-04 顶部](design-review-2026-09-09/composition-evidence/check09-04-reader-information.jpg)、[05 末尾](design-review-2026-09-09/composition-evidence/check09-05-reader-information-bottom.jpg) | [check10-35 顶部](design-review-2026-09-09/composition-evidence/check10-35-small-reader-information.jpg)、[37 内部末尾](design-review-2026-09-09/composition-evidence/check10-37-small-reader-information-scrolled.jpg) | 正常和窄状态均已复核完整边界／固定头／内部滚动；窄每次约一项信息，关闭后恢复正文空间。36 未滚进内部，不充当末尾证据 |
| 设置：外观／字体／十配色 | [check03-02 正常](design-review-2026-09-09/composition-evidence/check03-02-appearance.jpg)、[31 宽 125%](design-review-2026-09-09/composition-evidence/check03-31-wide-appearance.jpg)、[34 宽 150%](design-review-2026-09-09/composition-evidence/check03-34-wide-150.jpg)、全部 [palette 组](design-review-2026-09-09/composition-evidence/README.md#配色原图) | [check10-04 模式](design-review-2026-09-09/composition-evidence/check10-04-exact-small-appearance.jpg)、[check12-05 开关落定](design-review-2026-09-09/composition-evidence/check12-05-small-appearance-state.jpg) | 十配色选择及 125／150 控件轴可见；200% 模式已换成纵向组，S10-01 已由 check12-01～05 关闭，减少动态效果鼠标／空格切换实际执行。保存失败／重试、全部 hover／pressed／focus 及减少动态效果下所有页面未遍历 |
| 设置：生成笔记 | [check04-09／10](design-review-2026-09-09/composition-evidence/check04-10-generation-lower.jpg) | [check10-06 顶部](design-review-2026-09-09/composition-evidence/check10-06-small-generation-top.jpg)、[07 识别模型](design-review-2026-09-09/composition-evidence/check10-07-small-generation-middle.jpg)、[08 AI 下部](design-review-2026-09-09/composition-evidence/check10-08-small-generation-bottom.jpg) | 正常态全部主组有图，200% 已见分组／模型选项／AI 下部；全部展开、长语言／服务名及窄导出末尾未全覆盖，开关比例已由 check12 关闭 |
| 设置：服务与账号／编辑／模型菜单 | [check03-35 宽旧重复标题](design-review-2026-09-09/composition-evidence/check03-35-services-wide.jpg)、[check04-24／25 新编辑组](design-review-2026-09-09/composition-evidence/check04-25-service-editor-actions.jpg)、[after-08 模型菜单](design-review-2026-09-09/composition-evidence/after-08-model-menu.jpg) | [check10-09 服务顶](design-review-2026-09-09/composition-evidence/check10-09-small-services.jpg)、[15 服务卡](design-review-2026-09-09/composition-evidence/check10-15-small-ai-card.jpg)、[16／17／18 编辑及保存](design-review-2026-09-09/composition-evidence/check10-18-small-editor-actions.jpg) | 最新窄服务总页无重复 Bilibili 标题；编辑输入／认证／模型／底部操作各有图。11 实际为 About，不计编辑证据。校验／保存失败及长菜单／键盘仍缺 |
| 设置：存储／位置状态 | [check04-11](design-review-2026-09-09/composition-evidence/check04-11-storage.jpg) | [check10-19 顶部](design-review-2026-09-09/composition-evidence/check10-19-small-storage-top.jpg)、[20 动作](design-review-2026-09-09/composition-evidence/check10-20-small-storage-bottom.jpg) | 健康位置、说明／刷新及新增位置／详情入口有图。窄路径中段未完整展示；多个位置、长路径、不可用、迁移／恢复和失败写入仍缺 |
| 设置：应用／About／诊断 | [check04-12／13](design-review-2026-09-09/composition-evidence/check04-13-about-idle.jpg) | [check10-12 身份](design-review-2026-09-09/composition-evidence/check10-12-small-about.jpg)、[21 许可／引导](design-review-2026-09-09/composition-evidence/check10-21-small-about-actions.jpg)、[22 诊断首组](design-review-2026-09-09/composition-evidence/check10-22-small-about-health.jpg) | 身份、许可链接、引导入口及诊断标签／状态的窄重排有图；长版本、多项异常、全部展开和所有动作边界尚未完全展示 |
| 引导 1：识别方式／引擎／模型 | [check07-01／02](design-review-2026-09-09/composition-evidence/check07-02-engine-capabilities.jpg) | [check10-23 默认](design-review-2026-09-09/composition-evidence/check10-23-small-guide-engine.jpg)、[24 模型区](design-review-2026-09-09/composition-evidence/check10-24-small-guide-engine-choices.jpg) | 正常卡片首行轴闭环；窄步骤标题／底部固定操作稳定，模型卡仅部分可见，不能用该帧补全部描述。不能仅凭滚动视口推断不可达 |
| 引导 2：AI／认证／发现／检查 | [check04-15／16](design-review-2026-09-09/composition-evidence/check04-16-guide-ai-result.jpg)、[check03-11 失败](design-review-2026-09-09/composition-evidence/check03-11-ai-failure.jpg)、[12 运行](design-review-2026-09-09/composition-evidence/check03-12-ai-running.jpg)、[after-07／08 发现／菜单](design-review-2026-09-09/composition-evidence/after-08-model-menu.jpg) | [check10-25 地址](design-review-2026-09-09/composition-evidence/check10-25-small-guide-ai-top.jpg)、[26 用途开关](design-review-2026-09-09/composition-evidence/check10-26-small-guide-ai-model.jpg) | 正常初态／失败／忙碌／成功各有图；窄地址、用途和底部固定操作有图。26 不是完整模型框；认证编辑、停止后迟到响应与键盘轨迹仍缺，开关比例已由 check12 关闭 |
| 引导 3：Bilibili／原生 QR | [check04-17 未连接](design-review-2026-09-09/composition-evidence/check04-17-guide-account.jpg)、[check03-18 QR 就绪](design-review-2026-09-09/composition-evidence/check03-18-guide-login-loading.jpg) | [check10-27 未登录](design-review-2026-09-09/composition-evidence/check10-27-small-guide-bilibili.jpg) | 正常原生 QR 就绪有图，窄状态／说明／跳过完整；登录／检查动作部分在正文视口下方。未完成扫码登录、过期／取消／断网恢复的独立复核 |
| 引导 4：准备／真实阶段／暂停 | [check07-03 无样本](design-review-2026-09-09/composition-evidence/check07-03-model-start-fixed.jpg)、[04 阶段](design-review-2026-09-09/composition-evidence/check07-04-model-progress-fixed.jpg)、[check04-20 暂停](design-review-2026-09-09/composition-evidence/check04-20-model-paused.jpg) | [check10-28 资源不完整](design-review-2026-09-09/composition-evidence/check10-28-small-guide-model.jpg)、[29 准备动作](design-review-2026-09-09/composition-evidence/check10-29-small-guide-model-actions.jpg) | 正常等待／进度／暂停有图，窄资源不完整／复用说明／准备操作有图；不是本轮新增下载／验证。下载失败／完成和后台／文件重用未由独立审查者完整实操 |

## 动态、行为与测试证据

滚动条的正常 About [check04-12](design-review-2026-09-09/composition-evidence/check04-12-about.jpg)／[13](design-review-2026-09-09/composition-evidence/check04-13-about-idle.jpg) 对照只支持“所示显／隐端点内容宽度不变”。图库／查看器文字的连续滚动位置支持动作可达。它们没有测出显隐时序，也没有覆盖拖动 thumb、惯性、系统始终显示偏好和全部减少动态效果状态。

PID 43350 的 [release 性能摘要](design-review-2026-09-09/composition-evidence/performance-summary.json) 记录：453 次 draw 的 p95 为 5.612ms、p99 为 14.451ms、最大 29.573ms；96 次 animation interval 的 p95 为 20.070ms；17 次 input-to-present 的 p95 为 82.051ms。15.081 秒前台 idle 窗口内 draw_delta 为 0。reader 入场和完成 banner 分别记录 29／14 个样本，包含 0→1 的中间值。这些是这段原生操作中的实测样本，**不能写成保证 60fps、所有交互都流畅或所有动画都验证完成**；draw 耗时也不是端到端帧间隔。原始 [PID 43350 trace](design-review-2026-09-09/composition-evidence/release-performance-trace.jsonl) 和[仪器构建身份](design-review-2026-09-09/composition-evidence/performance-build.json) 已归档。该可执行文件 SHA-256 为 `ba99a18d64ae0679df6aa31988b0d2e5a986ffe1b2433334dbc15cb65a717e1c`，早于库去重、生成信息及取消的最后修改；它不是最终安装二进制的性能证明。

| 执行证据 | 实际结果 | 能说明什么／不能说明什么 |
| --- | --- | --- |
| [tests-desktop-final.log](design-review-2026-09-09/composition-evidence/tests-desktop-final.log) | **201 passed，0 failed，3 ignored**；最终 9ec2f6e 重新执行，用时 2.64s；此前 5740837 的 201 项用时 2.81s | 转换意图／完成归属、启动库扫描等待、恢复链、阅读锚点、读取失败提示清理及 AI 派发进度／组件结果／暂停与取消反馈等断言执行成功；3 ignored 不算通过。不是原生构图验收 |
| [tests-task-execution.log](design-review-2026-09-09/composition-evidence/tests-task-execution.log) | **11 passed，0 failed** | 核心任务执行相关断言；不证明 UI 点击和动画 |
| [本轮格式检查](design-review-2026-09-09/composition-evidence/format-changed-final.log) | 15 个本轮 desktop Rust 模块 rustfmt 检查通过 | [全仓检查](design-review-2026-09-09/composition-evidence/format-final.log) 仍有现有 legacy／core／vendor 差异，**未通过**；不记为全仓格式通过 |
| [最终 release 构建](design-review-2026-09-09/composition-evidence/build-release-final.log) | 9ec2f6e release、不含 performance，49.57s 完成；日志保留 14 项 warning 及依赖未来兼容性提示 | 证明该构建生成成功；没有把 warning 忽略成零警告，也不以编译替代原生审查 |
| skill 结构、引用与来源校验 | 29 份来源 SHA-256 验证通过；结构和路径检查通过 | 证明资料存在、未改变和结构有效；不证明设计质量 |
| 本轮归档校验 | 原图复制后哈希核对、图片格式／尺寸读取、相对图片与日志引用检查 | 证明归档可审查且图像字节未被编辑；不证明所有页面已独立查看 |

上述测试日志由主代理运行产生，本报告编辑者只读取日志。更早 `tests-desktop.log` 的 191 passed、`tests-desktop-cleanup.log` 的 195 passed 和 `tests-desktop-switch-final.log` 的 197 passed 作为历史保留，不能替代新增测试后的 201 passed。最后的 9ec2f6e 是机械格式化提交，主代理仍重新执行了这份最终测试；身份与 release 构建分开记录。

## 问题关闭与收尾状态

本轮登记的构图、重复信息和状态反馈缺陷已经获得对应复拍或明确的源码／测试依据。最后的读取成功旧提示残留由 check12-07→08 关闭，最近部分结果去重由 check12-11／12 关闭；较早失败图继续保留，不能摘作最后结果。没有新登记而尚未修复的必修缺陷。

Design Preview `20260909.5` 已实际安装，启动 PID、可执行文件、签名及哈希身份已记录。About、实际用户库、reader、实际 40 图图库及返回 00:27 的落定图已独立复核；相应入场图与落定图均已归档，详见安装记录和阅读独立记录的最后更新。没有把新包安装等同于全状态验收。

## 验证边界

矩阵中未遍历的组合属于证据边界，不等于已发现的必修缺陷。真实小窗的设置、四步引导、库／卡片、阅读／信息／图库／查看器和完成任务主要区段已完成逐图审查；极限尺寸仍需要分段滚动，不能把可达描述成宽裕空间。

- 冷启动普通组件检测过快，没有捕获该中性等待状态静帧；check11-07 已就绪，只有源码／测试依据。链接成功全流程、用户在 reader 加载极短窗口改输入／跳页，以及每种网络／磁盘时序未全部原生重放。
- 未逐一覆盖的页面组合包括全部模型卡长描述、认证／长字段、诊断多异常、设置保存失败、存储迁移、QR 登录完成／过期、模型准备失败／完成、损坏截图／缺失原视频、搜索零／多命中和不确定重发。已有健康态或别页失败不会替代它们。
- 鼠标和空格切换已对修正后的开关实际执行；不代表全应用键盘序列或所有选中／hover／pressed／focus 组合。显隐端点、运动 trace 和性能样本也不替代全部拖动／惯性／系统偏好／动画中断组合。

## 官方离线资料与项目规则

旧 skill 只有摘要和链接，不足以让后续实现者离线判断对齐、分组、字体测量和状态反馈。本轮实际获取官方内容并写入 [course2md-design](../.agents/skills/course2md-design/SKILL.md)，来源范围与许可见[资料说明](../.agents/skills/course2md-design/references/material-sources.md)和 [sources.json](../.agents/skills/course2md-design/references/material/sources.json)。

资料包有 **29 个带 SHA-256 的来源文件，共 201,582 字节**，不把 manifest 和项目指南计入这 29 个文件：

| 保存类型 | 数量 | 实际保存内容与边界 |
| --- | ---: | --- |
| 官方全文文本 | 10 | 7 篇官方网页的完整作者正文转换为 Markdown，3 份官方仓库 Markdown；涵盖布局、网格、M3 角色、Symbols、状态、动效、进度、卡片、列表和对话框。保留正文、代码、表格、文字说明与链接；不下载插图／视频，不声称完整视觉页面镜像。 |
| M3 短摘 | 16 | 从 M3 官方公开内容数据核对的短引文，每篇不超过 25 个英文词，附官方页面 URL、公开数据 URL、版本、获取日期、哈希与许可状态。涵盖 layout／grids／canonical layouts／typography／icons／states／motion／progress 及相关组件。 |
| 许可证据 | 3 | 两份官方 Apache 2.0 许可全文，以及 Android 文档许可条款短摘。 |

M3 是 JavaScript 页面。本轮读取的是其公开发布的实际内容数据，记录版本为 `2026-09-02_06-10-10`；没有把非空下载体积或 HTML 壳当成完整规范。M3 网站正文没有核实到开放再分发许可，因此项目仅保存短摘和来源信息，未保存完整 M3 JSON、正文或图片。

10 份全文来自有明确许可的官方文档：Android 文档与 Material Android／Web 仓库按各自 Apache 2.0 依据保存；Google Fonts 的 Material Symbols 文档按其 CC BY 4.0 页脚保存归属及转换说明。CC BY 4.0 法律文本端点返回 403，资料包保留许可 URL，未声称下载到该许可全文。商标和链接媒体没有被推定为同一许可。

官方原则与本项目选择分别记录。官方材料用于核对布局锚点、接近关系、类型角色、图标测量、状态层、真实进度和有意中断的对话框等概念；它们并未规定 course2md 必须采用 200px 标签列、560px 控件上限或每条主标签配图标。后者以及 UI 主标签加粗、说明放共享信息面板、正文保持普通字重、滚动条按交互显隐、移除例行计划与完成中间页，属于用户要求和项目适配。

可实施规则位于[布局与文字](../.agents/skills/course2md-design/references/layout-and-type.md)、[状态与动效](../.agents/skills/course2md-design/references/states-and-motion.md)、[设置构图](../.agents/skills/course2md-design/references/settings.md)和[任务连续性](../.agents/skills/course2md-design/references/interaction.md)。实现前先列共同轴线和文字／图标实际边界，再选共享控件；不能把长篇笔记正文机械地变为全粗体或每段一个图标。

## 最终构建与安装身份

当前代码提交为 `9ec2f6ef89b923226fa178f722c7b6a07fa71423`，本轮已形成十一个按关注点分开的规范／代码／格式提交：

| 提交 | 关注点 |
| --- | --- |
| `1998b5a` | 官方 Material 离线资料与设计／审查规则 |
| `86657f2` | 共享标签轴、信息面板及状态动效 |
| `4fc12e2` | 引导与设置构图稳定性 |
| `59ae6c9` | 转换连续性与任务反馈语义 |
| `4c8e23f` | 阅读布局与生成信息区可用性 |
| `6c2e997` | 启动时库扫描等待与取消反馈 |
| `d9a9bda` | 中性准备状态与已看过的旧结果通知确认 |
| `e217d0d` | 库恢复与笔记读取失败的可操作提示 |
| `2a06d54` | 共享开关随界面文字缩放及引导 caller 接入 |
| `5740837` | 成功重试清除自己对应的旧失败提示、新通知保留 |
| `9ec2f6e` | 本轮 icons／main／onboarding／reader_ui 的机械格式化 |

`6c2e997` 的前一份非 performance release 身份另存为 [build-identity-6c2e997.json](design-review-2026-09-09/composition-evidence/build-identity-6c2e997.json)，编译产物 SHA-256 为 `9bc5468fd6e44714b9804840fb9bd6c8f0a5582586482652205813a4cf2f9a7e`，同时记录各 fixture 内副本哈希。它对应 check09-01～08 等中间复拍；不会因为后续覆盖 final-build.json 而丢失归属。

check10-04～51 为 debug e217，check11 为 release e217，check12-01～05 为 debug 574，check12-06 起及 installed 为 release 9ec。各组身份和原图哈希分别保留，不将较早截图追溯归属后来二进制。

[final-build.json](design-review-2026-09-09/composition-evidence/final-build.json) 记录最终 release 未启用 performance，编译产物 SHA-256 为 `824cbe063bc7c8095146d723f9876fb84020c7f68485aa03c1464049c317fd27`；精确小窗使用 debug 574，编译产物为 `95ca4561c06237b7ba2c4329d2abc1592ea8925ac63d4643a5b4119dd56cd6a1`。fixture 包各自的签名副本哈希也在该文件中，不能声称与编译产物或最终安装副本逐字节相同。此前 PID 43350 的仪器包性能不构成这份安装包的性能证明。

[installation.json](design-review-2026-09-09/composition-evidence/installation.json) 记录实际安装路径 `/Users/aac6fef/Applications/course2md Design Preview.app`，语义版本 `2.0.0-alpha.1`，CFBundleVersion `20260909.5`，代码 `9ec2f6e`；安装副本 desktop SHA-256 为 `99553c4d8d8d1b4322d02798d945e84e0cae6beb7fad1a64e062f7f9dfccafc1`，worker 为 `4601c7d5e04eb897217b6c7ee92b9ef0b98cd485186cda2724ab32b7b915058c`。主代理记录签名验证通过、13 项配置文件保持原样、公开 release 未修改，旧包备份在 `/Users/aac6fef/Library/Application Support/course2md-design-preview/backups/composition-20260909-170139`。报告编辑者读取记录，没有亲自安装。

[installed-01 About](design-review-2026-09-09/composition-evidence/installed-01-about.jpg) 可见构建标识 `9ec2f6ef89b9`；[02 实际用户库](design-review-2026-09-09/composition-evidence/installed-02-library.jpg) 显示原有 40 段笔记／40 张截图，[03 实际 reader](design-review-2026-09-09/composition-evidence/installed-03-reader.jpg) 显示摘要、正文、图片与目录。本报告编辑者已独立看过这三张图，未发现新的必修构图问题。实际运行 PID 为 `98163`，可执行文件为安装包内的 `Contents/MacOS/course2md-desktop`；主代理再次确认启动后哈希与安装记录一致、签名有效。[installed-04／05 图库](design-review-2026-09-09/composition-evidence/installed-05-gallery-settled.jpg) 和 [06／07 返回 00:27](design-review-2026-09-09/composition-evidence/installed-07-reader-anchor-settled.jpg) 分别保留入场与落定图，04／06 不充当落定布局证据。阅读审查者已独立查看 02／03／05／07 及对应原生状态快照：图库首行三卡均为三行摘要，动作完整且共基线；07 的 00:27 段落和目录选中一致。03 的“摘要”标题只在原生状态文本中，静帧从摘要首段开始，不能充当首开绝对顶端的证明。该真实资料的 40 图来自用户原有笔记，不与六图 fixture 的 CLI 准备方式混淆，也不声称每一张图和所有滚动位置均独立检查。
