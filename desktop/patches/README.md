# Reviewed GPUI source patches

`scripts/sources.py --locked` checks these patches against the pinned source
revisions, then applies them to the generated `.deps` worktrees. Repeating source
preparation preserves the same result. Edits inside or outside a patch are
checked in a disposable copy and rejected without changing the original files.
Developer checkouts are never patched.

- `zed-input-accessibility.patch` adds an accessibility-only focus delegate for
  controls whose semantic frame and editing engine are separate elements. It
  does not register another keyboard dispatch target or Tab stop.
- `component-input-accessibility.patch` connects each input's semantic frame to
  the actual editor focus, exposes ordinary text runs and directional selection,
  handles native selection updates, reports read-only/disabled states, and skips
  disabled inputs in Tab navigation. Masked/password values remain excluded.
- `zed-macos-window.patch` forwards native window focus to AccessKit, initializes
  the adapter from the window's existing key state, and requests an AppKit draw
  when GPUI is invalidated. This lets initial drawing proceed when a display-link
  callback has not started. During a frame callback it leaves animation demand
  to the display link, avoiding an AppKit redraw loop that starves input and
  asynchronous results while a spinner is visible.
- `component-tooltip-lifecycle.patch` dismisses a window's managed tooltip
  before mouse or keyboard navigation can remove its trigger. It also cancels
  delayed tooltips, while preserving normal hovering and other windows.

When upgrading the locked revisions, review upstream changes before adjusting a
patch. Do not edit `.deps` to fix an application build. The preparer deliberately
stops if a patch no longer applies or finds unrecognized source modifications.

Validation:

```sh
python3 -m unittest discover -s desktop/scripts -p test_sources.py
python3 desktop/scripts/sources.py --locked
cargo check --manifest-path desktop/Cargo.toml
```

The patches contain regression tests for semantic focus delegation, an actual
accessibility-enabled input draw with one Tab stop, Unicode and
line-break offsets, reversed selection, and rejecting stale native text ranges.
Native AX/VoiceOver verification still requires running the desktop application.
