//! A single controlled selection with native radio semantics and roving keyboard focus.
use gpui::prelude::FluentBuilder as _;
use gpui::*;
use gpui_component::Icon;
use std::{collections::BTreeMap, rc::Rc};

actions!(
    course2md_choices,
    [NextChoice, PreviousChoice, FirstChoice, LastChoice]
);

pub(super) fn init(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("right", NextChoice, Some("SingleChoiceGroup")),
        KeyBinding::new("down", NextChoice, Some("SingleChoiceGroup")),
        KeyBinding::new("left", PreviousChoice, Some("SingleChoiceGroup")),
        KeyBinding::new("up", PreviousChoice, Some("SingleChoiceGroup")),
        KeyBinding::new("home", FirstChoice, Some("SingleChoiceGroup")),
        KeyBinding::new("end", LastChoice, Some("SingleChoiceGroup")),
    ]);
}

type Change = Rc<dyn Fn(&SharedString, &mut Window, &mut App)>;
#[derive(Clone)]
struct OptionItem {
    value: SharedString,
    label: SharedString,
    disabled: bool,
}

#[derive(IntoElement)]
pub struct SingleChoiceGroup {
    id: ElementId,
    label: SharedString,
    value: Option<SharedString>,
    options: Vec<OptionItem>,
    icons: BTreeMap<SharedString, Icon>,
    full_width: bool,
    disabled: bool,
    on_change: Option<Change>,
    reveal_in: Option<ScrollHandle>,
}
impl SingleChoiceGroup {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            value: None,
            options: Vec::new(),
            icons: BTreeMap::new(),
            full_width: false,
            disabled: false,
            on_change: None,
            reveal_in: None,
        }
    }
    pub fn options<K: Into<SharedString>, V: Into<SharedString>>(
        mut self,
        options: impl IntoIterator<Item = (K, V)>,
    ) -> Self {
        self.options = options
            .into_iter()
            .map(|(value, label)| OptionItem {
                value: value.into(),
                label: label.into(),
                disabled: false,
            })
            .collect();
        self
    }
    pub fn selected(mut self, value: impl Into<SharedString>) -> Self {
        self.value = Some(value.into());
        self
    }
    /// Fill the parent's width with equal segments and one moving selection surface.
    pub fn full_width(mut self) -> Self {
        self.full_width = true;
        self
    }
    /// Attach an icon by option value; this may be called before or after `options`.
    pub fn icon(mut self, value: impl Into<SharedString>, icon: impl Into<Icon>) -> Self {
        self.icons.insert(value.into(), icon.into());
        self
    }
    pub fn reveal_in(mut self, scroll: ScrollHandle) -> Self {
        self.reveal_in = Some(scroll);
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    pub fn disable_option(mut self, value: impl AsRef<str>) -> Self {
        for option in &mut self.options {
            if option.value.as_ref() == value.as_ref() {
                option.disabled = true;
            }
        }
        self
    }
    pub fn on_change(
        mut self,
        callback: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(callback));
        self
    }
}

#[derive(Clone)]
struct Navigation {
    options: Vec<OptionItem>,
    focus: Vec<FocusHandle>,
    selected: Option<usize>,
    on_change: Option<Change>,
}
#[derive(Clone, Copy)]
enum Direction {
    Next,
    Previous,
    First,
    Last,
}
fn destination(enabled: &[usize], current: Option<usize>, direction: Direction) -> Option<usize> {
    if enabled.is_empty() {
        return None;
    }
    let position = current.and_then(|current| enabled.iter().position(|index| *index == current));
    Some(match direction {
        Direction::First => enabled[0],
        Direction::Last => *enabled.last().unwrap(),
        Direction::Next => enabled[position.map_or(0, |index| (index + 1) % enabled.len())],
        Direction::Previous => {
            enabled[position.map_or(enabled.len() - 1, |index| {
                (index + enabled.len() - 1) % enabled.len()
            })]
        }
    })
}
impl Navigation {
    fn navigate(&self, direction: Direction, window: &mut Window, cx: &mut App) {
        let enabled = self
            .options
            .iter()
            .enumerate()
            .filter(|(_, option)| !option.disabled)
            .map(|(index, _)| index)
            .collect::<Vec<_>>();
        let current = self
            .focus
            .iter()
            .position(|focus| focus.is_focused(window))
            .or(self.selected);
        let Some(index) = destination(&enabled, current, direction) else {
            return;
        };
        self.focus[index].focus(window, cx);
        if self.selected != Some(index)
            && let Some(on_change) = &self.on_change
        {
            on_change(&self.options[index].value, window, cx);
        }
    }
}
impl RenderOnce for SingleChoiceGroup {
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        if self.disabled {
            self.options
                .iter_mut()
                .for_each(|option| option.disabled = true);
        }
        let state = window.use_keyed_state(self.id.clone(), cx, |_, _| {
            BTreeMap::<SharedString, FocusHandle>::new()
        });
        let handles = state.update(cx, |handles, cx| {
            self.options
                .iter()
                .map(|option| {
                    handles
                        .entry(option.value.clone())
                        .or_insert_with(|| cx.focus_handle())
                        .clone()
                })
                .collect::<Vec<_>>()
        });
        let selected = self
            .options
            .iter()
            .position(|option| Some(&option.value) == self.value.as_ref());
        let entry = selected
            .filter(|index| !self.options[*index].disabled)
            .or_else(|| self.options.iter().position(|option| !option.disabled));
        let navigation = Navigation {
            options: self.options.clone(),
            focus: handles.clone(),
            selected,
            on_change: self.on_change.clone(),
        };
        let next = navigation.clone();
        let previous = navigation.clone();
        let first = navigation.clone();
        let last = navigation;
        let count = self.options.len();
        let full_width = self.full_width;
        let position = if full_width && count > 0 {
            // The channel belongs to the group, not the chosen value. Retargets
            // continue from the currently painted position, including reversals.
            crate::motion::value(
                ElementId::NamedChild(self.id.clone().into(), "selection-position".into()),
                selected.unwrap_or(0) as f32,
                window,
                cx,
            )
            .clamp(0., (count - 1) as f32)
        } else {
            0.
        };
        let icons = self.icons;
        let radios = self.options.into_iter().enumerate().map(|(index, option)| {
            let checked = selected == Some(index);
            let on_change = self.on_change.clone();
            let focus = handles[index].clone();
            let icon = icons.get(&option.value).cloned();
            let background = super::color(if checked {
                super::SURFACE
            } else {
                super::SEGMENT_TRACK
            });
            let hover_background = if full_width {
                super::color(super::HOVER_WARM).opacity(0.5)
            } else {
                super::blend(background, super::color(super::HOVER_WARM), 0.65)
            };
            let active_background = if full_width {
                super::color(super::ACCENT_SOFT).opacity(0.5)
            } else {
                super::blend(background, super::color(super::ACCENT_SOFT), 0.65)
            };
            gpui_base::Radio::new(option.value.clone())
                .checked(checked)
                .disabled(option.disabled)
                .accessibility_label(if option.disabled {
                    SharedString::from(format!("{}，当前不可用", option.label))
                } else {
                    option.label.clone()
                })
                .set_position(index + 1, count)
                .track_focus(&handles[index])
                .tab_stop(entry == Some(index))
                .min_h(rems(2.286))
                .min_w_0()
                .max_w_full()
                .h_auto()
                .px(px(14.))
                .py(px(2.))
                .rounded_full()
                .border_2()
                .border_color(gpui::transparent_black())
                .text_size(rems(1.))
                .bg(background)
                .text_color(super::color(if checked { super::INK } else { super::GRAY }))
                .when(checked && !full_width, |radio| {
                    radio
                        .font_weight(FontWeight::SEMIBOLD)
                        .shadow(super::shadow_segment_selected())
                })
                .when(full_width, |radio| {
                    radio
                        .relative()
                        .flex()
                        .items_center()
                        .justify_center()
                        .flex_1()
                        .min_h(px(40.))
                        .px(px(12.))
                        .bg(gpui::transparent_black())
                        // Equal weight keeps text in place while the surface moves.
                        .font_weight(FontWeight::MEDIUM)
                        .when(cfg!(test), |radio| {
                            radio.debug_selector(move || {
                                format!("full-choice-option-{index}").into()
                            })
                        })
                })
                .when(!option.disabled, |radio| {
                    radio
                        .hover(|style| style.bg(hover_background))
                        .active(|style| style.bg(active_background))
                })
                .when(option.disabled, |radio| radio.opacity(0.55))
                .focus(|style| style.border_color(super::color(super::INK)).shadow_sm())
                .when(full_width || icon.is_some(), |radio| {
                    radio.child(
                        gpui_base::h_flex()
                            .min_w_0()
                            .gap(px(8.))
                            .items_center()
                            .when_some(icon.clone(), |row, icon| {
                                row.child(icon.size(px(18.)).flex_shrink_0())
                            })
                            .child(
                                div()
                                    .min_w_0()
                                    .whitespace_nowrap()
                                    .text_ellipsis()
                                    .child(option.label.clone()),
                            ),
                    )
                })
                .when(!full_width && icon.is_none(), |radio| {
                    radio.child(option.label)
                })
                .on_change(move |_, _, window, cx| {
                    focus.focus(window, cx);
                    if !checked && let Some(on_change) = &on_change {
                        on_change(&option.value, window, cx);
                    }
                })
        });
        // v_flex 列里子项默认横向拉伸；包一层 h_flex 让轨道按内容收宽。
        let reveal_id = SharedString::from(format!("choice-reveal-{:?}", self.id));
        let group = gpui_base::h_flex().w_full().min_w_0().child(
            gpui_base::RadioGroup::new(self.id)
                .aria_label(self.label)
                .axis(Axis::Horizontal)
                .key_context("SingleChoiceGroup")
                .flex()
                .flex_row()
                .flex_wrap()
                .self_start()
                .gap(px(2.))
                .p(px(2.))
                .rounded_full()
                .bg(super::color(super::SEGMENT_TRACK))
                .max_w_full()
                .when(full_width, |group| {
                    group.w_full().when(cfg!(test), |group| {
                        group.debug_selector(|| "full-choice-track".into())
                    })
                })
                .on_action(move |_: &NextChoice, window, cx| {
                    next.navigate(Direction::Next, window, cx)
                })
                .on_action(move |_: &PreviousChoice, window, cx| {
                    previous.navigate(Direction::Previous, window, cx)
                })
                .on_action(move |_: &FirstChoice, window, cx| {
                    first.navigate(Direction::First, window, cx)
                })
                .on_action(move |_: &LastChoice, window, cx| {
                    last.navigate(Direction::Last, window, cx)
                })
                .map(|group| {
                    if full_width {
                        // Percentages resolve against this padding-free lane on
                        // every layout. Resizing needs no measured/cached bounds.
                        group.child(
                            gpui_base::h_flex()
                                .relative()
                                .w_full()
                                .min_w_0()
                                .items_stretch()
                                .when(cfg!(test), |lane| {
                                    lane.debug_selector(|| "full-choice-lane".into())
                                })
                                .when(selected.is_some() && count > 0, |lane| {
                                    lane.child(
                                        div()
                                            .absolute()
                                            .top(px(0.))
                                            .bottom(px(0.))
                                            .left(relative(position / count as f32))
                                            .w(relative(1. / count as f32))
                                            .rounded_full()
                                            .bg(super::color(super::SURFACE))
                                            .shadow(super::shadow_segment_selected())
                                            .when(cfg!(test), |indicator| {
                                                indicator.debug_selector(|| {
                                                    "full-choice-indicator".into()
                                                })
                                            }),
                                    )
                                })
                                .children(radios),
                        )
                    } else {
                        group.children(radios)
                    }
                }),
        );
        if let Some(scroll) = self.reveal_in {
            crate::focus_scroll::RevealFocus::new(reveal_id, group, scroll).into_any_element()
        } else {
            group.into_any_element()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Direction, SingleChoiceGroup, destination};
    use gpui::{
        Bounds, Context, IntoElement, Modifiers, ParentElement as _, Pixels, Render, SharedString,
        Styled as _, TestAppContext, VisualTestContext, Window, div, px,
    };
    use std::time::Duration;

    struct FullWidthHarness {
        count: usize,
        selected: usize,
        width: Pixels,
        changes: usize,
    }

    impl Render for FullWidthHarness {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            div().w(self.width).child(
                SingleChoiceGroup::new("full-width-choice", "文字大小")
                    .options(
                        (0..self.count)
                            .map(|index| (index.to_string(), format!("{}%", 100 + 25 * index))),
                    )
                    .selected(self.selected.to_string())
                    .full_width()
                    .on_change(cx.listener(|this, next: &SharedString, _, cx| {
                        this.selected = next.parse().unwrap();
                        this.changes += 1;
                        cx.notify();
                    })),
            )
        }
    }

    const OPTION_SELECTORS: [&str; 4] = [
        "full-choice-option-0",
        "full-choice-option-1",
        "full-choice-option-2",
        "full-choice-option-3",
    ];

    fn choice_bounds(cx: &mut VisualTestContext, name: &'static str) -> Bounds<Pixels> {
        cx.debug_bounds(name)
            .unwrap_or_else(|| panic!("missing {name}"))
    }

    fn draw_choice(cx: &mut VisualTestContext) {
        cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        });
    }

    fn assert_inside_lane(cx: &mut VisualTestContext, count: usize, width: Pixels) {
        let track = choice_bounds(cx, "full-choice-track");
        let lane = choice_bounds(cx, "full-choice-lane");
        let indicator = choice_bounds(cx, "full-choice-indicator");
        assert_eq!(track.size.width, width);
        assert_eq!(lane.left() - track.left(), px(2.));
        assert_eq!(track.right() - lane.right(), px(2.));
        assert_eq!(lane.top() - track.top(), px(2.));
        assert_eq!(track.bottom() - lane.bottom(), px(2.));
        assert!(indicator.left() >= lane.left() - px(0.5));
        assert!(indicator.right() <= lane.right() + px(0.5));
        assert_eq!(indicator.top(), lane.top());
        assert_eq!(indicator.bottom(), lane.bottom());
        for index in 0..count {
            let option = choice_bounds(cx, OPTION_SELECTORS[index]);
            assert!((option.size.width - lane.size.width / count as f32).abs() <= px(0.5));
            assert!((indicator.size.width - option.size.width).abs() <= px(0.5));
        }
    }

    fn exercise_full_width_motion(cx: &mut TestAppContext, count: usize) {
        let (view, cx) = cx.add_window_view(|_, _| FullWidthHarness {
            count,
            selected: 0,
            width: px(420.),
            changes: 0,
        });
        draw_choice(cx);
        assert_inside_lane(cx, count, px(420.));
        let start = choice_bounds(cx, "full-choice-indicator");
        let options = (0..count)
            .map(|index| choice_bounds(cx, OPTION_SELECTORS[index]))
            .collect::<Vec<_>>();
        let destination = options[count - 1];

        cx.simulate_click(destination.center(), Modifiers::default());
        cx.update(|window, cx| {
            assert_eq!(
                view.read(cx).selected,
                count - 1,
                "click is accepted immediately"
            );
            assert_eq!(view.read(cx).changes, 1);
            window.draw(cx).clear(cx);
        });
        assert_eq!(
            choice_bounds(cx, "full-choice-indicator").left(),
            start.left()
        );
        cx.simulate_click(destination.center(), Modifiers::default());
        cx.update(|_, cx| assert_eq!(view.read(cx).changes, 1, "same choice does not save twice"));

        cx.executor().advance_clock(Duration::from_millis(70));
        draw_choice(cx);
        let middle = choice_bounds(cx, "full-choice-indicator");
        assert!(
            middle.left() > start.left(),
            "selection must leave its starting cell"
        );
        assert!(
            middle.left() < destination.left(),
            "selection must expose an intermediate frame"
        );
        for (index, before) in options.into_iter().enumerate() {
            assert_eq!(choice_bounds(cx, OPTION_SELECTORS[index]), before);
        }
        draw_choice(cx);
        assert_eq!(
            choice_bounds(cx, "full-choice-indicator"),
            middle,
            "ordinary redraw must not restart motion"
        );

        // Reverse before settling, then resize without advancing time. Neither
        // operation may teleport the current selection or use old geometry.
        cx.simulate_click(start.center(), Modifiers::default());
        cx.update(|window, cx| window.draw(cx).clear(cx));
        assert_eq!(
            choice_bounds(cx, "full-choice-indicator").left(),
            middle.left()
        );
        cx.update(|window, cx| {
            view.update(cx, |view, cx| {
                view.width = px(284.);
                cx.notify();
            });
            window.draw(cx).clear(cx);
        });
        assert_inside_lane(cx, count, px(284.));
        let resized = choice_bounds(cx, "full-choice-indicator");
        cx.executor().advance_clock(Duration::from_millis(35));
        draw_choice(cx);
        let returning = choice_bounds(cx, "full-choice-indicator");
        assert!(returning.left() < resized.left());
        assert!(returning.left() > choice_bounds(cx, "full-choice-lane").left());
        assert_inside_lane(cx, count, px(284.));

        cx.executor().advance_clock(Duration::from_millis(300));
        cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
            window.simulate_next_frame(cx);
            window.refresh();
            window.draw(cx).clear(cx);
            assert_eq!(
                window.simulate_next_frame(cx),
                0,
                "settled control must stop scheduling frames"
            );
        });
        assert_eq!(
            choice_bounds(cx, "full-choice-indicator").left(),
            choice_bounds(cx, "full-choice-lane").left()
        );
    }

    #[gpui::test]
    fn full_width_choices_move_inside_current_geometry_and_settle(cx: &mut TestAppContext) {
        for count in [3, 4] {
            exercise_full_width_motion(cx, count);
        }
    }

    #[gpui::test]
    fn full_width_choices_respect_reduce_motion_on_the_changed_frame(cx: &mut TestAppContext) {
        cx.update(|cx| cx.set_reduce_motion(true));
        let (view, cx) = cx.add_window_view(|_, _| FullWidthHarness {
            count: 4,
            selected: 0,
            width: px(420.),
            changes: 0,
        });
        draw_choice(cx);
        let destination = choice_bounds(cx, "full-choice-option-3");
        cx.simulate_click(destination.center(), Modifiers::default());
        cx.update(|window, cx| {
            assert_eq!(view.read(cx).selected, 3);
            window.draw(cx).clear(cx);
            assert_eq!(window.simulate_next_frame(cx), 0);
        });
        assert_eq!(
            choice_bounds(cx, "full-choice-indicator").left(),
            destination.left()
        );
        assert_inside_lane(cx, 4, px(420.));
    }

    #[test]
    fn arrows_skip_disabled_values_wrap_and_start_when_selection_is_unknown() {
        let enabled = [0, 2, 3];
        assert_eq!(destination(&enabled, Some(0), Direction::Next), Some(2));
        assert_eq!(destination(&enabled, Some(3), Direction::Next), Some(0));
        assert_eq!(destination(&enabled, Some(0), Direction::Previous), Some(3));
        assert_eq!(destination(&enabled, None, Direction::Next), Some(0));
        assert_eq!(destination(&enabled, None, Direction::Previous), Some(3));
        assert_eq!(destination(&enabled, Some(1), Direction::First), Some(0));
        assert_eq!(destination(&enabled, Some(1), Direction::Last), Some(3));
        assert_eq!(destination(&[], Some(0), Direction::Next), None);
    }
}
