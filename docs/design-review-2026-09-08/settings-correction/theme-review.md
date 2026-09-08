# 主题与共享选项独立复审

日期：2026-09-08。审查者：`settings_shell`。依据 [course2md-design](../../../.agents/skills/course2md-design/SKILL.md) 及[设置验收约定](../../../.agents/skills/course2md-design/references/settings.md)，检查用户指出的空间利用、导航、层级、重复文字和可见状态反馈。

本记录仅描述此审查者的覆盖：逐张查看下列 21 张原生应用截图，并只读审查 `32d4fff` 对默认 `SingleChoiceGroup` 的影响。归档图片与当时查看的 `/tmp/course2md-settings-2026-09-08/` 原图逐一校验一致；不将其他审查者的操作、测试或截图覆盖计入本记录。

## 原生截图观察

| 逐张查看的截图 | 实际观察与结论 |
| --- | --- |
| [Paper](theme-Paper.jpg)、[Nord Snow](theme-Nord-Snow.jpg)、[Tokyo Day](theme-Tokyo-Day.jpg)、[Catppuccin Latte](theme-Catppuccin-Latte.jpg) | 四套浅色主题中，宽窗四张卡片一行完整呈现，名称及选中勾可见。页面标题、分组、字段和帮助文字有层级；类别标记和应用顶部的设置选中态可辨。未见明显裁切、错边或新增静态设计问题。 |
| [Ink](theme-Ink.jpg)、[Nord](theme-Nord.jpg)、[Tokyo Night](theme-Tokyo-Night.jpg)、[Frappé](theme-Catppuccin-Frappé.jpg)、[Macchiato](theme-Catppuccin-Macchiato.jpg)、[Mocha](theme-Catppuccin-Mocha.jpg) | 六套深色主题均呈三列两行，完整名称及选中勾可见，卡片与分组边缘一致。未见明显错误的明暗色块或文字消失。十套截图均未出现重复的“当前主题”、配色形容词或自动保存叙述。 |
| [06：宽窗外观](06-appearance-wide-light.jpg) | 左侧导航与右侧内容形成独立区域，主标题突出，四张浅色卡片完整横排；没有工作台窄列造成的大面积无效留白。截图本身不能证明滚动行为。 |
| [08：跟随系统双配色](08-system-both-palettes-wide.jpg)、[09：双预设](09-system-presets-retained.jpg) | 两张均显示“跟随系统”选中，同时提供浅色和深色配色。08 的标记分别为 Latte、Tokyo Night，09 为 Paper、Nord。只能确认拍摄时的同时选中状态，不能据文件名证明保存或重启恢复。 |
| [10：860 窄窗初版](10-appearance-narrow-light.jpg) | **P2：空间利用不足。** 浅色卡片只有两列，每张约 400px；第二行仅露出预览上缘，当前 Latte 的名称和勾在首屏之外。问题是浏览与选中状态发现成本，而非单纯越界。 |
| [20：860 窄窗修正](20-final-appearance-narrow-light.jpg) | **上述 P2 关闭。** 四张浅色卡片一行展示，完整的 Catppuccin Latte 名称和勾均在首屏内，没有明显拥挤或截断。底部只露出“界面偏好”标题；不代表整个外观页无需滚动。 |
| [24：1050 紧凑导航](24-compact-1050-navigation.jpg) | 五项类别保持单行，导航用满可用宽度，左右外缘与下方卡片网格一致；四张浅色卡片及名称完整可见。之前担心的中间宽度额外收窄、外缘跳变在此截图中未出现。 |
| [25：宽窗悬停](25-palette-hover-wide.jpg) | Nord Snow 有轮廓和阴影强调；Latte 仍保留唯一选中勾与浅色强调底部，悬停与选中可以区分。父代理提供的操作记录为从空白拖入 Nord Snow，AX 确认当前仍为 Latte；本图不作为按压中间帧或过渡时序的证据。 |
| [32：最终外观](32-final-appearance-active.jpg) | 配色勾及模式控件状态轮廓可见，无新增静态层级或对齐问题。不能仅凭文件名或轮廓判定正在按压；外观与字号采用 `full_width`，不属于本轮默认分支调用点验证。 |
| [33：生成页顶部](33-final-generation-top.jpg)、[34：生成页下部](34-final-generation-controls.jpg) | 字幕语言、识别位置和本地模型的选中胶囊清楚，分别可见“自动”“自动”和 Qwen3-ASR 1.7B 的选中态；未见文本裁切或静态错位。字段标签与帮助文字有区分。34 已滚至内容下部，类别导航与当前面板标题仍可见；不由单帧推断连续滚动或动画表现。 |
| [35：服务列表](35-final-services-wide.jpg) | 语音服务、AI 服务、来源账号的分组标题及图标层级一致。截图含 AI 服务条目及 Bilibili 未登录卡片，未见新增实质设计问题。画面是列表，没有打开服务编辑器。 |

## 11 个默认调用点的源码审查

审查 [choice_group.rs](../../../desktop/src/choice_group.rs)、[import_ui.rs](../../../desktop/src/import_ui.rs) 和 [settings_ui.rs](../../../desktop/src/settings_ui.rs)。按生产调用位置计数，共 11 处；共享服务选择器在多个页面复用，仍算一个调用位置。

| 调用位置 | 数量 | 审查要点 |
| --- | ---: | --- |
| 来源确认：`import-text-source` | 1 | 动态字幕使用 `track:<id>`，语音使用 `speech`；加载或缓存提示改变标签时，选项身份保持稳定。选择与确认回调未改变。 |
| 工作台／来源选项：`import-speech-location`、`import-local-engine` | 2 | 识别位置和按环境过滤的模型保留稳定值及当前选项，回调未改变。 |
| 生成设置：`default-subtitle-language`、`default-asr-device`、`default-asr-hardware`、`default-local-model` | 4 | 选中值、平台过滤及自定义值处理沿用现有逻辑；失败保存未因颜色过渡提前发布新选中偏好。 |
| 工作台与设置共用：`service-choice` | 1 | 组 ID 区分用途和当前任务／默认设置，选项使用服务版本 ID；当前停用服务保留及绑定回调未改变。 |
| 服务编辑器：`service-protocol`、`service-auth-mode`、`service-test-purpose` | 3 | 协议过滤、待绑定和测试忙碌时的禁用条件、选中值及回调未改变。 |

未发现可归因于该改动的选中状态、字体或布局回归。颜色过渡键由组 ID 与选项值组成，所查调用点未见冲突；默认分支统一使用 `MEDIUM`，选择切换不再改变字重。原有内容宽度、内边距、高度和换行策略没有改为等宽。焦点、tab stop、禁用状态仍由原有控件逻辑驱动。外观模式和字号两处 `full_width` 调用不计入上述 11 处。

## 证据边界

本次复审未操作 GUI、运行 Cargo 或修改源文件。静态图片没有覆盖颜色过渡中间帧、按压时序、快速重定向和最终停止请求帧，也不能证明保存失败恢复或重启持久化。

工作台、来源确认、长字幕／服务名称、展开的高级硬件与 NPU 模型列表，以及服务编辑器的协议、认证、测试和忙碌状态，本记录只有源码复审，没有当前原生画面验证。默认组选项原有换行策略仍存在；未据源码将这些长内容状态判为视觉通过。32–35 的静态复核也不替代默认选择前后边界稳定性或动画测试。结论限定于所列证据，不构成全产品或全部交互状态的设计验收。
