# Settings composition

Read with [the shared system](system.md). Earlier 520px subcolumns, repeated category titles and special side/underline markers were local decisions that produced an incoherent screen. They are superseded by this composition.

## Layout

Use a leading category sidebar when the current window and text scale leave a useful content pane; otherwise use a compact single-row category control. Keep navigation outside content scrolling. Reuse capsule selection and tab behavior rather than drawing a page-specific underline, side stripe or bordered selected button.

Within the content pane, begin with meaningful groups such as “关于” or “运行检查”. Do not repeat the category name directly above them. A form group has one label/control alignment system: its outer edges align with the other groups, while a selector has a useful bounded width inside it. A wide pane is not permission to scatter labels and values to opposite extremes.

## Appearance

Appearance mode and the independently saved light/dark palettes are different choices. Show stable light and dark preset previews; open a clearly titled palette chooser containing actual previews. The number of available palettes must not move unrelated settings when the appearance mode changes. Preserve all ten palettes and all existing persisted values without introducing a new theme-family lifecycle.

Use the same label/control row for mode, font size and the motion switch. A chooser calculates columns from its actual inner width and the current label size. Long palette names remain readable. Immediate changes and persistence failure must be visible in the chooser; failure must not silently close it or claim the new choice is saved.

## Application and diagnostics

Group product identity/version, related actions and licenses in reading order. Keep actions near their information. Use compact capability/status rows; place program, model and device details in distinct subordinate groups when expanded. Be explicit about the difference between an available runtime and a model that has actually been loaded. Do not show irrelevant platform internals at the same level as the user's next action.

## Native evidence

Check the whole initial screen before scrolling, then expanded diagnostic/service/model states. Compare normal, narrow and wide windows at default and enlarged text. Inspect selected + hover, selected + pressed and selected + keyboard focus separately. A static selected screenshot does not cover their interaction.

Selection movement and theme-color interpolation should read as one response; avoid category/page re-entry fades and changing card-grid height on the same click. Record what native observations establish and what remains unvisited. Do not describe runtime timing, tests or screenshots alone as blanket design acceptance.
