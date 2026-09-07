//! Shared colors keep controls, navigation and document surfaces consistent.
use gpui::{App, Window, px, rems, rgb};
use gpui_component::{Theme, ThemeMode};
#[path = "choice_group.rs"]
mod choice_group;
pub use choice_group::SingleChoiceGroup;

pub const CANVAS: u32 = 0xf8f8f6;
pub const SURFACE: u32 = 0xffffff;
pub const SIDEBAR: u32 = 0xe8ebf0;
pub const INK: u32 = 0x252a32;
pub const MUTED: u32 = 0x485363;
pub const LINE: u32 = 0xe3e5e2;
pub const CONTROL: u32 = 0x657184;
pub const BLUE: u32 = 0x315bc5;
pub const TINT: u32 = 0xeaf0ff;
pub const SUCCESS: u32 = 0x237552;

/// Plain Div text is not exposed by GPUI's native accessibility bridge. Keep labels on
/// leaf elements so an ancestor does not replace the accessibility of its controls.
pub fn accessible_text(
    id: impl Into<gpui::ElementId>,
    value: impl Into<gpui::SharedString>,
) -> gpui::Stateful<gpui::Div> {
    use gpui::*;
    let value = value.into();
    div()
        .id(id)
        .role(Role::Label)
        .aria_label(value.clone())
        .child(value)
}

/// Navigation has an explicit current-location treatment, separate from form toggles.
pub fn navigation(
    button: gpui_component::button::Button,
    selected: bool,
) -> gpui_component::button::Button {
    use gpui::{Styled, prelude::FluentBuilder};
    use gpui_component::{Selectable, button::ButtonVariants};
    button
        .ghost()
        .selected(selected)
        .toggled(selected)
        .text_color(rgb(if selected { SURFACE } else { INK }))
        .when(selected, |button| {
            button.bg(rgb(BLUE)).font_weight(gpui::FontWeight::SEMIBOLD)
        })
}

pub fn init(cx: &mut App) {
    choice_group::init(cx);
    Theme::change(ThemeMode::Light, None, cx);
    let theme = Theme::global_mut(cx);
    theme.font_size = px(14.);
    theme.radius = px(6.);
    theme.radius_lg = px(8.);
    let colors = &mut theme.colors;
    colors.background = rgb(CANVAS).into();
    colors.foreground = rgb(INK).into();
    colors.border = rgb(CONTROL).into();
    colors.input = rgb(CONTROL).into();
    colors.switch = rgb(CONTROL).into();
    colors.switch_thumb = rgb(SURFACE).into();
    colors.muted = rgb(SIDEBAR).into();
    colors.muted_foreground = rgb(MUTED).into();
    colors.accent = rgb(TINT).into();
    colors.accent_foreground = rgb(BLUE).into();
    colors.primary = rgb(BLUE).into();
    colors.progress_bar = rgb(BLUE).into();
    colors.primary_foreground = rgb(SURFACE).into();
    colors.primary_hover = rgb(0x284eae).into();
    colors.primary_active = rgb(0x214396).into();
    colors.button = rgb(SURFACE).into();
    colors.button_foreground = rgb(INK).into();
    colors.secondary_foreground = rgb(INK).into();
    colors.button_hover = rgb(0xf2f4f7).into();
    colors.button_primary = colors.primary;
    colors.button_primary_foreground = colors.primary_foreground;
    colors.button_primary_hover = colors.primary_hover;
    colors.button_primary_active = colors.primary_active;
    colors.button_active = rgb(TINT).into();
    colors.secondary_active = rgb(TINT).into();
    colors.ring = rgb(BLUE).into();
    colors.selection = rgb(0xcbdafa).into();
    theme.tokens = theme.colors.into();
    Theme::sync_base(cx);
}

/// Infrequent reveals only: source confirmation and expanded task options.
pub fn reveal(view: gpui::Div, id: impl Into<gpui::ElementId>, cx: &App) -> gpui::AnyElement {
    use gpui::{Animation, AnimationExt, IntoElement, Styled};
    if cx.reduce_motion() {
        return view.into_any_element();
    }
    view.with_animation(
        id,
        Animation::new(std::time::Duration::from_millis(180))
            .with_easing(|t| 1. - (1. - t).powi(3)),
        |view, t| view.relative().top(px(6. * (1. - t))).opacity(t),
    )
    .into_any_element()
}

/// Strong selection for mutually exclusive choices and toggle chips.
pub fn choice(
    button: gpui_component::button::Button,
    selected: bool,
) -> gpui_component::button::Button {
    use gpui::{Styled, prelude::FluentBuilder};
    use gpui_component::Selectable;
    button
        .selected(selected)
        .toggled(selected)
        .h_auto()
        .min_h(rems(2.6))
        .bg(rgb(if selected { BLUE } else { SURFACE }))
        .text_color(rgb(if selected { SURFACE } else { INK }))
        .border_color(rgb(if selected { BLUE } else { CONTROL }))
        .when(selected, |b| b.font_weight(gpui::FontWeight::SEMIBOLD))
}

/// Retargetable expansion keeps the content and neighboring groups spatially connected.
pub fn disclosure(
    id: impl Into<gpui::ElementId>,
    open: bool,
    content: gpui::Div,
    window: &mut gpui::Window,
    cx: &mut App,
) -> gpui::AnyElement {
    use gpui::{IntoElement, Styled};
    let id = id.into();
    let progress = gpui_base::transition(
        id.clone(),
        if open { 1_f32 } else { 0_f32 },
        gpui_base::Transition::new(std::time::Duration::from_millis(200)).easing(
            gpui_base::Easing::CubicBezier {
                x1: 0.2,
                y1: 0.,
                x2: 0.,
                y2: 1.,
            },
        ),
        window,
        cx,
    );
    if progress <= 0.001 && !open {
        return gpui::div().into_any_element();
    }
    gpui_base::MotionReveal::new(id, progress, content.w_full().pb(px(4.)).into_any_element())
        .into_any_element()
}

/// One baseline for ordinary actions; content buttons explicitly opt into auto height.
pub fn control(id: impl Into<gpui::ElementId>) -> gpui_component::button::Button {
    use gpui::Styled;
    gpui_component::button::Button::new(id)
        .h_auto()
        .min_h(rems(2.6))
        .flex_shrink_0()
        .text_size(rems(1.0))
}

/// GPUI Component's Root resets rem size from Theme on every render. Update that
/// authority as well as the current window so parent and modal renders agree.
pub fn apply_scale(scale: f32, window: &mut Window, cx: &mut App) {
    let scale = if [1.0, 1.25, 1.5, 2.0].contains(&scale) {
        scale
    } else {
        1.0
    };
    let font_size = px(14.0 * scale);
    if Theme::global(cx).font_size != font_size {
        let theme = Theme::global_mut(cx);
        theme.font_size = font_size;
        theme.mono_font_size = px(13.0 * scale);
        Theme::sync_base(cx);
    }
    window.set_rem_size(font_size);
}
