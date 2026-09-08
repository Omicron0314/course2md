# Settings: design decisions and acceptance

Read this for settings, category navigation, single-choice controls and design acceptance. The user rejected the 2026-09-08 appearance screen even after two internal acceptance claims. The missing standard was product design quality, not merely valid layout rectangles.

## Source → product decision

| Official guidance, checked 2026-09-08 | course2md decision |
| --- | --- |
| [Apple Settings](https://developer.apple.com/design/human-interface-guidelines/settings): category navigation stays visible, indicates the active pane, and titles identify the pane. | Keep settings categories outside content scrolling. The app uses an embedded settings page at the user's request; Apple's separate settings-window convention is not a requirement for this product. |
| [Apple Sidebars](https://developer.apple.com/design/human-interface-guidelines/sidebars): use a leading sidebar for peer areas; compact navigation can suit limited space. | At 1100 logical pixels and above, use a 200px sidebar, 32px gutter and a content pane, within a 1200px shell with equal 24px outer gutters. Below this, show all five categories in one full-width row. These numbers are local choices. |
| [Material segmented buttons](https://m3.material.io/components/segmented-buttons/guidelines): short labels, one row, and restrained padding on wide screens. | Appearance mode and font scale share an equal-segment capsule. Limit the control group to 520px; the user's full-width navigation annotation is not a request to stretch every form control across the entire wide pane. Do not wrap a segmented control into multiple rows. |
| [Apple Typography](https://developer.apple.com/design/human-interface-guidelines/typography): size, weight and color convey information hierarchy. | A 28px page heading, 18px section heading, 14px medium field label and 12px regular help must look distinct at default size. Never demote a field label to helper text to make it fit. |
| [Apple Writing](https://developer.apple.com/design/human-interface-guidelines/writing): concise labels, useful help, and errors near their cause. | Keep palette names and actual previews. Remove repeated “current theme”, color adjectives and obvious auto-save narration. Keep failure/retry feedback below the fixed pane heading, outside content scrolling; never hide it above an already-scrolled grid. Application preferences belong to both Appearance and Application panes, so avoid duplicate warnings or a misleading cross-category link. |
| [Material states](https://m3.material.io/foundations/interaction/states/overview) and [Apple Motion](https://developer.apple.com/design/human-interface-guidelines/motion): visible, consistent feedback; motion explains changes. | The current location uses a persistent accent shape and semibold label. Errors get a separate marker. Selection has an observable intermediate position/color and settles promptly; hover/pressed states exist on every base Tab/Radio. |

The project uses classic Material selection/layout principles with macOS behavior. It does not claim to reproduce the latest M3 Expressive components, glass treatment or spring parameters. The skill's 160/200/280ms timings and geometric constants are course2md choices, not quoted platform defaults.

## Concrete acceptance

- Compare 860×620, 1140×820 and a wide desktop window at default text size, in light and dark themes. Review the whole initial screen before scrolling. A large monitor must not display settings as a workbench-width island.
- At every size, the current app page and settings category are immediately identifiable. Scroll to the end of a long category: all category navigation remains available. No two-line capsule navigation, clipped label or shifted shared edge.
- The settings title is visually above section titles, field labels and helper text in hierarchy. A label and its toggle form one bounded row; the control is not pushed to the far edge of an unrelated wide pane.
- Appearance mode and font scale have the same track, selected shape, padding and feedback. Selected and keyboard-focused are distinct states. Hovering an unselected control must not make it appear selected.
- Theme cards contain preview, name and selection mark. Both stored presets can be chosen while staying in system mode. Repeated clicks on the current choice do not save again or restart feedback.
- Observe mode, font-scale and category marker transitions, card selection and theme color changes. Record an intermediate state and settling, including rapid retargeting. Do not use a page fade as evidence that the selector animates. The existing reduce-motion behavior remains functional.
- Verify save failure near the edited control, with a visible retry path. A failed selection must retain the previously saved active preference and must not falsely show the new choice as saved.
- An independent reviewer receives the actual screen and this contract, not a checklist prefilled with “fixed”. Record findings by screen/state and link their observed resolution. Missing runtime states remain explicitly unverified.

UI performance tests establish timing, functional tests establish behavior, and native screenshots establish the photographed layout. None alone establishes all three or grants blanket product design approval.
