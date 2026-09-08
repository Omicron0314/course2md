//! Semantic design tokens and animated appearance for all desktop pages.
use crate::palettes::{PaletteId, ThemePreferences};
use gpui::{App, Pixels, Rems, Rgba, Window, px, rems, rgb};
use gpui_component::{Theme, ThemeMode};
use std::{cell::RefCell, time::Instant};
#[path = "choice_group.rs"]
mod choice_group;
pub use choice_group::SingleChoiceGroup;

macro_rules! tokens {
    ($($name:ident),+ $(,)?) => {
        #[allow(non_camel_case_types)]
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        #[repr(usize)]
        pub enum ColorToken { $($name),+ }
        $(pub const $name: ColorToken = ColorToken::$name;)+
        const ALL_TOKENS: &[ColorToken] = &[$(ColorToken::$name),+];
    }
}
tokens!(
    CANVAS,
    SURFACE,
    INSET,
    INK,
    GRAY,
    FAINT,
    HAIRLINE,
    CARD_LINE,
    CONTROL,
    ACCENT,
    ACCENT_STRONG,
    ACCENT_SOFT,
    PRIMARY,
    PRIMARY_HOVER,
    PRIMARY_ACTIVE,
    ON_PRIMARY,
    SUCCESS,
    SUCCESS_BG,
    WARNING,
    WARNING_BG,
    DANGER,
    DANGER_BG,
    BADGE_PROGRESS,
    BADGE_PROGRESS_BG,
    PROGRESS_FILL,
    PROGRESS_TRACK,
    FIND_HIGHLIGHT,
    FIND_CURRENT,
    FIND_CURRENT_LINE,
    SEGMENT_TRACK,
    SWITCH_TRACK_OFF,
    HOVER_WARM,
    SELECTION
);
const COLOR_COUNT: usize = ALL_TOKENS.len();

pub const SIDEBAR: ColorToken = INSET;
pub const COVER: ColorToken = INSET;
pub const MUTED: ColorToken = GRAY;
pub const LINE: ColorToken = HAIRLINE;
pub const BLUE: ColorToken = ACCENT_STRONG;
pub const TINT: ColorToken = ACCENT_SOFT;

pub fn blend(from: Rgba, to: Rgba, t: f32) -> Rgba {
    let t = t.clamp(0., 1.);
    Rgba {
        r: from.r + (to.r - from.r) * t,
        g: from.g + (to.g - from.g) * t,
        b: from.b + (to.b - from.b) * t,
        a: from.a + (to.a - from.a) * t,
    }
}

fn palette_colors(id: PaletteId) -> [Rgba; COLOR_COUNT] {
    let p = id.colors();
    let tint = |foreground, amount| blend(rgb(p.surface), rgb(foreground), amount);
    std::array::from_fn(|index| match ALL_TOKENS[index] {
        CANVAS => rgb(p.canvas),
        SURFACE => rgb(p.surface),
        INSET => rgb(p.inset),
        INK => rgb(p.text),
        GRAY => rgb(p.muted),
        FAINT => rgb(p.subtle),
        HAIRLINE | CARD_LINE => rgb(p.border),
        CONTROL => rgb(p.control),
        ACCENT | ACCENT_STRONG | PRIMARY | PROGRESS_FILL => rgb(p.accent),
        ON_PRIMARY => rgb(p.on_accent),
        PRIMARY_HOVER => blend(rgb(p.accent), rgb(p.text), 0.10),
        PRIMARY_ACTIVE => blend(rgb(p.accent), rgb(p.text), 0.20),
        ACCENT_SOFT => tint(p.accent, if id.is_dark() { 0.16 } else { 0.10 }),
        SUCCESS => rgb(p.success),
        SUCCESS_BG => tint(p.success, 0.12),
        WARNING => rgb(p.warning),
        WARNING_BG => tint(p.warning, 0.12),
        DANGER => rgb(p.danger),
        DANGER_BG => tint(p.danger, 0.12),
        BADGE_PROGRESS => rgb(p.accent),
        BADGE_PROGRESS_BG => tint(p.accent, 0.10),
        PROGRESS_TRACK | SEGMENT_TRACK | SWITCH_TRACK_OFF => rgb(p.border),
        HOVER_WARM => tint(p.text, 0.05),
        SELECTION => rgb(p.selection),
        FIND_HIGHLIGHT => tint(p.warning, 0.16),
        FIND_CURRENT => tint(p.warning, 0.32),
        FIND_CURRENT_LINE => rgb(p.warning),
    })
}

struct PaintState {
    target: Option<PaletteId>,
    from: [Rgba; COLOR_COUNT],
    current: [Rgba; COLOR_COUNT],
    started: Instant,
}
thread_local! {
    // GPUI renders application chrome and all overlays on its UI thread.
    // Thread-local paint values keep helpers context-free without unsafe globals.
    static PAINT: RefCell<PaintState> = RefCell::new(PaintState {
        target: None, from: palette_colors(PaletteId::Paper),
        current: palette_colors(PaletteId::Paper), started: Instant::now(),
    });
}

pub fn color(token: ColorToken) -> Rgba {
    PAINT.with(|paint| paint.borrow().current[token as usize])
}

pub fn system_dark(cx: &App) -> bool {
    ThemeMode::from(cx.window_appearance()).is_dark()
}

/// Resolve system changes on each notified frame, and retarget palette changes
/// from their current paint values. Persisted settings are the sole authority.
pub fn apply_preference(preference: &ThemePreferences, window: &mut Window, cx: &mut App) {
    let id = preference.resolve(system_dark(cx));
    let target = palette_colors(id);
    let (changed, running) = PAINT.with(|paint| {
        let mut paint = paint.borrow_mut();
        let now = Instant::now();
        if paint.target.is_none() {
            paint.target = Some(id);
            paint.current = target;
            paint.from = target;
            return (true, false);
        }
        if paint.target != Some(id) {
            paint.from = paint.current;
            paint.target = Some(id);
            paint.started = now;
        }
        let t = if cx.reduce_motion() {
            1.
        } else {
            (now.duration_since(paint.started).as_secs_f32() / 0.280).min(1.)
        };
        let eased = 1. - (1. - t).powi(3);
        let next = std::array::from_fn(|i| blend(paint.from[i], target[i], eased));
        let changed = next != paint.current;
        paint.current = next;
        (changed, t < 1.)
    });
    if changed {
        sync_component_theme(id.is_dark(), cx);
    }
    if running {
        window.request_animation_frame();
    }
}

fn sync_component_theme(dark: bool, cx: &mut App) {
    let theme = Theme::global_mut(cx);
    theme.mode = if dark {
        ThemeMode::Dark
    } else {
        ThemeMode::Light
    };
    theme.radius = px(8.);
    theme.radius_lg = RADIUS_CARD;
    let colors = &mut theme.colors;
    macro_rules! set {
        ($token:expr => $($field:ident),+ $(,)?) => { $(colors.$field = color($token).into();)+ };
    }
    set!(CANVAS => background, title_bar, tab_bar, status_bar, tiles);
    set!(SURFACE => button, popover, list, table, accordion, group_box, tab_active);
    set!(INSET => sidebar, table_head, table_foot, description_list_label, list_head, list_even, table_even, tab, secondary, button_secondary);
    set!(INK => foreground, button_foreground, popover_foreground, secondary_foreground, button_secondary_foreground, sidebar_foreground, group_box_foreground, table_head_foreground, table_foot_foreground, tab_active_foreground);
    set!(GRAY => muted_foreground, description_list_label_foreground, tab_foreground);
    set!(HAIRLINE => border, title_bar_border, sidebar_border, status_bar_border, table_row_border, window_border);
    set!(CONTROL => input, scrollbar_thumb, switch);
    set!(FAINT => scrollbar_thumb_hover);
    set!(ACCENT => accent_foreground, primary, button_primary, caret, ring, link, progress_bar, slider_bar, drag_border, sidebar_primary);
    set!(ON_PRIMARY => primary_foreground, button_primary_foreground, sidebar_primary_foreground);
    set!(PRIMARY_HOVER => primary_hover, button_primary_hover, link_hover);
    set!(PRIMARY_ACTIVE => primary_active, button_primary_active, link_active);
    set!(ACCENT_SOFT => accent, drop_target, list_active, sidebar_accent, table_active);
    set!(ACCENT_STRONG => list_active_border, table_active_border, sidebar_accent_foreground);
    set!(HOVER_WARM => button_hover, secondary_hover, button_secondary_hover, list_hover, table_hover);
    set!(SEGMENT_TRACK => muted, skeleton, tab_bar_segmented, button_active, secondary_active, button_secondary_active);
    set!(SELECTION => selection);
    set!(SURFACE => switch_thumb, slider_thumb);
    set!(DANGER => danger, button_danger_foreground, danger_hover, danger_active);
    set!(ON_PRIMARY => danger_foreground, success_foreground, warning_foreground, info_foreground);
    set!(DANGER_BG => button_danger, button_danger_hover, button_danger_active);
    set!(SUCCESS => success, button_success_foreground, success_hover, success_active);
    set!(SUCCESS_BG => button_success, button_success_hover, button_success_active);
    set!(WARNING => warning, button_warning_foreground, warning_hover, warning_active);
    set!(WARNING_BG => button_warning, button_warning_hover, button_warning_active);
    set!(ACCENT => info, info_hover, info_active, button_info_foreground);
    set!(ACCENT_SOFT => button_info, button_info_hover, button_info_active);
    colors.overlay = gpui::hsla(0., 0., 0., if dark { 0.58 } else { 0.28 });
    colors.scrollbar = gpui::hsla(0., 0., 0., 0.);
    theme.tokens = theme.colors.into();
    Theme::sync_base(cx);
}

/* ---------- 几何（px，不随字号缩放） ---------- */
pub const RADIUS_PILL: Pixels = px(999.);
pub const RADIUS_CARD: Pixels = px(12.);
pub const RADIUS_HERO: Pixels = px(16.);
pub const RADIUS_SMALL: Pixels = px(8.);
/// Compact form choices stay associated with their labels in a wide pane.
pub const CONTROL_GROUP_MAX: Pixels = px(520.);

/* ---------- 栏宽（rems，随字号缩放的结构尺寸） ---------- */
pub const COLUMN: Rems = rems(65.714);
pub const TOC_PANEL: Rems = rems(16.);

/* ---------- 字级（rems；14px 为 1rem 基准） ---------- */
pub const TEXT_AUX: Rems = rems(0.857);
pub const TEXT_BODY: Rems = rems(1.);
pub const TEXT_TITLE: Rems = rems(1.286);
pub const TEXT_READER: Rems = rems(1.143);
/// Main page headline.
pub const TEXT_DISPLAY: Rems = rems(2.);

/* ---------- 中性阴影 ---------- */
fn shadow_color(alpha: f32) -> gpui::Hsla {
    gpui::hsla(0., 0., 0., alpha)
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
        .text_color(color(if selected { ON_PRIMARY } else { INK }))
        .when(selected, |button| {
            button
                .bg(color(BLUE))
                .font_weight(gpui::FontWeight::SEMIBOLD)
        })
}

pub fn init(cx: &mut App) {
    choice_group::init(cx);
    Theme::change(ThemeMode::Light, None, cx);
    Theme::global_mut(cx).font_size = px(14.);
    sync_component_theme(false, cx);
}

/// Compatibility wrapper for source confirmations.
pub fn reveal(view: gpui::Div, id: impl Into<gpui::ElementId>, cx: &App) -> gpui::AnyElement {
    crate::motion::enter(id, view, cx)
}

pub fn disclosure(
    id: impl Into<gpui::ElementId>,
    open: bool,
    content: gpui::Div,
    window: &mut Window,
    cx: &mut App,
) -> gpui::AnyElement {
    crate::motion::disclosure(id, open, content, window, cx)
}

/// One baseline for ordinary actions; content buttons explicitly opt into auto height.
pub fn control(id: impl Into<gpui::ElementId>) -> gpui_component::button::Button {
    use gpui::Styled;
    gpui_component::button::Button::new(id)
        .h_auto()
        .min_h(rems(2.571))
        .flex_shrink_0()
        .text_size(rems(1.0))
}

/* ---------- 共享控件 ---------- */

/// A selectable preview surface. Its opaque contents cannot conceal pointer
/// feedback: hover changes the outline/shadow, and press dims the whole card.
pub fn selection_card(
    id: impl Into<gpui::ElementId>,
    selected: bool,
    amount: f32,
) -> gpui_base::Button {
    use gpui::{prelude::*, *};
    gpui_base::Button::new(id)
        .selected(selected)
        .aria_toggled(if selected {
            gpui::accesskit::Toggled::True
        } else {
            gpui::accesskit::Toggled::False
        })
        .flex()
        .flex_col()
        .h_auto()
        .min_h(px(0.))
        .min_w_0()
        .rounded(RADIUS_CARD)
        .border_2()
        .border_color(blend(color(HAIRLINE), color(ACCENT), amount))
        .bg(color(SURFACE))
        .cursor_pointer()
        .hover(|style| style.border_color(color(ACCENT_STRONG)).shadow_sm())
        .active(|style| style.opacity(0.8).shadow_none())
        .focus(|style| style.border_color(color(INK)).shadow_sm())
}

/// The single forward action of a page or region, in the active theme's accent.
pub fn primary_pill(id: impl Into<gpui::ElementId>) -> gpui_component::button::Button {
    use gpui::Styled;
    use gpui_component::button::ButtonVariants;
    control(id)
        .primary()
        .rounded(RADIUS_SMALL)
        .px(px(16.))
        .gap(px(8.))
        .font_weight(gpui::FontWeight::SEMIBOLD)
}

/// Meaningful secondary action with a control-strength border.
pub fn outline_pill(id: impl Into<gpui::ElementId>) -> gpui_component::button::Button {
    use gpui::Styled;
    control(id)
        .outline()
        .rounded(RADIUS_SMALL)
        .px(px(16.))
        .gap(px(8.))
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
        .rounded(RADIUS_SMALL)
        .text_color(color(GRAY))
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
        .bg(color(SEGMENT_TRACK))
}

/// Inset note for supporting information.
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
        .bg(color(HOVER_WARM))
        .text_size(TEXT_AUX)
        .text_color(color(GRAY))
        .child(value)
}

/// One segment: selected uses a raised surface with a soft shadow.
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
        .text_color(color(if selected { INK } else { GRAY }))
        .when(selected, |button| {
            button
                .bg(color(SURFACE))
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

/// Icon, padding and label belong to one container; callers append the label.
pub fn badge(kind: BadgeKind) -> gpui::Div {
    use gpui::{Styled, div, prelude::*};
    use gpui_component::Sizable;
    let (text, bg, icon) = match kind {
        BadgeKind::Success => (SUCCESS, SUCCESS_BG, crate::icons::check_circle()),
        BadgeKind::Warning => (WARNING, WARNING_BG, crate::icons::warning()),
        BadgeKind::Danger => (DANGER, DANGER_BG, crate::icons::error()),
        BadgeKind::Progress => (BADGE_PROGRESS, BADGE_PROGRESS_BG, crate::icons::schedule()),
        BadgeKind::Neutral => (GRAY, INSET, crate::icons::info()),
    };
    div()
        .flex()
        .items_center()
        .flex_shrink_0()
        .gap(px(6.))
        .min_h(px(24.))
        .px(px(8.))
        .py(px(3.))
        .rounded_full()
        .bg(color(bg))
        .text_color(color(text))
        .text_size(TEXT_AUX)
        .font_weight(gpui::FontWeight::MEDIUM)
        .whitespace_nowrap()
        .child(icon.small())
}

/// Preference switches use the same semantic accent as other selected controls.
pub fn coral_switch(switch: gpui_component::switch::Switch) -> gpui_component::switch::Switch {
    switch.color(color(ACCENT_STRONG))
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
