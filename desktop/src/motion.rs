//! Small, shared motion vocabulary: entrances, retargetable values and live work.
use crate::theme::{ACCENT, PROGRESS_FILL, PROGRESS_TRACK, color};
use gpui::{prelude::*, *};
use gpui_component::{Icon, Sizable};
use std::time::Duration;

pub const ENTER_MS: u64 = 240;
pub const VALUE_MS: u64 = 200;

pub fn ease_out(t: f32) -> f32 {
    1. - (1. - t.clamp(0., 1.)).powi(3)
}

/// IDs belong to a logical state, not a frame or a progress value.
pub fn enter<E: IntoElement + Styled + 'static>(
    id: impl Into<ElementId>,
    view: E,
    cx: &App,
) -> AnyElement {
    if cx.reduce_motion() {
        return view.into_any_element();
    }
    view.with_animation(
        id,
        Animation::new(Duration::from_millis(ENTER_MS))
            .with_easing(ease_out)
            .with_max_fps(60.),
        |view, t| view.relative().top(px(8. * (1. - t))).opacity(t),
    )
    .into_any_element()
}

pub fn spinner(id: impl Into<ElementId>, cx: &App) -> AnyElement {
    let icon = Icon::default()
        .path("icons/loader-circle.svg")
        .small()
        .text_color(color(ACCENT));
    if cx.reduce_motion() {
        return icon.into_any_element();
    }
    icon.with_animation(
        id,
        Animation::new(Duration::from_millis(1000))
            .repeat()
            .with_max_fps(60.),
        |icon, t| icon.rotate(radians(std::f32::consts::TAU * t)),
    )
    .into_any_element()
}

pub fn value(id: impl Into<ElementId>, target: f32, window: &mut Window, cx: &mut App) -> f32 {
    gpui_base::transition(
        id.into(),
        target,
        gpui_base::Transition::new(Duration::from_millis(VALUE_MS)).ease(ease_out),
        window,
        cx,
    )
}

pub fn progress(
    id: impl Into<ElementId>,
    progress: f32,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let target = if progress.is_finite() {
        progress.clamp(0., 1.)
    } else {
        0.
    };
    let amount = value(id, target, window, cx);
    div()
        .w_full()
        .h(px(5.))
        .rounded_full()
        .overflow_hidden()
        .bg(color(PROGRESS_TRACK))
        .child(
            div()
                .h_full()
                .w(relative(amount))
                .rounded_full()
                .bg(color(PROGRESS_FILL)),
        )
        .into_any_element()
}

pub fn disclosure(
    id: impl Into<ElementId>,
    open: bool,
    content: Div,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    if cx.reduce_motion() {
        return if open {
            content.into_any_element()
        } else {
            div().hidden().into_any_element()
        };
    }
    let id = id.into();
    let amount = gpui_base::transition(
        id.clone(),
        if open { 1_f32 } else { 0_f32 },
        gpui_base::Transition::new(Duration::from_millis(ENTER_MS)).ease(ease_out),
        window,
        cx,
    );
    if amount <= 0.001 && !open {
        return div().hidden().into_any_element();
    }
    gpui_base::MotionReveal::new(id, amount, content.w_full().pb(px(4.)).into_any_element())
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::ease_out;
    #[test]
    fn easing_finishes_exactly_and_preserves_forward_motion() {
        assert_eq!(ease_out(0.), 0.);
        assert_eq!(ease_out(1.), 1.);
        let samples = (0..=100)
            .map(|i| ease_out(i as f32 / 100.))
            .collect::<Vec<_>>();
        assert!(samples.windows(2).all(|p| p[0] <= p[1]));
    }
}
