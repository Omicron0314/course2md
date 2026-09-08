//! Reveal newly focused form rows without pinning them during manual scrolling.
use gpui::{prelude::*, *};

#[derive(IntoElement)]
pub struct RevealFocus {
    id: ElementId,
    child: AnyElement,
    scroll: ScrollHandle,
}

struct State {
    container: FocusHandle,
    focused: Option<FocusHandle>,
    geometry: Option<(Pixels, Size<Pixels>)>,
}

impl RevealFocus {
    pub fn new(id: impl Into<ElementId>, child: impl IntoElement, scroll: ScrollHandle) -> Self {
        Self {
            id: id.into(),
            child: child.into_any_element(),
            scroll,
        }
    }
}

impl RenderOnce for RevealFocus {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = window.use_keyed_state(self.id.clone(), cx, |_, cx| State {
            container: cx.focus_handle(),
            focused: None,
            geometry: None,
        });
        let focus = state.read(cx).container.clone();
        div()
            .on_children_prepainted(move |bounds, window, cx| {
                let Some(bounds) = bounds.first().copied() else {
                    return;
                };
                let state = state.clone();
                let scroll = self.scroll.clone();
                let geometry = (window.rem_size(), window.bounds().size);
                // Focus ancestry is only reliable after this frame has committed:
                // prepaint moves reused subtrees out of the previous dispatch tree.
                // This also covers a focus change while a saved-status row disappears.
                window.defer(cx, move |window, cx| {
                    let focused = window
                        .focused(cx)
                        .filter(|_| state.read(cx).container.contains_focused(window, cx));
                    state.update(cx, |state, _| {
                        let reveal = focused.is_some()
                            && (state.focused != focused || state.geometry != Some(geometry));
                        state.focused = focused;
                        state.geometry = Some(geometry);
                        if !reveal {
                            return;
                        }
                        let viewport = scroll.bounds();
                        if viewport.size.height <= px(0.) {
                            return;
                        }
                        let delta = reveal_delta(
                            f32::from(bounds.top()),
                            f32::from(bounds.bottom()),
                            f32::from(viewport.top()) + 8.,
                            f32::from(viewport.bottom()) - 8.,
                        );
                        if delta != 0. {
                            scroll.set_offset(scroll.offset() + point(px(0.), px(delta)));
                            window.refresh();
                        }
                    });
                });
            })
            .id(self.id)
            .track_focus(&focus)
            .w_full()
            .min_w_0()
            .child(self.child)
    }
}

fn reveal_delta(top: f32, bottom: f32, visible_top: f32, visible_bottom: f32) -> f32 {
    if top < visible_top || bottom - top > visible_bottom - visible_top {
        visible_top - top
    } else if bottom > visible_bottom {
        visible_bottom - bottom
    } else {
        0.
    }
}

#[cfg(test)]
mod tests {
    use super::reveal_delta;

    #[test]
    fn reveals_clipped_controls_and_the_start_of_oversized_rows() {
        assert_eq!(reveal_delta(100., 160., 80., 400.), 0.);
        assert_eq!(reveal_delta(30., 100., 80., 400.), 50.);
        assert_eq!(reveal_delta(380., 440., 80., 400.), -40.);
        assert_eq!(reveal_delta(300., 900., 80., 400.), -220.);
    }
}
