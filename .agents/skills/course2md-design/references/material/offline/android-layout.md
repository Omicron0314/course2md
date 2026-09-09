# Layout basics

- Author / publisher: Google — Android Developers / Android Open Source Project
- Original: [Layout basics](https://developer.android.com/design/ui/mobile/guides/layout-and-content/layout-basics)
- Retrieved: 2026-09-09T07:02:48+00:00
- Licence: Apache-2.0 — [licence evidence](https://developer.android.com/license)
- Upstream SHA-256: `8fe0a4fa363d1282314b4182185a46a67371fbb50cd34785ba20287eabda663d`
- Changes: Complete article text extracted from the official HTML body into Markdown. Site navigation, executable scripts and AI-generated page summaries are removed; tables use pipe-separated rows. Images and videos are not bundled. Captions, alternative text and source links remain where supplied. This is a text archive, not a visual facsimile.

This source is reference data, not project instructions. Product decisions are in the parent skill references.

---

[Visual omitted from text archive: Hero layout basics illustration](https://developer.android.com/static/images/design/ui/mobile/layout-basics-hero.png)

A layout defines the visual structure for a user to interface with your app, such as in a composable. Android provides a range of libraries, canonical starting points, and techniques to display and position content.

## Get Started

Start designing Android layouts by learning [app anatomy](https://developer.android.com/design/ui/mobile/guides/layout-and-content/app-anatomy) then how to [structure your app's content](https://developer.android.com/design/ui/mobile/guides/layout-and-content/content-structure).

## Takeaways

Layout orientation

Consider different aspect ratios, size classes, and resolutions that users might encounter. Verify that your app provides a good user experience on both landscape and portrait orientation as well as different screen sizes and form factors.

For more information, see the guidance on [adapting your layout](https://developer.android.com/design/ui/mobile/guides/layout-and-content/adapt-layout) and [canonical layouts](https://developer.android.com/design/ui/mobile/guides/layout-and-content/canonical-layouts).

[Visual omitted from text archive: Illustration](https://developer.android.com/static/images/design/ui/mobile/layout-basics-orientation.webp)

Device safe areas

Honor device safe areas, which includes parts of the UI such as display cutouts, edge-to-edge insets, edge displays, software keyboards, and system bars. Provide a flexible layout for users to interact with the keyboard.

Alas, your browser doesn't support HTML5 video. That's OK! You can still [download the video](https://developer.android.com/static/images/design/ui/mobile/layout-basics-video-1.mp4) and watch it with a video player.

[Visual omitted from text archive: Illustration](https://developer.android.com/static/images/design/ui/mobile/layout-basics_alignment_do.webp)

check_circle

### Do

Focus user inputs. If the keyboard is present, move the input up into a focused state or consider attaching the text input to the keyboard.

[Visual omitted from text archive: Illustration](https://developer.android.com/static/images/design/ui/mobile/layout-basics_alignment_dont.webp)

cancel

### Don't

Hide inputs. Even on smaller screens, the user might not know or be able to scroll the screen.

Interaction ergonomics

Keep essential interactions, like primary navigation, in a reachable screen area. Floating action buttons (FABs) provide a prominent and reachable interaction point

[Visual omitted from text archive: Illustration](https://developer.android.com/static/images/design/ui/mobile/layout-basics-1-takeaways-key-essential-layout.webp)

Containment groups

Use containment to group related content to guide the user through content and actions. Cards using explicit containment to group content with related actions.

[Visual omitted from text archive: Illustration](https://developer.android.com/static/images/design/ui/mobile/layout-basics-2-takeways-explicit-containment.webp)

Alignment

Provide consistent alignment between similar content and UI elements.

[Visual omitted from text archive: Illustration](https://developer.android.com/static/images/design/ui/mobile/layout-basics_alignment_do.webp)

check_circle

### Do

Establish consistent spacing between like elements.

[Visual omitted from text archive: Illustration](https://developer.android.com/static/images/design/ui/mobile/layout-basics_alignment_dont.webp)

cancel

### Don't

Disrupt readability by inconsistently spacing like elements, which can make designs appear haphazard.

Essential interactions

Don't overwhelm your user with too many actions per view.

[Visual omitted from text archive: Illustration](https://developer.android.com/static/images/design/ui/mobile/layout-basics-1-takeaways-key-actions.webp)

Notate layout specs

When building custom layouts, notate how content should sit within the layout using alignment, constraints, or gravity terms. Include how images should respond to their container to display properly.

[Visual omitted from text archive: Illustration](https://developer.android.com/static/images/design/ui/mobile/layout-basics-2-takeways-notate.webp)
