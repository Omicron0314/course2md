//! Theme tokens for the centered single-column workspace direction (v2 mock in
//! docs/ux-mock/app.html). Warm cream canvas, ink text, coral accents and one
//! near-black primary action per region.
use gpui::{App, Pixels, Rems, Window, px, rems, rgb};
use gpui_component::{Theme, ThemeMode};
#[path = "choice_group.rs"]
mod choice_group;
pub use choice_group::SingleChoiceGroup;

/* ---------- 色板 ---------- */
pub const CANVAS: u32 = 0xf6f2eb;
pub const SURFACE: u32 = 0xffffff;
pub const INSET: u32 = 0xfbf7f0;
pub const INK: u32 = 0x221f1c;
pub const GRAY: u32 = 0x6e6558;
pub const FAINT: u32 = 0x8a8074;
pub const HAIRLINE: u32 = 0xe7e0d2;
pub const CARD_LINE: u32 = 0xede8dd;
pub const CONTROL: u32 = 0x8a7e68;
/// Decorative coral: wordmark glyph, current-tab underline, platform dots only.
pub const ACCENT: u32 = 0xd95d4e;
/// Text-level coral: selected marks, switches on, current TOC item (≈4.8:1 on white).
pub const ACCENT_STRONG: u32 = 0xc24a3c;
pub const ACCENT_SOFT: u32 = 0xf9e9e5;
pub const PRIMARY: u32 = INK;
pub const PRIMARY_HOVER: u32 = 0x3a342e;
pub const PRIMARY_ACTIVE: u32 = 0x171412;
pub const SUCCESS: u32 = 0x256b45;
pub const SUCCESS_BG: u32 = 0xe1f1e6;
pub const WARNING: u32 = 0x8a5410;
pub const WARNING_BG: u32 = 0xfaf0dc;
pub const DANGER: u32 = 0xb4231a;
pub const DANGER_BG: u32 = 0xfcede9;
pub const BADGE_PROGRESS: u32 = 0x6e5f4c;
pub const BADGE_PROGRESS_BG: u32 = 0xefe9dd;
pub const PROGRESS_FILL: u32 = 0x8a7e68;
pub const PROGRESS_TRACK: u32 = 0xefe9dd;
pub const FIND_HIGHLIGHT: u32 = 0xfcefc7;
pub const FIND_CURRENT: u32 = 0xf7dc8e;
pub const FIND_CURRENT_LINE: u32 = 0xe3c26a;
pub const SEGMENT_TRACK: u32 = 0xece6d9;
pub const SWITCH_TRACK_OFF: u32 = 0xd8d0bf;
pub const HOVER_WARM: u32 = 0xf1ebdf;
pub const SELECTION: u32 = 0xf3dad4;

/* ---------- 旧常量别名：页面仍引用，逐页迁移后由 M8 清理 ---------- */
/// Legacy alias of `SEGMENT_TRACK`.
pub const SIDEBAR: u32 = SEGMENT_TRACK;
/// Legacy alias of `CARD_LINE`.
pub const COVER: u32 = CARD_LINE;
/// Legacy alias of `GRAY`.
pub const MUTED: u32 = GRAY;
/// Legacy alias of `HAIRLINE`.
pub const LINE: u32 = HAIRLINE;
/// Legacy alias of `ACCENT_STRONG`.
pub const BLUE: u32 = ACCENT_STRONG;
/// Legacy alias of `ACCENT_SOFT`.
pub const TINT: u32 = ACCENT_SOFT;

/* ---------- 几何（px，不随字号缩放） ---------- */
pub const RADIUS_PILL: Pixels = px(999.);
pub const RADIUS_CARD: Pixels = px(12.);
pub const RADIUS_HERO: Pixels = px(18.);
pub const RADIUS_SMALL: Pixels = px(6.);

/* ---------- 栏宽（rems，随字号缩放的结构尺寸） ---------- */
pub const COLUMN: Rems = rems(51.429);
pub const COLUMN_SETTINGS: Rems = rems(48.571);
pub const TOC_PANEL: Rems = rems(16.);

/* ---------- 字级（rems；14px 为 1rem 基准） ---------- */
pub const TEXT_AUX: Rems = rems(0.857);
pub const TEXT_BODY: Rems = rems(1.);
pub const TEXT_TITLE: Rems = rems(1.286);
pub const TEXT_READER: Rems = rems(1.143);
/// Hero wordmark only.
pub const TEXT_DISPLAY: Rems = rems(3.);

/* ---------- 阴影（暖棕多层） ---------- */
fn shadow_color(alpha: f32) -> gpui::Hsla {
    // rgb(60, 50, 40) → hsl(30, 20%, 19.6%)
    gpui::hsla(30. / 360., 0.2, 0.196, alpha)
}
/// Hero input card: near 0/1/2 @5% + far 0/16/40 @8%.
pub fn shadow_hero() -> Vec<gpui::BoxShadow> {
    vec![
        gpui::BoxShadow::new(px(0.), px(1.), shadow_color(0.05)).blur_radius(px(2.)),
        gpui::BoxShadow::new(px(0.), px(16.), shadow_color(0.08)).blur_radius(px(40.)),
    ]
}
/// Selected segment pill: 0/1/2 @12%.
pub fn shadow_segment_selected() -> Vec<gpui::BoxShadow> {
    vec![gpui::BoxShadow::new(px(0.), px(1.), shadow_color(0.12)).blur_radius(px(2.))]
}
/// Overlay panels and dialogs: 0/12/32 @16%.
pub fn shadow_popover() -> Vec<gpui::BoxShadow> {
    vec![gpui::BoxShadow::new(px(0.), px(12.), shadow_color(0.16)).blur_radius(px(32.))]
}

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
    theme.radius = RADIUS_CARD;
    theme.radius_lg = RADIUS_HERO;
    let colors = &mut theme.colors;
    colors.background = rgb(CANVAS).into();
    colors.foreground = rgb(INK).into();
    colors.border = rgb(CONTROL).into();
    colors.input = rgb(CONTROL).into();
    colors.switch = rgb(SWITCH_TRACK_OFF).into();
    colors.switch_thumb = rgb(SURFACE).into();
    colors.muted = rgb(BADGE_PROGRESS_BG).into();
    colors.muted_foreground = rgb(GRAY).into();
    colors.accent = rgb(ACCENT_SOFT).into();
    colors.accent_foreground = rgb(ACCENT_STRONG).into();
    colors.primary = rgb(PRIMARY).into();
    colors.primary_foreground = rgb(SURFACE).into();
    colors.primary_hover = rgb(PRIMARY_HOVER).into();
    colors.primary_active = rgb(PRIMARY_ACTIVE).into();
    colors.progress_bar = rgb(PROGRESS_FILL).into();
    colors.button = rgb(SURFACE).into();
    colors.button_foreground = rgb(INK).into();
    colors.secondary_foreground = rgb(INK).into();
    colors.button_hover = rgb(HOVER_WARM).into();
    colors.button_primary = colors.primary;
    colors.button_primary_foreground = colors.primary_foreground;
    colors.button_primary_hover = colors.primary_hover;
    colors.button_primary_active = colors.primary_active;
    colors.button_active = rgb(BADGE_PROGRESS_BG).into();
    colors.secondary_active = rgb(BADGE_PROGRESS_BG).into();
    colors.ring = rgb(ACCENT_STRONG).into();
    colors.selection = rgb(SELECTION).into();
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

/// Retargetable expansion keeps the content and neighboring groups spatially connected.
pub fn disclosure(
    id: impl Into<gpui::ElementId>,
    open: bool,
    content: gpui::Div,
    window: &mut gpui::Window,
    cx: &mut App,
) -> gpui::AnyElement {
    use gpui::{IntoElement, Styled};
    // MotionReveal starts at zero height and depends on extra frames to measure
    // itself; with reduced motion that stalls on an empty group. Show the final
    // state immediately instead.
    if cx.reduce_motion() {
        return if open {
            content.w_full().pb(px(4.)).into_any_element()
        } else {
            gpui::div().into_any_element()
        };
    }
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

/* ---------- v2 控件语法（与 docs/ux-mock/app.html 一致） ---------- */

/// The single forward action of a page or region: near-black pill, white text.
pub fn primary_pill(id: impl Into<gpui::ElementId>) -> gpui_component::button::Button {
    use gpui::Styled;
    use gpui_component::button::ButtonVariants;
    control(id)
        .primary()
        .rounded(RADIUS_PILL)
        .px(px(16.))
        .font_weight(gpui::FontWeight::SEMIBOLD)
}

/// Meaningful secondary action: surface pill with a control-strength border.
pub fn outline_pill(id: impl Into<gpui::ElementId>) -> gpui_component::button::Button {
    use gpui::Styled;
    control(id).outline().rounded(RADIUS_PILL).px(px(16.))
}

/// Tertiary inline action: quiet gray text.
///
/// Do not add a hover text color here: gpui-component's Button applies its own
/// variant hover during render, and a user-set hover style trips the base
/// button's `hover style already set` debug assertion (debug builds panic).
pub fn quiet(id: impl Into<gpui::ElementId>) -> gpui_component::button::Button {
    use gpui::Styled;
    use gpui_component::button::ButtonVariants;
    control(id)
        .ghost()
        .rounded(RADIUS_PILL)
        .px(px(12.))
        .text_color(rgb(GRAY))
}

/// Capsule track for mutually exclusive segments (source, view, tabs).
pub fn seg_track() -> gpui::Div {
    use gpui::{Styled, div};
    div()
        .flex()
        .items_center()
        .flex_shrink_0()
        .gap(px(2.))
        .p(px(2.))
        .rounded_full()
        .bg(rgb(SEGMENT_TRACK))
}

/// Cream inset note for scope/policy lines (mock .scope-note).
pub fn banner_note(
    id: impl Into<gpui::ElementId>,
    value: impl Into<gpui::SharedString>,
) -> gpui::Stateful<gpui::Div> {
    use gpui::*;
    let value = value.into();
    div()
        .id(id)
        .role(gpui::Role::Label)
        .aria_label(value.clone())
        .w_full()
        .p(px(12.))
        .rounded(RADIUS_CARD)
        .bg(rgb(HOVER_WARM))
        .text_size(TEXT_AUX)
        .text_color(rgb(GRAY))
        .child(value)
}

/// One segment: selected renders as a white pill with a soft shadow.
pub fn seg_item(id: impl Into<gpui::ElementId>, selected: bool) -> gpui_component::button::Button {
    use gpui::{Styled, prelude::FluentBuilder};
    use gpui_component::{Selectable, button::ButtonVariants};
    control(id)
        .ghost()
        .rounded(RADIUS_PILL)
        .min_h(rems(2.286))
        .px(px(14.))
        .selected(selected)
        .toggled(selected)
        .text_color(rgb(if selected { INK } else { GRAY }))
        .when(selected, |button| {
            button
                .bg(rgb(SURFACE))
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .shadow(shadow_segment_selected())
        })
}

/// Status badge kinds; text always pairs with its tinted background.
pub enum BadgeKind {
    Success,
    Warning,
    Danger,
    Progress,
    Neutral,
}

/// Status badge: pill, tinted background, saturated-enough text color. The
/// h_flex wrapper keeps the pill hugging its content inside v_flex parents.
pub fn badge(kind: BadgeKind) -> gpui::Div {
    use gpui::{Styled, div, prelude::*};
    let (text, bg) = match kind {
        BadgeKind::Success => (SUCCESS, SUCCESS_BG),
        BadgeKind::Warning => (WARNING, WARNING_BG),
        BadgeKind::Danger => (DANGER, DANGER_BG),
        BadgeKind::Progress => (BADGE_PROGRESS, BADGE_PROGRESS_BG),
        BadgeKind::Neutral => (GRAY, BADGE_PROGRESS_BG),
    };
    gpui_base::h_flex().child(
        div()
            .flex()
            .items_center()
            .flex_shrink_0()
            .min_h(px(20.))
            .px(px(10.))
            .py(px(1.))
            .rounded_full()
            .bg(rgb(bg))
            .text_color(rgb(text))
            .text_size(TEXT_AUX)
            .font_weight(gpui::FontWeight::SEMIBOLD)
            .whitespace_nowrap(),
    )
}

/// Switches read their checked color from `tokens.primary` (near-black in v2);
/// preference rows want the text-level coral instead.
pub fn coral_switch(switch: gpui_component::switch::Switch) -> gpui_component::switch::Switch {
    switch.color(rgb(ACCENT_STRONG))
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
