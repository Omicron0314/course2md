---
name: course2md-design
description: Design, implement and independently review course2md's native GPUI pages, shared controls, wording, themes and motion. Not for CLI-only or conversion-engine work.
---

# course2md desktop design

Use Material Design 3 as the common visual and interaction vocabulary. Use macOS conventions for window controls, dragging, menus and keyboard behavior. The product uses Material Icons and configurable palettes; it does not reproduce every Material mobile dimension or Expressive animation. Read [the design system](references/system.md) before changing shared controls or composition, and [settings](references/settings.md) when touching preferences.

## Design before patching

Start with the user's task, the complete screen, and its relationship to other screens. An arrow on a rejected screenshot identifies a problem, not necessarily the best implementation. Explain a design choice by the information or action it serves. Do not add concepts, empty cards, fixed heights or extra instructions just to satisfy visual symmetry.

Define the hierarchy and shared alignment before individual dimensions. A screen needs a recognizable primary task, related information grouped together, and secondary controls that do not compete with the main action. Whitespace separates groups; it must not scatter a small amount of related information across a large card. A border needs a purpose such as input, selection or containment of an independently interactive object. Avoid stacking a divider, heading and bordered card for every section.

Use the existing shared implementation as the place to correct inconsistencies. Reusing a function name is insufficient if pages override its height, padding, shape or selected state. Check actual inner component defaults, not just the builder's outer style.

## Product structure

- The workbench is a continuous video import form: choose source, enter/select video, read it, then configure generation. Do not ask people to create, name, manage or discard an empty note/draft. Navigating to import preserves current input; tasks own their submitted data independently. Saved generated notes remain the output users read.
- Keep the initial form limited to decisions needed before reading. Automatic language handling is the normal path; explicit subtitle-track choice belongs with the source after reading, with advanced preferences in Settings.
- The library presents generated content. A truly empty library has one useful import action; search/filter failure states keep the controls needed to recover. Do not show an entire organisation toolbar before there is content to organise.
- The title bar provides stable navigation. It stays quieter than a page's forward action. Peer views may share capsule visuals while keeping tab semantics. A sidebar uses leading-aligned items, not a rotated horizontal track.
- Navigation already identifies the selected category. Content headings describe useful groups; avoid repeated category headings and redundant return actions when global navigation remains available.
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

Preserve all built-in palettes, their provenance and independent light/dark preferences. Theme previews show actual colors and a clear current choice. Choosing an inactive appearance's preset must not change the appearance mode. Failed saves keep a visible recovery action and do not claim persistence.

Use one principal animated response per intent. A selector moves its indicator; a theme change interpolates colors; a busy task shows live progress. Do not simultaneously fade the entire page, rearrange its controls and move a selector to advertise one click. Keep selection labels stationary, start and settle smoothly, and accept retargeting immediately. Existing reduce-motion preferences show the final state directly. Real busy states use determinate progress only when progress data exists.

## Independent review and delivery

For a broad change, assign independent subagent review to every affected page, including shared-component callers, overlays and loading/empty/failure/completion states. Reviewers start from the original rejected screens and this system, not a list of claimed fixes.

First compare complete native screens for hierarchy, grouping, rhythm, density, coherence and discoverability. Then check geometry and behavior. Do not equate “no overflow”, compilation, tests or an animation duration with good design. Record concrete findings and their resolution, and distinguish screenshot evidence from source inspection and runtime observation.

Verify normal/narrow/wide native windows, light/dark palettes and every theme. The product's offered 100/125/150/200% text scales are normal layout cases: controls grow and content reflows. This is not authorization for a specialist accessibility audit. Keep ordinary keyboard navigation and existing system preferences.

Test changed behavior meaningfully. Profile if a performance defect is present; do not turn each design review into an unrelated performance programme. Never perform filesystem/network work during rendering. Use isolated fixtures for failing saves, generation and destructive flows. Finish with atomic commits, candid evidence and a runnable reviewed preview; do not alter published tags or replace release assets for an ordinary UI change.
