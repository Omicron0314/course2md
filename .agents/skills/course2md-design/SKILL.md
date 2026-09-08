---
name: course2md-design
description: Design, implement and review course2md's native GPUI desktop interface. Apply to pages, controls, icons, wording, themes and animation; not to CLI-only or conversion-engine changes.
---

# course2md desktop design

Use a compact desktop adaptation of Material Design 3, with macOS window behavior and the user's chosen Material Icons and configurable color themes. Preserve working layout and interaction from earlier versions unless a concrete problem requires changing them. These rules govern shared colors, controls and motion; they do not authorize replacing a useful composition for stylistic consistency.

For settings, navigation, selection controls or a design acceptance review, also read [the settings contract](references/settings.md). It maps the user's rejected screen to official guidance, shared implementation and observable acceptance criteria. An existing screenshot or passing test count is not a design approval.

## Product hierarchy

- Put 工作台、我的笔记、任务 and 设置 in a single integrated title bar, with icons and a stable selected treatment. Reserve native window-control space and draggable background; buttons must receive clicks immediately.
- Show the brand once at most. A page headline explains the next useful action; do not spend the workspace repeating a large logo.
- Keep each section to one prominent next action. Other actions have outlined or quiet treatments. Destructive actions are clearly labeled and visually separated from forward actions.
- Separate navigation, mode selection, data entry and results. Source tabs sit above the input surface. The video URL has a persistent label, visible border/background, a real input cursor and a useful example; platform logos sit outside the editable area.
- Use YouTube and Bilibili's recognizable brand marks, with their intrinsic colors. Material Icons represent interface actions, never substitute for a platform logo.
- Keep normal wording about the user's task: 自动, 读取视频, 开始生成, 阅读笔记. Put protocols, raw paths, runtimes, model implementation details, technical errors and diagnostic contracts under a clearly labeled details disclosure when they do not affect the current decision.
- The workbench contains the current input for a new note. Do not expose drafts, a draft picker, draft management, draft naming or a discard-draft workflow. Keep current input when navigating; starting a new note resets that form. Submitted tasks retain their own source and settings. Service settings use ordinary edit, save and cancel actions, without a separate draft lifecycle.
- A saved note is content to open. Make its row/card visibly clickable and provide a distinct 阅读笔记 action. Use completion status only where a generation/version state actually needs explanation.

## Layout and controls

- Use the shared primitives in `desktop/src/theme.rs` and the theme-aware component library. Never hard-code page colors.
- Project spacing scale: 4, 8, 12, 16, 24, 32, 48 logical pixels. Use 8 between an icon and label; 16–24 inside cards; 24–32 between sections. These are product choices, not claimed platform requirements.
- Body text: 14; supporting text: 12; section titles: 18; page titles: 28. Use `TEXT_DISPLAY` for the page heading, `TEXT_TITLE` for sections, `TEXT_BODY` for field labels and `TEXT_AUX` for necessary help. Labels use medium/semibold weight; descriptions use regular muted text. Check the actual rendered hierarchy: declaring tokens while calling a section token for the page title is a failure.
- Ordinary controls are 36–40 high with an 8-pixel radius. Cards use 12; source/preview surfaces may use 16. Reserve capsules for compact status badges and segmented selectors.
- Workbench content is centered around 800–920 wide. Libraries may use a wider content region; readers use a comfortable text measure and a separately sized outline. Keep equal outer gutters and align related section edges.
- Settings have their own responsive shell: stable category navigation and a separately scrolling pane. Wide windows use a leading sidebar; narrow windows use one full-width navigation row. Mode/value choices are compact controls within that pane, not category navigation: reuse `SingleChoiceGroup`, with a capsule track and moving selected capsule, and constrain large groups to a useful control width. Do not rebuild them with page-specific button borders or mix rectangular and capsule selections.
- Measure centering against the complete window. Account for a component's built-in padding before adding safe areas; macOS title-bar navigation needs equal left and right reservations. Calculate columns from the actual inner width after gutters and the actual rem-based gap, not a second approximate page width.
- Keep layout in the normal parent/child tree. A shared animation must not override a caller's padding, width or position. Expanded content must have its correct natural height on its first frame and after a width/content change. Do not hide overflow to conceal an incorrect size calculation; long content needs a deliberate wrapping or scrolling path.
- Inspect the component's inner layout when alignment appears ineffective. A Button's centered inner label cannot be left-aligned by changing only the outer flex container. Keep badges to short status labels; place usernames, filenames and explanations in a separately constrained text region.
- Labels and inputs form consistent vertical groups. Show validation next to its field and keep the input and recovery action available after failure.
- Every important action and navigation item pairs a consistent 18–20 pixel Material icon with text. Use `desktop/src/icons.rs`. Do not mix emoji, text glyphs and unrelated icon families. Small status badges use 14–16 pixel icons.
- Badges are one padded container holding an icon, a gap and a label. Success/warning/error/progress must use the same semantic tokens on every page. Never draw a colored empty pill beside unstyled text.
- Show clear hover and pressed feedback for clickable rows and controls. Do not make static text look like a button or a disabled control look like an active action.

## Theme contract

- Route all UI colors through semantic tokens: canvas, surface, inset, text, muted text, border, control border, accent, on-accent, success, warning, error and their surfaces.
- Include all chrome, text fields, menus, dialogs, switches, progress tracks, badges, reader highlights and empty states. No white islands in dark themes and no dark-only text on dark surfaces.
- Theme preference supports system, light and dark appearance, with independently saved light and dark palettes. Apply selection immediately; restore it on launch. A failed save must not silently publish a new active preference.
- Built-in families: Paper / Ink, Nord (dark and Snow light adaptation), Tokyo Night / Day, Catppuccin Latte / Frappé / Macchiato / Mocha. Use upstream palettes, document any semantic adaptations, and retain their licenses in the asset provenance.
- Theme preview cards show the actual palette, its name and a clear selected mark. The section already identifies light/dark appearance; do not repeat it or add poetic color descriptions. Omit a second “current theme” sentence when the grid shows the choice. In system mode, allow editing both saved palettes without temporarily changing the appearance mode.

## Motion contract

- Use `desktop/src/motion.rs` for shared transitions. Motion explains a user's action or live work; do not add perpetual decoration.
- Project timing: 120–160 ms hover/press and opacity entrance; 200 ms selection or progress retargeting; 280 ms theme color interpolation. Disclosures enter at their full natural height with a short fade and close immediately. Do not use cached-height clipping or position offsets that move content beyond its container. Use ease-out for entry and smooth retargeting for values that can change mid-animation. Never delay accepting an action until animation finishes.
- Provide actual animated busy indicators during URL reading, subtitle loading, note opening, service testing, model downloads, QR generation, export and migration. Keep layout stable and allow cancellation when supported.
- Animate page/section changes, source confirmation, option expansion, progress changes and result/error notices. Stable IDs must preserve animation state across ordinary rerenders; changing data must not restart every animation.
- Page fades do not provide selection feedback. Category markers, mode/value selections and theme-card selection need their own transition; hover and pressed feedback must be explicitly styled on unstyled base controls. Keep the same keyed wrapper tree when reduce-motion changes so subsequent controls do not lose state.
- Show truthful determinate progress only with real progress data. Otherwise use an indeterminate indicator and a plain status sentence. Completion should settle into a clear result and next action.
- Honor the existing reduce-motion preference by showing final states immediately; do not let it break measurements or hide content. It is a fallback behavior, not the focus of a separate audit.

## Review and verification

For broad UI work, assign independent subagent reviews by page and enumerate subpages, dialogs and exceptional states. Give reviewers the actual running screenshots as well as code. Turn findings into specific fixes; do not mark unvisited states as visually verified.

Start review from the user's annotated failures and the current design contract, not from an implementer's list of claimed fixes. Evaluate hierarchy, redundancy, grouping, consistency, discoverability and visible state feedback as separate questions from clipping and functional behavior. A claim about animation needs observed intermediate state and final settling; a duration constant or a settled screenshot is insufficient. Record what the evidence cannot establish.

Verify at a normal desktop size and the supported narrow size, in a light and dark palette; exercise every built-in theme, live switching and restart persistence. Check title-bar drag and controls, input-to-result flows, and animations in the running native build. Use isolated test data for generation, login, storage and destructive flows. Keep the user's installed release and real notes safe during development.

Verify default text size first. Open disclosures and dialogs before resizing, use long real content, and inspect the first changed frame as well as the settled state. A screenshot of an empty or collapsed page does not verify its populated or expanded layout. Specialist accessibility and enlarged-text audits are outside this work unless the user requests them.

For a performance complaint, profile an active workload and distinguish main-thread blocking, expensive drawing and delayed presentation. Render methods must not read files, probe disks or perform network work. Move checks to background work, cache results and reject stale results after a path change. Record measured frame/input latency when making performance claims; idle CPU and successful unit tests cannot establish responsiveness.

Run appropriate existing tests for configuration persistence and changed behavior. Use a theme coverage check to catch hard-coded UI colors and missing embedded icons, but do not substitute it for visual inspection. Record remaining real limitations honestly. Finish with atomic commits and a runnable reviewed build.

## References and precedence

Primary references (reviewed 2026-09-08):

- [Material Design 3 interaction states](https://m3.material.io/foundations/interaction/states/overview), [color roles](https://m3.material.io/styles/color/roles), [motion](https://m3.material.io/styles/motion/easing-and-duration): semantic roles, consistent state feedback, purposeful transitions.
- [Apple toolbars](https://developer.apple.com/design/human-interface-guidelines/toolbars), [text fields](https://developer.apple.com/design/human-interface-guidelines/text-fields), [motion](https://developer.apple.com/design/human-interface-guidelines/motion): desktop hierarchy, input affordance and native window behavior.
- [Google Material Icons](https://github.com/google/material-design-icons), [Nord](https://www.nordtheme.com/docs/colors-and-palettes), [Tokyo Night](https://github.com/folke/tokyonight.nvim), [Catppuccin](https://github.com/catppuccin/palette): original icon and palette sources.

The user's explicit requirements for Material Icons, application themes and integrated navigation take precedence over a platform reference's preference for SF Symbols or system-only appearance. Do not add glass effects merely because a guideline describes them. These numeric sizes/timings are course2md's desktop adaptation, not a claim of exact Material or Apple defaults.
