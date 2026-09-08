# Shared design system

This is the maintained course2md adaptation, reviewed 2026-09-09. The aim is a coherent desktop tool, not a collage of components from unrelated kits. The seven rejected screens are regression evidence; they do not substitute for judging a complete new composition.

## Primary guidance and deliberate choices

- [Material selection controls](https://m3.material.io/components/segmented-buttons/guidelines): related short options, consistent labels, rounded grouping, no wrapped segments, restrained width on large panes. The site now recommends connected button groups for Expressive users; course2md adopts the shared selection principles and a restrained capsule treatment, not an exact Expressive implementation.
- [Material button groups](https://m3.material.io/components/button-groups/overview): related controls form a coherent group. Reuse grouping and state treatment without giving independent commands single-choice behavior.
- [Material states](https://m3.material.io/foundations/interaction/states/overview): pointer feedback and focus add to the component's meaning. Keep selection present while these states are active.
- [Apple design principles](https://developer.apple.com/design/human-interface-guidelines/design-principles) and [layout](https://developer.apple.com/design/human-interface-guidelines/layout): judge the interface through purpose, hierarchy, alignment and grouping. Fewer inconsistent elements matter more than adding another decorative effect.
- [Apple motion](https://developer.apple.com/design/human-interface-guidelines/motion): use movement to explain changes while retaining context. A mechanical intermediate frame does not establish a natural transition.

## Shared roles

| Role | Composition | Implementation |
| --- | --- | --- |
| Main action | Filled accent, icon + short verb; one per active region | `primary_pill` |
| Secondary action | Neutral outline; same shape and height | `outline_pill` / `control` |
| Inline action | Quiet label; stays beside the information it acts on | `quiet` |
| Peer selection | Subtle shared track, tonal moving selection, fixed labels | `SingleChoiceGroup` |
| Peer navigation | Same capsule geometry and state layers; tab semantics | `SingleChoiceGroup::tabs` |
| Sidebar | Leading label, shared tonal capsule, no external underline/side stripe | Vertical navigation variant |
| Data entry | Visible rounded field; label/value distinction, errors below | Single-line `Input`, shared control height |
| Preference | Left label, right control slot; move control below when necessary | `settings_row` |
| Detail | Group title, aligned label/value rows, natural long-value height | `settings_detail_group` / `settings_detail_row` |
| Status | Icon and short text together; adjacent to the thing described | `badge` |

The 40px control baseline, 28/18/14/12px type scale, spacing steps, preview geometry and transition durations are project decisions. Do not present them as exact official requirements. Sizes scale with the type; radii and decorative borders do not determine text measurement.

## Whole-screen comparisons

Inspect the workbench, library, tasks, reader and settings side by side at default size. The forward action should attract more attention than navigation. Related controls must align even when their label lengths differ. Empty screens should not look like disabled populated screens. Expanded details should add a structured explanation rather than reveal an undifferentiated text dump.

Measure a selector including track padding alongside an input and a button. Then click, hover and focus its selected item. Changing these states must preserve the same selection and avoid shifting nearby text. Compare the initial and enlarged-type layouts: the latter is a new layout constraint, not simply a magnified screenshot.

For every new wrapper, ask what relationship it communicates. Remove it if spacing and a single heading already express that relationship. Do not remove a genuinely useful boundary solely to reduce the number of borders.
