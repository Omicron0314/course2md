# 设置页设计验收失败复盘

> 历史记录：这轮交付随后再次被用户拒绝。下文的 520px 子列、主题网格和验收结论不能继续作为当前规范；以 [2026-09-09 体系重置复盘](DESIGN-SYSTEM-RESET-2026-09-09.md)及项目 design skill 为准。

用户再次打开上一轮交付的页面，就指出了导航、文字层级、选择形态、冗余说明和动画反馈的问题。[用户批注原图](design-review-2026-09-08/settings-correction/user-rejected-settings.png)是本轮起点。此前“逐页复审通过”的表述范围过大，不能成立；构建成功、功能测试和性能采样的事实仍有效，但它们不是设计质量的证明。

## 漏检了什么

| 用户看到的问题 | 实际原因 | 本轮约束 |
| --- | --- | --- |
| 宽屏设置挤在中央窄列，分类可以用侧栏 | 设置沿用工作台的920px最大宽度，未按信息结构区分布局；此前只检查1140和860窗口，没有评价用户宽屏初始画面。 | 设置独立1200px壳；宽窗200px分类侧栏与内容并列，窄窗单行分类导航。分类不随内容滚走。 |
| 分类条没有占据可用宽度 | 手写Tabs按标签收缩，且允许折成多行。 | 窄窗五个分类均分一行；宽窗采用侧栏，不将分类导航误做模式选择器。 |
| 形状与项目胶囊风格不一致 | 同一互斥选择由Tab、Button和Radio三处分别写样式；共享seg_item使用8px圆角，外观页又覆盖边框与选中颜色，与另一组胶囊控件不一致。 | 外观模式和字号复用共享SingleChoiceGroup；限制控件组最大520px，统一轨道、选中胶囊和鼠标反馈。 |
| 当前设置状态不明显 | 顶栏只用淡背景加文字着色，问题状态又覆盖文字颜色。 | 稳定的强调色形状、对应前景和半粗体表示当前位置；问题使用独立警示图标，不取消当前位置样式。 |
| 设置标题太小，标签与说明像同一级 | 旧Skill要求24–28px页面标题与18px分组分层，实现却给页面标题使用18px章节字号；字段标签又被text_sm缩成辅助文字。 | 本轮统一28px页面标题；按实际角色用TEXT_DISPLAY / TEXT_TITLE / TEXT_BODY / TEXT_AUX，检查渲染结果，不能只看文件里声明过规范。 |
| “当前主题”和配色描述没有必要 | 在预览、选中勾选和分组已传达信息后，继续增加重复说明。 | 主题卡只有真实预览、名称和选中标记；删除重复当前主题行、配色形容词和明显的自动保存介绍。 |
| 看不到需要的动画 | 把页面淡入和整套颜色插值当成了控件反馈；分类、外观模式、字号、主题卡选择实际上直接切换。base Tab/Radio也不会自动补hover/pressed。 | 选择标记有自己的短过渡，文字和布局不移动；检查中间状态、连续改选和最终停止。 |

独立审查还发现：外观保存失败显示在调色卡之后，可能不在当前视口；跟随系统时无法直接编辑另一套预设；开关被推到整个面板最右侧；减动画分支改变祖先ID链，可能让子控件丢掉动画状态。这些与用户批注属于同一轮信息组织和状态反馈缺陷，一并处理。

## 为什么两次“验收”仍然失败

1. **把工程验证当成设计判断。** 我主要确认能点击、能保存、没有明显越界，随后用了“设计通过”的说法。是否能立即识别页面、是否重复、控件是否一致、反馈是否可感知，都没有逐项判断。
2. **规范没有落实到共享实现。** Skill写了标题字号和胶囊规则，实际页面仍可任意覆盖。没有检查控件内部默认样式，也误以为无样式的base控件有悬停和动画。
3. **验证范围偏向已做的改动。** 给审查者提供了修复清单和若干局部截图，容易沿着实现者的结论确认；没有先给完整初始画面和设计要求做独立评判。
4. **没有区分动画证据。** 时长常量、最终截图、绘制帧率各证明不同事情。它们不能证明用户所点击的选择器经历了可见、连贯的过渡。
5. **结论超出了证据。** 未访问的宽屏、异常、长内容状态没有明确限制结论。我负责整合和交付，这个错误不应转交给用户继续找问题。

## 采用的规范及边界

以 [Apple Settings](https://developer.apple.com/design/human-interface-guidelines/settings)、[Sidebars](https://developer.apple.com/design/human-interface-guidelines/sidebars)、[Typography](https://developer.apple.com/design/human-interface-guidelines/typography)、[Writing](https://developer.apple.com/design/human-interface-guidelines/writing) 确定桌面信息架构与层级；以 [Material segmented buttons](https://m3.material.io/components/segmented-buttons/guidelines) 和 [interaction states](https://m3.material.io/foundations/interaction/states/overview) 确定选择控件及状态反馈。

这是对 course2md 的明确适配：保留用户要求的内嵌标题栏设置、Material图标和配色主题。1100px断点、1200px壳、200px侧栏、520px选择组以及160/200/280ms时长是项目选择，不冒充Apple或最新M3 Expressive的精确要求，也不因此引入玻璃效果或专项可访问性审计。

规范已写入[项目skill](../.agents/skills/course2md-design/SKILL.md)和[设置设计合同](../.agents/skills/course2md-design/references/settings.md)。后者按官方来源、项目决定、共享组件和可观察条件组织，后续相关设计与验收必须读取。

## 实现与运行证据

本轮使用隔离的原生应用、配置及合成课程，先检查默认文字大小。按正常窗口、860×620最小窗口、约1050px断点前窗口与1800×1040宽窗观察。截图来自实际GPUI窗口，导出会缩小较大窗口的像素尺寸，不据此倒推字号。

- [窄窗最终外观](design-review-2026-09-08/settings-correction/20-final-appearance-narrow-light.jpg)中四套浅色主题、名称与当前勾选全部进入首屏；[断点前导航](design-review-2026-09-08/settings-correction/24-compact-1050-navigation.jpg)五项单行，外缘与内容一致。首个修正构建仍把窄窗浅色卡排成两列，审查后再改成四列；没有拿早期“两列没有越界”当作合理布局。
- [宽窗系统模式](design-review-2026-09-08/settings-correction/08-system-both-palettes-wide.jpg)同时呈现两套预设；实际分别选Paper与Nord后，[系统模式仍保留](design-review-2026-09-08/settings-correction/09-system-presets-retained.jpg)。十套主题均实际切换并保留原生截图。未选中的[Nord Snow悬停](design-review-2026-09-08/settings-correction/25-palette-hover-wide.jpg)有边框强调，当前Latte同时保留勾选与底色，二者可以区分。
- 在最小窗口滚到下方卡片，将**测试配置**目录暂设为只读，再点Mocha。[保存失败](design-review-2026-09-08/settings-correction/12-save-failed-while-scrolled.jpg)和重试入口保持可见，磁盘仍是Tokyo Night；切到[应用分类](design-review-2026-09-08/settings-correction/13-failure-application-no-duplicate.jpg)没有重复提示或指向外观的错误链接。恢复目录权限并重试后，磁盘保存Mocha，[错误区域消失](design-review-2026-09-08/settings-correction/14-retry-saved.jpg)。用户的真实配置不用于这个故障测试。
- 生成、服务与账号、存储、应用四个分类另行检查。[生成页](design-review-2026-09-08/settings-correction/29-final-generation-wide.jpg)合并了重复自定义语言入口，模型说明改为首次使用时下载；[服务结果](design-review-2026-09-08/settings-correction/18-service-test-scrolled-result.jpg)在窄窗滚动后仍能看到说明及取消、测试、保存；[存储详情](design-review-2026-09-08/settings-correction/19-storage-details-narrow.jpg)可滚动，分类保持固定。服务测试仅使用本机测试地址，本次结果是未确认，没有写成连接成功。

两个独立审查者按原始批注和规范提出问题：[设置分类审查](design-review-2026-09-08/settings-correction/review.md)覆盖五个分类及已拍到的异常、展开状态；[主题与控件审查](design-review-2026-09-08/settings-correction/theme-review.md)覆盖十套主题、宽窄布局、断点前导航和主题卡悬停。两份记录各自区分实际所见、源码检查与未覆盖状态。最终[外观](design-review-2026-09-08/settings-correction/32-final-appearance-active.jpg)和[生成开关](design-review-2026-09-08/settings-correction/34-final-generation-controls.jpg)已实际确认紧凑行宽，共享顶栏还在工作台、我的笔记和任务中检查当前位置样式；这不被扩写为三条业务流程的重新全量验收。

## 动效和卡顿证据

共享选择器的GPUI测试实际驱动选择、推进时钟、测量中间帧几何，并覆盖连续反向选择、窗口宽度变化及结束后不再请求帧。更改“减少动态效果”保持同一祖先ID链，避免控件状态被重新建立。原生采样另外记录绘制中真正使用的选择值；文件编码与写入放在后台，普通交付构建不包含采样代码。

补测还纠正了一个方法错误：`get_app_state`默认在后台打开窗口，定向点击并不等于实际激活；后台低频绘制不能用于证明前台动画流畅。通过Finder正常打开应用后，记录窗口的实际active状态再测量。另一处旧文档把定向Cmd+Tab当作系统级切换应用，也已撤回该项结论。

前台补测在暗色切浅色时再次捕获约1001.89ms的present阻塞：该次选择值从2到1.773、1.722后停顿，再落到1，实际耗时约1042ms。这次异常不能算作200ms流畅通过，也不能被随后正常样本抵消。

进一步分段采样在PID 82035、活动窗口空闲40秒后的连续切换中重现：present为1001.306ms，其中`nextDrawable`占1000.567ms；编码0.222ms、提交0.040ms、等待调度0.406ms、事务呈现0.050ms。[原始事件摘录](design-review-2026-09-08/settings-correction/present-stall-before.json)保留了选择的1036.815ms耗时。此前约13ms内已有两次正常的非事务提交，随后备用重绘触发了事务呈现；显示刷新和16ms备用唤醒在争相驱动画面。

`4cf0289`改为按每个窗口的请求代次消费需求：进入回调时消费旧请求，回调内的新动画请求保留；备用唤醒检查实际完成的回调，只在32ms内没有进度时接管。正常显示刷新仍使用自己的频率。没有以延长Metal超时或跳过事务呈现要求来掩盖阻塞。新增7项状态机测试和6项异步执行器测试，通过临时独立Cargo harness原样提取生产定义及测试后[实际运行13项](design-review-2026-09-08/settings-correction/frame-wake-tests.json)，没有另写一套模拟实现。

优化采样版PID 91479重复模式切换、空闲40秒后切换、异步读取及窗口恢复。[最终统计与采样边界](design-review-2026-09-08/settings-correction/fixed-pid91479-final-statistics.json)记录了5062次present，最大8.358ms，没有超过100ms的present；累计绘制p95为7.426ms、最大36.831ms，输入至呈现p95为29.770ms、最大49.054ms。确认活动的模式切换包含11–12个严格中间值，约200–215ms到终态；活动动画呈现间隔p95为18.285ms。设置页的40秒空闲段没有持续绘制。这些是此设备、此工作负载的CPU侧事件，不包含完整OS输入队列、GPU完成和屏幕扫描输出，也不保证所有未来场景不会停顿。

保留另一项边界异常：最小化附近出现542.114ms的失效至呈现等待；失效至绘制完成为541.785ms，其中绘制本身2.787ms，紧随的present为0.316ms。它不属于此次定位的`nextDrawable`阻塞，也不据此推断具体系统原因。恢复窗口后的末尾仍约每秒两次绘制，选择动画事件为空；因此不宣称整个进程一直零绘制，停帧结论只针对已列出的设置空闲及最小化后完成区间。

采样分组以日志中的`window_active_at_sample`为准，它是每秒一次的GPUI窗口活动状态，没有逐帧全局前台应用标记。新应用首次打开后的16组实际是非活动采样，不能沿用进度消息中的“前台16次”；中途失焦的一组也被单独排除。重新激活后的短批次与空闲后的活动样本另行比较。最初尝试通过工具切换Finder来验证后台读取，没有可靠改变该标志，因此只算异步读取完成，不算后台证据。

最后使用真实窗口最小化建立了明确的非活动区间。[读取开始](design-review-2026-09-08/settings-correction/44-fixed-before-minimize.jpg)后最小化，25秒受控提取在不可见期间完成；恢复前工作区文件已含解析标题，活动采样为false且后续绘制停止。[恢复窗口](design-review-2026-09-08/settings-correction/45-fixed-restored-result.jpg)直接显示已读取来源，无需再次读取或缩放窗口。该项证明最小化后的完成与恢复，不扩展为所有遮挡方式的后台呈现验证。

## 提交与交付记录

设计规范、共享选择器及设置布局分别提交为`a9cfa10`、`f2593c1`、`adad47f`；共同开关行宽度为`8d95e08`，不等长选项动画为`32d4fff`，可选分段诊断为`b9df11b`，重绘调度修复为`4cf0289`。最终桌面测试134通过、0失败、3项原有忽略；源补丁准备脚本6项通过；独立执行的调度测试13项通过。

本机`~/Applications/course2md Design Preview.app`已更新为普通优化构建`2.0.0-alpha.1+design.4cf0289c6449`，未开启performance功能或窗口夹具的debug-assertions覆盖。[实际关于页](design-review-2026-09-08/settings-correction/47-installed-about.jpg)确认构建号，代码签名严格验证通过。签名后的桌面二进制SHA-256为`8bd923d373c94884572f23db782bfba00f28860826369c2c1df39fe8424fd640`；随包CLI保持原字节。

旧预览和配置备份在`~/Library/Application Support/course2md-design-preview/backups/2026-09-08-settings-correction/`。用户原有应用偏好在备份、更新、启动及导航后哈希一致；浅色Catppuccin Latte、100%文字大小保留。[最终外观页](design-review-2026-09-08/settings-correction/48-installed-appearance.jpg)已由独立审查者复核，应用停留在此页供用户测试。发布的alpha.1标签、资产及`/Applications/course2md.app`未被替换。本记录撤回的是超出证据的设计验收结论，不用新的测试数量再作全产品设计保证。
