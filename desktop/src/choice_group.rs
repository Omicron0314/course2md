//! A single controlled selection with native radio semantics and roving keyboard focus.
use gpui::prelude::FluentBuilder as _;
use gpui::*;
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
                .children(self.options.into_iter().enumerate().map(|(index, option)| {
                    let checked = selected == Some(index);
                    let on_change = self.on_change.clone();
                    let focus = handles[index].clone();
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
                        .bg(super::color(if checked {
                            super::SURFACE
                        } else {
                            super::SEGMENT_TRACK
                        }))
                        .text_color(super::color(if checked { super::INK } else { super::GRAY }))
                        .when(checked, |radio| {
                            radio
                                .font_weight(FontWeight::SEMIBOLD)
                                .shadow(super::shadow_segment_selected())
                        })
                        .when(option.disabled, |radio| radio.opacity(0.55))
                        .focus(|style| style.border_color(super::color(super::INK)).shadow_sm())
                        .child(option.label)
                        .on_change(move |_, _, window, cx| {
                            focus.focus(window, cx);
                            if let Some(on_change) = &on_change {
                                on_change(&option.value, window, cx);
                            }
                        })
                })),
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
    use super::{Direction, destination};
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
