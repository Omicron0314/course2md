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
    mounted: bool,
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
            mounted: false,
        });
        let focus = state.read(cx).container.clone();
        // Read ancestry before prepaint moves reused dispatch subtrees out of the
        // previous frame. During prepaint, contains_focused can otherwise miss them.
        let focused = window
            .focused(cx)
            .filter(|_| focus.contains_focused(window, cx));
        div()
            .on_children_prepainted(move |bounds, window, cx| {
                let Some(bounds) = bounds.first() else { return };
                state.update(cx, |state, cx| {
                    let geometry = (window.rem_size(), window.bounds().size);
                    let reveal = focused.is_some()
                        && (state.focused != focused || state.geometry != Some(geometry));
                    state.focused = focused.clone();
                    state.geometry = Some(geometry);
                    if !state.mounted {
                        state.mounted = true;
                        window.defer(cx, |window, _| window.refresh());
                    }
                    if !reveal {
                        return;
                    }
                    let viewport = self.scroll.bounds();
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
                        let scroll = self.scroll.clone();
                        let offset = scroll.offset() + point(px(0.), px(delta));
                        window.defer(cx, move |window, _| {
                            scroll.set_offset(offset);
                            window.refresh();
                        });
                    }
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
