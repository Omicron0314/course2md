# Official Material references and project adaptation

Acquired from official sources on **2026-09-09**. The selected M3 website content belongs to the site's published **2026-09-02_06-10-10** build; individual upstream edit dates are in the manifest. This package supports offline implementation decisions. It is not a complete Material site mirror and does not establish that the application already complies.

## What is actually saved

| Representation | Contents | How to use |
| --- | --- | --- |
| Full official article text | Ten relevant Google documentation articles or repository Markdown documents, listed below | Read the relevant original when changing the corresponding component; HTML articles are converted to Markdown text, with tables, code, captions and links |
| Short M3 quotations | Sixteen relevant M3 topic pages, each with a short verified verbatim quotation | Confirms the provenance of the inspected page; use the topic notes below and project references for implementation |
| Licence evidence | Official Apache licence copies, the Android documentation-licence clause, and recorded licence URLs | Distinguishes the right to redistribute each source from the project's own rules |
| Provenance manifest | [sources.json](material/sources.json) | Original/public data URLs, version or commit, UTC capture date, upstream update date where supplied, upstream SHA-256, saved SHA-256, representation and licence |

The full article archives contain real readable content, not navigation-only HTML or a “requires JavaScript” shell. Text extraction removes website chrome, scripts and separately marked AI-generated page summaries. Images/videos are **not bundled**; supplied captions, alt text and source links are retained. Original repository Markdown retains its media links. Therefore the archive preserves the complete selected article's authored text, not a visual facsimile or all linked assets.

### Licensing and attribution

Google's [Android Content License](https://developer.android.com/license) assigns documentation and included code to Apache-2.0 unless otherwise noted; it separately assigns other site content to CC-BY-2.5. The captured Android pages here are documentation. The relevant clause is [saved locally](material/licenses/android-content-policy-excerpt.md). The repository documents carry the official [Material Android](material/licenses/material-android-license.md) or [Material Web](material/licenses/material-web-license.md) Apache licence. The Google Fonts guide has its own CC-BY-4.0 footer, governed by [Google Developers Site Policies](https://developers.google.com/terms/site-policies); its archive includes attribution and modification notice. Trademarks and linked media are not inferred to share the documentation licence.

The inspected M3 website's shipped footer links [Google Terms of Service](https://policies.google.com/terms). No explicit open licence for its prose was verified in the fetched bundle/page data. Calling the design system “open-source” does not by itself license all website prose or illustrations. Consequently the M3 captures retain only short quotations plus provenance; the complete M3 JSON, page text, screenshots, videos and illustrations are not redistributed here. The hashes identify the actual official content inspected, without asserting permission to republish it.

Google / Material Design and the Android Open Source Project are the original publishers. The adapted rules in `system.md`, `layout-and-type.md`, `states-and-motion.md`, `settings.md` and `interaction.md` are course2md's decisions. This adaptation does not imply Google endorsement.

The CC-BY-4.0 legal-code endpoint returned HTTP 403 when fetched, so this package retains the licence URL and attribution rather than claiming a downloaded legal-code copy. This does not affect the separately fetched and licensed Google Fonts article.

## Complete licensed official documents

| Topic | Offline original text | Original source |
| --- | --- | --- |
| Layout, alignment, containment, visual priority | [Layout basics](material/offline/android-layout.md) | [Android design documentation](https://developer.android.com/design/ui/mobile/guides/layout-and-content/layout-basics) |
| Grids, spacing and flexible columns | [Grids and units](material/offline/android-grids.md) | [Android design documentation](https://developer.android.com/design/ui/mobile/guides/layout-and-content/grids-and-units) |
| Material 3 type roles, theme roles and shape | [Material 3 in Compose](material/offline/android-material3.md) | [Official M3 implementation guide](https://developer.android.com/develop/ui/compose/designsystems/material3) |
| Icons, axes and optical size | [Material Symbols guide](material/offline/google-symbols.md) | [Google Fonts documentation](https://developers.google.com/fonts/docs/material_symbols) |
| States and feedback | [Material Web ripple](material/offline/web-ripple.md) | [Official Material Web source](https://github.com/material-components/material-web/blob/main/docs/components/ripple.md) |
| Motion schemes, easing and transitions | [Material Android motion](material/offline/android-motion.md) | [Official Material Android source](https://github.com/material-components/material-components-android/blob/master/docs/theming/Motion.md) |
| Progress and actual measurements | [Progress indicators](material/offline/android-progress.md) | [Android M3 component documentation](https://developer.android.com/develop/ui/compose/components/progress) |
| Cards | [Card](material/offline/android-card.md) | [Android M3 component documentation](https://developer.android.com/develop/ui/compose/components/card) |
| Lists and content slots | [Material Web lists](material/offline/web-lists.md) | [Official Material Web source](https://github.com/material-components/material-web/blob/main/docs/components/list.md) |
| Dialogs | [Dialog](material/offline/android-dialog.md) | [Android M3 component documentation](https://developer.android.com/develop/ui/compose/components/dialog) |

The repository captures are pinned to commits recorded in their headers/manifest, rather than an unversioned local copy. Android/Web examples explain official component concepts; they are not instructions to add those frameworks to this Rust/GPUI app. Do not copy platform-specific widget APIs or mobile navigation into the desktop merely because they appear in an archived example.

## Inspected M3 principles

These are concise paraphrases of the inspected official topic pages, followed by explicit project use. The short originals and their evidence are linked; exact project measurements live in the project references.

### Layout and grouping

- [Layout overview](material/excerpts/m3-layout.md): the grid, bars/rails and panes organize hierarchy and key actions across available sizes. Project use: define the actual content pane and align its title, first group and actions before styling controls.
- [Grids and spacing](material/excerpts/m3-grids.md): global rulers provide recurring anchors; consistent spacing/sizing and proximity group related elements. Explicit containment and implicit grouping are both available. Project use: common icon/text/control columns, tighter internal gaps than group gaps, and a boundary only when it explains a relationship.
- [Canonical layouts](material/excerpts/m3-canonical-layouts.md): feed, list-detail and supporting-pane arrangements are starting structures that adapt. Project use: choose from the information relationship, not a mandatory card layout; navigation remains reachable when content reflows.

The old `/foundations/layout/understanding-layout/overview` path returned 404 at acquisition. The current recorded route is `/foundations/layout/layout-overview`. Use the manifest's observed URLs rather than assuming older links are still authoritative.

### Typography and icons

- [Typography](material/excerpts/m3-typography.md): role, size, weight, line height and tracking work together. Baseline/emphasized styles coexist; body text serves long reading. The August 2026 update distinguishes language-height categories and warns about fixed-height components. Chinese is in the medium group. Project use: actual fallback-font measurement, role-specific line boxes, emphasized UI labels and regular note prose. Do not apply a blanket “every sentence bold” rule.
- [Icons](material/excerpts/m3-icons.md): preserve a coherent icon family, optical weight and size. Desktop can use 20dp icons/40dp targets. Inline Symbols need optical baseline alignment; the guidance mentions an approximately 11.5% text-size shift. Project use: inspect actual SVG/glyph rendering before choosing a shared correction. Do not apply a glyph baseline offset to an already centered SVG.

The complete Compose guide supplies the baseline role table; current M3 adds emphasized styles and language-height guidance. These are documented editions, not interchangeable claims that every platform component has the newest tokens.

### States and motion

- [States](material/excerpts/m3-states.md): selection combines with transient interaction feedback. One overlay sits above the base and below content. Current opacities are 8/10/10/16% for hover/focus/press/drag. Project use: preserve the selected base and apply one transient layer, with keyboard focus still visible.
- [Motion physics](material/excerpts/m3-motion-physics.md): Standard favors functional use; spatial/effects motion have separate roles and effects avoid overshoot. Springs can accommodate interruption and retargeting. Project use: functional shared motion, stationary labels and immediate retargeting.
- [Easing and duration](material/excerpts/m3-easing.md): this legacy system remains for transitions and teams without the new physics implementation; it is no longer the maintained Expressive component system. Project use: a documented fallback, not a claim of adopting current spring tokens.
- [Transitions](material/excerpts/m3-transitions.md): match the relationship between states, keep layout stable, use a coherent direction and avoid messy overlapping fades. Project use: one principal response per intent; no repeated page re-entry on every choice.
- [Progress](material/excerpts/m3-progress.md): determinate indicators show real known progress; unknown progress uses indeterminate feedback. Group related work under one indicator and use consistent variants. Project use: truthful phases, no invented percent/time, and no mandatory completion page before the note.

### Containers, choices and explanation

- [Cards](material/excerpts/m3-cards.md): a card contains one topic and its actions; its dimensions follow content. Simpler heading/spacing/divider hierarchy can be preferable. Project use: avoid nested panels and empty fixed-height cards around ordinary form groups.
- [Lists](material/excerpts/m3-lists.md): repeat a consistent arrangement of icons, text and actions, with concise comparable items. Project use: shared leading/content/trailing slots and stable action positions.
- [Dialogs](material/excerpts/m3-dialogs.md): an intentional interruption for critical information or a decision; a scrolling body retains its title/actions. Project use: consequential confirmation/repair, with explicit return and preserved input.
- [Tooltips](material/excerpts/m3-tooltips.md): plain tooltips label icon controls; richer help can explain context. Critical information cannot be confined to them. Project use: optional details may be disclosed, but consequences/errors stay visible with actions. The project's in-place information panel is not a modal dialog.
- [Segmented buttons](material/excerpts/m3-segmented-buttons.md) and [button groups](material/excerpts/m3-button-groups.md): Expressive recommends connected button groups; short related choices use a coherent group and should not wrap within a segment. Project use: shared capsule choices, measured inner width and stable labels. This is a restrained project adaptation, not a full Expressive shape-morph implementation.

## Verify and update the archive

Run from the repository root:

```sh
python3 .agents/skills/course2md-design/scripts/material_sources.py --verify
```

This checks every saved file against its recorded SHA-256 and rejects a JavaScript shell. It verifies provenance/integrity, not design quality. For a read-only comparison of the recorded upstream responses, use `--check-remote`. For an intentional recapture of the existing allowlist, use `--refresh`; failed acquisition completes no writes. Refresh does not update project design choices automatically.

M3 data URLs are version-pinned. A current-edition update requires checking the new site's published routes/version, selecting the same relevant topics, rechecking licence evidence and changing the manifest deliberately. Do not crawl the full site, fetch unrelated media or silently upgrade a short M3 quotation into a full copyrighted page. Preserve the distinction between official source, paraphrase and project decision.
