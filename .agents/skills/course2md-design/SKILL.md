---
name: course2md-design
description: Design, implement and review course2md's native GPUI desktop interface. Apply to pages, controls, icons, wording, themes and animation; not to CLI-only or conversion-engine changes.
---

# course2md desktop design

Use a compact desktop adaptation of Material Design 3, with macOS window behavior and the user's chosen Material Icons and configurable color themes. Follow the product rules below. They replace the cream-only, pill-heavy web mockups in older project documents.

## Product hierarchy

- Put 工作台、我的笔记、任务 and 设置 in a single integrated title bar, with icons and a stable selected treatment. Reserve native window-control space and draggable background; buttons must receive clicks immediately.
- Show the brand once at most. A page headline explains the next useful action; do not spend the workspace repeating a large logo.
- Keep each section to one prominent next action. Other actions have outlined or quiet treatments. Destructive actions are clearly labeled and visually separated from forward actions.
- Separate navigation, mode selection, data entry and results. Source tabs sit above the input surface. The video URL has a persistent label, visible border/background, a real input cursor and a useful example; platform logos sit outside the editable area.
- Use YouTube and Bilibili's recognizable brand marks, with their intrinsic colors. Material Icons represent interface actions, never substitute for a platform logo.
- Keep normal wording about the user's task: 自动, 读取视频, 开始生成, 阅读笔记. Put protocols, raw paths, runtimes, model implementation details, technical errors and diagnostic contracts under a clearly labeled details disclosure when they do not affect the current decision.
- A saved note is content to open. Make its row/card visibly clickable and provide a distinct 阅读笔记 action. Use completion status only where a generation/version state actually needs explanation.

## Layout and controls

- Use the shared primitives in `desktop/src/theme.rs` and the theme-aware component library. Never hard-code page colors.
- Project spacing scale: 4, 8, 12, 16, 24, 32, 48 logical pixels. Use 8 between an icon and label; 16–24 inside cards; 24–32 between sections. These are product choices, not claimed platform requirements.
- Body text: 14; supporting text: 12; section titles: 18; page titles: 24–28. Use the system font, regular body and semibold headings. Reserve monospace for technical details. Avoid giant branding headlines and excessive gray microcopy.
- Ordinary controls are 36–40 high with an 8-pixel radius. Cards use 12; source/preview surfaces may use 16. Reserve capsules for compact status badges and segmented selectors.
- Workbench content is centered around 800–920 wide. Libraries may use a wider content region; readers use a comfortable text measure and a separately sized outline. Keep equal outer gutters and align related section edges.
- Labels and inputs form consistent vertical groups. Show validation next to its field and keep the input and recovery action available after failure.
- Every important action and navigation item pairs a consistent 18–20 pixel Material icon with text. Use `desktop/src/icons.rs`. Do not mix emoji, text glyphs and unrelated icon families. Small status badges use 14–16 pixel icons.
- Badges are one padded container holding an icon, a gap and a label. Success/warning/error/progress must use the same semantic tokens on every page. Never draw a colored empty pill beside unstyled text.
- Show clear hover and pressed feedback for clickable rows and controls. Do not make static text look like a button or a disabled control look like an active action.

## Theme contract

- Route all UI colors through semantic tokens: canvas, surface, inset, text, muted text, border, control border, accent, on-accent, success, warning, error and their surfaces.
- Include all chrome, text fields, menus, dialogs, switches, progress tracks, badges, reader highlights and empty states. No white islands in dark themes and no dark-only text on dark surfaces.
- Theme preference supports system, light and dark appearance, with independently saved light and dark palettes. Apply selection immediately; restore it on launch. A failed save must not silently publish a new active preference.
- Built-in families: Paper / Ink, Nord (dark and Snow light adaptation), Tokyo Night / Day, Catppuccin Latte / Frappé / Macchiato / Mocha. Use upstream palettes, document any semantic adaptations, and retain their licenses in the asset provenance.
- Theme preview cards show the actual palette, a miniature content/control sample, name, appearance and a clear selected mark. Color selection is separate from content preferences.

## Motion contract

- Use `desktop/src/motion.rs` for shared transitions. Motion explains a user's action or live work; do not add perpetual decoration.
- Project timing: 120–160 ms hover/press; 200 ms selection or progress retargeting; 240 ms page/state entrance and disclosure; 280 ms theme color interpolation. Use ease-out for entry and smooth retargeting for values that can change mid-animation. Never delay accepting an action until animation finishes.
- Provide actual animated busy indicators during URL reading, subtitle loading, note opening, service testing, model downloads, QR generation, export and migration. Keep layout stable and allow cancellation when supported.
- Animate page/section changes, source confirmation, option expansion, progress changes and result/error notices. Stable IDs must preserve animation state across ordinary rerenders; changing data must not restart every animation.
- Show truthful determinate progress only with real progress data. Otherwise use an indeterminate indicator and a plain status sentence. Completion should settle into a clear result and next action.
- Honor the existing reduce-motion preference by showing final states immediately; do not let it break measurements or hide content. It is a fallback behavior, not the focus of a separate audit.

## Review and verification

For broad UI work, assign independent subagent reviews by page and enumerate subpages, dialogs and exceptional states. Give reviewers the actual running screenshots as well as code. Turn findings into specific fixes; do not mark unvisited states as visually verified.

Verify at a normal desktop size and the supported narrow size, in a light and dark palette; exercise every built-in theme, live switching and restart persistence. Check title-bar drag and controls, input-to-result flows, and animations in the running native build. Use isolated test data for generation, login, storage and destructive flows. Keep the user's installed release and real notes safe during development.

Run appropriate existing tests for configuration persistence and changed behavior. Use a theme coverage check to catch hard-coded UI colors and missing embedded icons, but do not substitute it for visual inspection. Record remaining real limitations honestly. Finish with atomic commits and a runnable reviewed build.

## References and precedence

Primary references (reviewed 2026-09-08):

- [Material Design 3 interaction states](https://m3.material.io/foundations/interaction/states/overview), [color roles](https://m3.material.io/styles/color/roles), [motion](https://m3.material.io/styles/motion/easing-and-duration): semantic roles, consistent state feedback, purposeful transitions.
- [Apple toolbars](https://developer.apple.com/design/human-interface-guidelines/toolbars), [text fields](https://developer.apple.com/design/human-interface-guidelines/text-fields), [motion](https://developer.apple.com/design/human-interface-guidelines/motion): desktop hierarchy, input affordance and native window behavior.
- [Google Material Icons](https://github.com/google/material-design-icons), [Nord](https://www.nordtheme.com/docs/colors-and-palettes), [Tokyo Night](https://github.com/folke/tokyonight.nvim), [Catppuccin](https://github.com/catppuccin/palette): original icon and palette sources.

The user's explicit requirements for Material Icons, application themes and integrated navigation take precedence over a platform reference's preference for SF Symbols or system-only appearance. Do not add glass effects merely because a guideline describes them. These numeric sizes/timings are course2md's desktop adaptation, not a claim of exact Material or Apple defaults.
