# 课程库页面 高保真静态 mock

纯 HTML/CSS（无框架无构建），用于在浏览器里校准布局，之后原样复刻到 GPUI。

## 打开方式

浏览器直接打开 `index.html`，用 URL query 切换配置：

- `?view=list`（默认）/ `?view=cards`
- `?scale=1`（默认）/ `1.25` / `1.5` / `2`

例：`index.html?view=list&scale=2` 模拟 200% 用户字号下的列表视图。

## rem ↔ GPUI 映射规则（核心）

| mock (CSS) | GPUI | 语义 |
|---|---|---|
| `html { font-size: calc(14px * var(--scale)) }` | 用户字号倍率 | 100% 时 1rem = 14px，200% 时 1rem = 28px |
| `rem` 尺寸 | `rems()` | **承载文字或随文字呼吸的结构尺寸**：侧栏宽 `14.857rem`、封面 `6.857×3.857rem`、导航项 `min-h 2.25rem`、控件 `min-h 2.6rem`、紧凑按钮 `2rem`、卡片列宽 `minmax(14rem, 1fr)`、chip `max-width 16rem`、全部字号 |
| `px` 尺寸 | `px()` | **永不缩放**：1px 发丝边框、padding、gap、圆角 6px、固定图标（导航 20px、文件夹 18px） |

事故对照：旧 GPUI 实现把侧栏写死 208px、封面写死 96×54px，200% 字号下文字翻倍容器不变，长文件夹名被挤成竖排。本 mock 里这些尺寸全部 rem 化，200% 时侧栏变 416px、封面变 192×108px，各行仍为 2 行截断。

## 组件 → GPUI 结构要点

- **侧栏**：固定宽 `rems(14.857)` + `flex_shrink_0`，`p(12px)`，右侧 1px 发丝线；`vstack`，底部「设置」用 `mt_auto` 类占位钉底。导航项单行省略号；文件夹行 = 图标(18px 固定) + 名称（`flex_1 min_w_0`，**最多 2 行**截断）+ 计数（灰、nowrap、`flex_shrink_0`）。
- **工具行**：单行 `flex` + 允许 `flex_wrap`；搜索框 `flex_1` + `min_w`；分段控件选中项蓝底白字，选中态跟随当前视图。
- **列表行**：单行 `flex`，**禁止 wrap**；封面 `flex_shrink_0`；标题区 `flex_1 min_w_0`（标题 2 行截断、meta 单行省略）；chip `flex_shrink` 允许收缩、`max_w` 取 `min(rems(16), 行宽的 26%)`；⋯ 钮 `flex_shrink_0`。
- **卡片网格**：`repeat(auto-fill, minmax(14rem, 1fr))`；卡片 `overflow hidden`，封面 16:9；底部行 chip `flex_1 min_w_0` + ⋯。
- **文件夹 chip（两种视图共用）**：📁 + 单行省略标签 + ▾。行宽紧张时不能只靠 `min_w_0` —— 见下。
- **窄内容区降级（内容区 < 24rem，GPUI 里按主区实际宽度判断，不是窗口宽）**：
  1. chip 隐藏标签，只剩 📁▾；
  2. 列表封面降档为 `3.429×1.929rem`（仍为 rem）；
  3. 行内 gap 12px → 8px。
  mock 用 CSS container query 实现；GPUI 没有 container query，需要在布局时量主区宽度手动切换这三条规则。

## 截图自查

无头 Chrome 生成（不提交，在 `/tmp/ux-mock-shots/`）：

```
1280×800: list-1x / list-2x / cards-1x / cards-2x
860×620 : narrow-list-1x / narrow-list-2x / narrow-cards-2x
```

生成命令（例）：

```sh
"/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" \
  --headless=new --disable-gpu --hide-scrollbars --force-device-scale-factor=1 \
  --window-size=1280,800 --screenshot=/tmp/ux-mock-shots/list-1x.png \
  "file:///…/docs/ux-mock/index.html?view=list&scale=1"
```
