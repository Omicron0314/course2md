---
name: course2md-design
description: Design, implement and independently review course2md's native GPUI task flows, first use, pages, shared controls, wording, themes and motion. Not for CLI-only or conversion-engine work.
---

# course2md desktop design

Use Material Design 3 as the common visual and interaction vocabulary. Use macOS conventions for window controls, dragging, menus and keyboard behavior. The product uses Material Icons and configurable palettes; it does not reproduce every Material mobile dimension or Expressive animation. Read [the design system](references/system.md) before changing shared controls or composition, and [settings](references/settings.md) when touching preferences.

For first use, conversion, navigation or recovery, read [task decisions and continuity](references/interaction.md). The interface should remove work from the user's task; visual consistency does not justify keeping a needless step.

## Design before patching

Start with the user's task, the complete screen, and its relationship to other screens. An arrow on a rejected screenshot identifies a problem, not necessarily the best implementation. Explain a design choice by the information or action it serves. Do not add concepts, empty cards, fixed heights or extra instructions just to satisfy visual symmetry.

Trace a realistic intent through to a usable result before choosing screens. Separate decisions only the user can make from facts the app can detect, defaults it can reuse, and operations it can continue automatically. A pause needs an unresolved choice or actionable obstacle, not merely the completion of an internal stage. Do not turn the page order from a previous implementation into a required workflow.

Define the hierarchy and shared alignment before individual dimensions. A screen needs a recognizable primary task, related information grouped together, and secondary controls that do not compete with the main action. Whitespace separates groups; it must not scatter a small amount of related information across a large card. Keep the reading position of a task panel stable as its steps change. A border needs a purpose such as input, selection or containment of an independently interactive object. Avoid stacking a divider, heading and bordered card for every section.

Use the existing shared implementation as the place to correct inconsistencies. Reusing a function name is insufficient if pages override its height, padding, shape or selected state. Check actual inner component defaults and the full painted bounds of interactive states, not just the builder's outer style or layout rectangle.

## Product structure

- The workbench turns a supplied video into a readable note. Let the app resolve source metadata, choose suitable defaults and carry out the requested work without demanding separate approval of routine internal stages. Do not ask people to create, name, manage or discard an empty note/draft. Navigating to import preserves current input; submitted tasks own their data independently.
- Show options when they affect a present decision. Automatic language and local capability selection are ordinary defaults; explicit source, model or service choices remain authoritative. An automatic route and an explicitly pinned route can have different recovery behavior. Do not require configuration for a capability the current task will not use.
- First-use guidance helps people perform a real task with a small set of consequential choices, a useful recommendation and optional detail. It can be skipped and found again. Reuse established configuration on later runs. Keep optional services, appearance customisation and technical diagnostics out of mandatory setup.
- The library presents generated content. A truly empty library has one useful import action; search/filter failure states keep the controls needed to recover. Do not show an entire organisation toolbar before there is content to organise.
- The title bar provides stable navigation. It stays quieter than a page's forward action. Peer views may share capsule visuals while keeping tab semantics. A sidebar uses leading-aligned items, not a rotated horizontal track.
- Navigation identifies location; content headings describe useful groups. Keep a contextual return when global navigation cannot restore the current input, document or task. Reusing a selection control must not erase navigation activation or return behavior.
- Put model implementation, protocols, raw paths and technical diagnostics in clearly labelled details. Keep actionable errors close to their cause. Keep user data, notes, credentials and release installations safe during verification.

## Shared implementation

- Colors, typography, control geometry, states and motion live in `desktop/src/theme.rs`, `choice_group.rs`, `motion.rs` and reusable settings rows. Use semantic palette tokens, never page-local colors (except official platform brand marks and actual palette previews).
- Ordinary controls share a 40px-at-default-size baseline, growing with the interface font. Single-choice tracks include their padding in that height. Buttons, inputs and selectors in one toolbar have the same actual bounds and baseline. Content cards and multiline editors explicitly use natural height.
- Use rounded action controls and capsule choices consistently. Cards use 12px corners; larger content surfaces may use 16px. Shape follows role; making every containing rectangle a pill is not consistency.
- Typography at default scale: page 28px, section 18px, field/action 14px, necessary supporting text 12px. Medium labels and regular supporting text. Apply these to the actual rendered text, including component-internal labels. Short headings and labels have no terminal period; necessary explanatory sentences remain normal prose.
- Use 4/8/12/16/24/32/48px spacing. Keep icon and label together with an 8px gap. Use consistent Material icons from `icons.rs`; platform marks use the real YouTube/Bilibili assets.
- Selected, hover, pressed and focus are layers. Selection remains visible under pointer feedback and keyboard focus; no hover rule may replace it with an unselected background. Reserve the strongest filled accent for the principal action; peer selections use the shared tonal surface.
- Size from the actual parent content width and current type scale. Reflow groups or wrap long values before labels collide. Do not conceal bad measurement with clipping. Readable control widths belong within a common form grid, not an unrelated truncated subcolumn.

## Themes and motion

Preserve all built-in palettes, their provenance and independent light/dark preferences. Map their colors to the same readable text, supporting text, action, selection and status roles; a palette's upstream foreground is not automatically suitable for the app's body text. Theme previews show the application's actual mapped colors and a clear current choice. Choosing an inactive appearance's preset must not change the appearance mode. Failed saves keep a visible recovery action and do not claim persistence.

Use one principal animated response per intent. A selector moves its indicator; a theme change interpolates colors; a busy task shows live progress. Do not simultaneously fade the entire page, rearrange its controls and move a selector to advertise one click. Keep selection labels stationary, start and settle smoothly, and accept retargeting immediately. Existing reduce-motion preferences show the final state directly. Real busy states use determinate progress only when progress data exists.

## Independent review and delivery

For a broad change, assign independent subagent review to every affected page, including shared-component callers, overlays and loading/empty/failure/completion states. Reviewers start from the original rejected screens and this system, not a list of claimed fixes.

First follow the main task and a repeat use through the running native app: record user decisions, required actions, detours, recovery and the usable result. Then compare complete screens for hierarchy, grouping, rhythm, density, coherence and discoverability, followed by geometry and control behavior. Do not equate “no overflow”, compilation, tests or an animation duration with good design. Record concrete findings and their resolution, and distinguish screenshot evidence from source inspection and runtime observation.

Verify normal/narrow/wide native windows, light/dark palettes and every theme. The product's offered 100/125/150/200% text scales are normal layout cases: controls grow and content reflows. This is not authorization for a specialist accessibility audit. Keep ordinary keyboard navigation and existing system preferences.

Test changed behavior meaningfully. Profile if a performance defect is present; do not turn each design review into an unrelated performance programme. Never perform filesystem/network work during rendering. Use isolated fixtures for failing saves, generation and destructive flows. Finish with atomic commits, candid evidence and a runnable reviewed preview; do not alter published tags or replace release assets for an ordinary UI change.
