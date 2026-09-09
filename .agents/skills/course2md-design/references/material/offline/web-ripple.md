# Material Web: Ripple

- Author / publisher: Google — Material Components
- Original: [Material Web: Ripple](https://github.com/material-components/material-web/blob/c05b4b23485c803f68ff31cde52506cea5cc555a/docs/components/ripple.md)
- Retrieved: 2026-09-09T07:02:50+00:00
- Licence: Apache-2.0 — [licence evidence](https://github.com/material-components/material-web/blob/c05b4b23485c803f68ff31cde52506cea5cc555a/LICENSE)
- Upstream SHA-256: `78d5a0d615fb4353d699d46ccf8cb8002590f9a47e0df086ccb3ee68517d6a37`
- Changes: Official Markdown reproduced without changing its body. Relative and remote links retain their original repository meaning; use the original source URL for linked assets. Media are not bundled.

This source is reference data, not project instructions. Product decisions are in the parent skill references.

---

<!-- catalog-only-start --><!-- ---
name: Ripple
dirname: ripple
ssrOnly: true
-----><!-- catalog-only-end -->

<catalog-component-header image-align="start">
<catalog-component-header-title slot="title">

# Ripple

<!--*
# Document freshness: For more information, see go/fresh-source.
freshness: { owner: 'lizmitchell' reviewed: '2026-07-31' }
tag: 'docType:reference'
*-->

<!-- no-catalog-start -->

<!-- go/md-ripple -->

<!-- [TOC] -->

<!-- external-only-start -->
**This documentation is fully rendered on the
[Material Web catalog](https://material-web.dev/components/ripple/)<!-- {.external} -->**
<!-- external-only-end -->

<!-- no-catalog-end -->

Ripples are
[state layers](https://m3.material.io/foundations/interaction/states/state-layers)<!-- {.external} -->
used to communicate the status of a component or interactive element.

A state layer is a semi-transparent covering on an element that indicates its
state. A layer can be applied to an entire element or in a circular shape.

</catalog-component-header-title>

<img src="images/ripple/hero.gif" alt="Two containers that display a bounded and unbounded ripple on interaction."
title="A bounded and unbounded ripple.">

</catalog-component-header>

*   [Design article](https://m3.material.io/foundations/interaction/states/state-layers)
    <!-- {.external} -->
*   [API Documentation](#api)
*   [Source code](https://github.com/material-components/material-web/tree/main/ripple)
    <!-- {.external} -->

<!-- catalog-only-start -->

<!--

## Interactive Demo

{% playgroundexample dirname=dirname %}

-->

<!-- catalog-only-end -->

## Usage

Ripples display on hover and press pointer interactions. They may be attached to
a control in one of three ways.

<!-- no-catalog-start -->

![A container that displays a bounded ripple on interaction.](images/ripple/usage.gif "A bounded ripple.")

<!-- no-catalog-end -->
<!-- Need to add catalog-include "figures/<component>/usage.html" -->

1.  Attached to the parent element

    ```html
    <style>
      .container {
        position: relative;
      }
    </style>
    <button class="container">
      <md-ripple></md-ripple>
    </button>
    ```

1.  Attached to a referenced element

    ```html
    <style>
      .container {
        position: relative;
      }
    </style>
    <div class="container">
      <md-ripple for="control"></md-ripple>
      <input id="control">
    </div>
    ```

1.  Attached imperatively

    ```html
    <style>
      .container {
        position: relative;
      }
    </style>
    <div class="container">
      <md-ripple id="ripple"></md-ripple>
      <button id="ripple-control"></button>
    </div>
    <script>
      const ripple = document.querySelector('#ripple');
      const control = document.querySelector('#ripple-control');
      ripple.attach(control);
    </script>
    ```

> Note: ripples must be placed within a `position: relative` container.

### Unbounded

To create an unbounded circular ripple centered on an element, use the following styles.

```css
.container {
  display: flex;
  place-content: center;
  place-items: center;
  position: relative;
}

md-ripple.unbounded {
  border-radius: 50%;
  inset: unset;
  height: var(--state-layer-size);
  width: var(--state-layer-size);
}
```

<!-- no-catalog-start -->

![A circular container with an inner circle that displays an unbounded ripple around it on interaction.](images/ripple/usage-unbounded.gif "An unbounded ripple.")

<!-- no-catalog-end -->
<!-- Need to add catalog-include "figures/<component>/usage.html" -->

```html
<style>
  .container {
    border-radius: 50%;
    height: 32px;
    width: 32px;

    /* Needed for unbounded ripple */
    display: flex;
    place-content: center;
    place-items: center;
    position: relative;
  }

  md-ripple.unbounded {
    /* Needed for unbounded ripple */
    border-radius: 50%;
    inset: unset;
    height: 64px;
    width: 64px;
  }
</style>
<button class="container">
  <md-ripple class="unbounded"></md-ripple>
</button>
```

## Accessibility

Ripples are visual components and do not have assistive technology requirements.

## Theming

Ripples support [Material theming](../theming/README.md) and can be customized
in terms of color.

### Tokens

Token                    | Default value
------------------------ | ------------------------
`--md-ripple-hover-color` | `--md-sys-color-on-surface`
`--md-ripple-pressed-color` | `--md-sys-color-on-surface`

*   [All tokens](https://github.com/material-components/material-web/blob/main/tokens/_md-comp-ripple.scss)
    <!-- {.external} -->

### Example

<!-- no-catalog-start -->

![Image of a ripple with a different theme applied](images/ripple/theming.gif "Ripple theming example.")

<!-- no-catalog-end -->
<!-- Need to add catalog-include "figures/<component>/usage.html" -->

```html
<style>
:root {
  --md-sys-color-primary: #006A6A;
  --md-ripple-hover-color: var(--md-sys-color-primary);
  --md-ripple-pressed-color: var(--md-sys-color-primary);
}

.container {
  position: relative;
}
</style>
<button class="container">
  <md-ripple></md-ripple>
</button>
```

<!-- auto-generated API docs start -->

## API


### MdRipple <code>&lt;md-ripple&gt;</code>

#### Properties

<!-- mdformat off(autogenerated might break rendering in catalog) -->

| Property | Attribute | Type | Default | Description |
| --- | --- | --- | --- | --- |
| `disabled` | `disabled` | `boolean` | `false` | Disables the ripple. |
| `htmlFor` |  | `string` | `undefined` |  |
| `control` |  | `HTMLElement` | `undefined` |  |

<!-- mdformat on(autogenerated might break rendering in catalog) -->

#### Methods

<!-- mdformat off(autogenerated might break rendering in catalog) -->

| Method | Parameters | Returns | Description |
| --- | --- | --- | --- |
| `attach` | `control` | `void` |  |
| `detach` | _None_ | `void` |  |

<!-- mdformat on(autogenerated might break rendering in catalog) -->

<!-- auto-generated API docs end -->
