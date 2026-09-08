//! Appearance preferences with real palette previews and immediate, durable selection.
use super::*;
use crate::palettes::{ALL, Appearance, PaletteId};
use crate::theme::*;

fn sample(palette: PaletteId) -> Div {
    let p = palette.colors();
    // These are the selected palette's actual preview colors, not application chrome.
    div()
        .w_full()
        .h(px(96.))
        .flex_shrink_0()
        .rounded_t(RADIUS_CARD)
        .overflow_hidden()
        .bg(rgb(p.canvas))
        .p(px(12.))
        .child(
            h_flex()
                .h(px(16.))
                .items_center()
                .gap(px(4.))
                .children(
                    [p.accent, p.success, p.warning]
                        .into_iter()
                        .map(|c| div().size(px(5.)).rounded_full().bg(rgb(c))),
                )
                .child(
                    div()
                        .ml(px(8.))
                        .h(px(4.))
                        .w(px(36.))
                        .rounded_full()
                        .bg(rgb(p.muted)),
                ),
        )
        .child(
            h_flex()
                .mt(px(8.))
                .gap(px(8.))
                .items_start()
                .child(
                    div()
                        .w(px(30.))
                        .h(px(44.))
                        .rounded(px(5.))
                        .bg(rgb(p.inset))
                        .child(
                            div()
                                .mx(px(5.))
                                .mt(px(7.))
                                .h(px(5.))
                                .rounded(px(2.))
                                .bg(rgb(p.accent)),
                        ),
                )
                .child(
                    v_flex()
                        .flex_1()
                        .h(px(44.))
                        .p(px(6.))
                        .gap(px(4.))
                        .rounded(px(6.))
                        .bg(rgb(p.surface))
                        .child(
                            div()
                                .h(px(5.))
                                .w(relative(0.65))
                                .rounded_full()
                                .bg(rgb(p.text)),
                        )
                        .child(
                            div()
                                .h(px(4.))
                                .w(relative(0.9))
                                .rounded_full()
                                .bg(rgb(p.border)),
                        )
                        .child(
                            div()
                                .h(px(12.))
                                .w(px(32.))
                                .rounded(px(3.))
                                .bg(rgb(p.accent)),
                        ),
                ),
        )
}

impl Desktop {
    fn palette_grid(
        &self,
        dark: bool,
        selected_palette: PaletteId,
        width: f32,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Div {
        let cards = ALL
            .into_iter()
            .filter(|id| id.is_dark() == dark)
            .collect::<Vec<_>>();
        let rem = f32::from(window.rem_size());
        let minimum_card_width = if dark { 200. } else { 180. };
        let available =
            (((width + 16.) / (minimum_card_width * rem / 14. + 16.)).floor() as usize).clamp(1, 4);
        // Keep complete rows when four/six presets do not divide into the
        // available columns, rather than leaving one isolated preview below.
        let columns =
            if (cards.len() == 4 && available == 3) || (cards.len() == 6 && available == 4) {
                available - 1
            } else {
                available
            };
        let mut grid = v_flex().w_full().min_w_0().gap(px(16.));
        for row in cards.chunks(columns) {
            let mut line = h_flex().w_full().min_w_0().gap(px(16.)).items_stretch();
            for &palette in row {
                let selected = palette == selected_palette;
                let amount = crate::motion::value(
                    ("palette-selection", palette as usize),
                    if selected { 1. } else { 0. },
                    window,
                    cx,
                );
                line = line.child(
                    selection_card(("palette", palette as usize), selected, amount)
                        .flex_1()
                        .p(px(0.))
                        .overflow_hidden()
                        .accessibility_label(format!(
                            "{}，{}{}",
                            palette.name(),
                            if dark { "深色主题" } else { "浅色主题" },
                            if selected { "，已选择" } else { "" }
                        ))
                        .child(
                            v_flex()
                                .w_full()
                                .min_w_0()
                                .whitespace_normal()
                                .text_left()
                                .child(sample(palette))
                                .child(
                                    h_flex()
                                        .w_full()
                                        .min_h(rems(3.43))
                                        .flex_1()
                                        .gap(px(8.))
                                        .p(px(12.))
                                        .items_center()
                                        .bg(theme::blend(
                                            color(SURFACE),
                                            color(ACCENT_SOFT),
                                            amount,
                                        ))
                                        .child(
                                            div()
                                                .flex_1()
                                                .min_w_0()
                                                .text_size(TEXT_BODY)
                                                .font_weight(FontWeight::SEMIBOLD)
                                                .text_color(color(INK))
                                                .child(palette.name()),
                                        )
                                        .child(
                                            icons::check_circle()
                                                .text_color(color(ACCENT))
                                                .size(px(18.))
                                                .opacity(amount)
                                                .flex_shrink_0(),
                                        ),
                                ),
                        )
                        .on_click(cx.listener(move |this, _, window, cx| {
                            let mut next = this.application_edit_base();
                            let previous = if dark {
                                next.appearance.dark
                            } else {
                                next.appearance.light
                            };
                            if previous == palette {
                                return;
                            }
                            next.appearance.select(palette);
                            if this.commit_application(next, cx) {
                                window.refresh();
                            }
                        })),
                );
            }
            for _ in row.len()..columns {
                line = line.child(div().flex_1());
            }
            grid = grid.child(line);
        }
        grid
    }

    pub(crate) fn appearance_page(
        &self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let preference = self.preferences.application().appearance.clone();
        let width = crate::views::settings_content_width(window);
        let modes = SingleChoiceGroup::new("appearance-mode", "外观模式")
            .options([("system", "跟随系统"), ("light", "浅色"), ("dark", "深色")])
            .icon("system", icons::computer())
            .icon("light", icons::sun())
            .icon("dark", icons::moon())
            .full_width()
            .selected(match preference.mode {
                Appearance::System => "system",
                Appearance::Light => "light",
                Appearance::Dark => "dark",
            })
            .reveal_in(self.scrolls[Page::Settings as usize].clone())
            .on_change(cx.listener(|this, selected: &SharedString, window, cx| {
                let mode = match selected.as_ref() {
                    "light" => Appearance::Light,
                    "dark" => Appearance::Dark,
                    _ => Appearance::System,
                };
                let mut next = this.application_edit_base();
                if next.appearance.mode == mode {
                    return;
                }
                next.appearance.mode = mode;
                if this.commit_application(next, cx) {
                    window.refresh();
                }
            }));
        let mut view = v_flex()
            .w_full()
            .min_w_0()
            .gap(px(28.))
            .child(div().w_full().max_w(CONTROL_GROUP_MAX).child(modes));
        // The two saved palettes remain editable without turning system mode
        // off. Section labels identify appearance; cards only need their names.
        for dark in [false, true] {
            if (preference.mode == Appearance::Light && dark)
                || (preference.mode == Appearance::Dark && !dark)
            {
                continue;
            }
            view = view.child(
                v_flex()
                    .w_full()
                    .min_w_0()
                    .gap(px(16.))
                    .child(
                        accessible_text(
                            ("palette-group", usize::from(dark)),
                            if dark { "深色主题" } else { "浅色主题" },
                        )
                        .role(Role::Heading)
                        .text_size(TEXT_TITLE)
                        .font_weight(FontWeight::SEMIBOLD),
                    )
                    .child(self.palette_grid(
                        dark,
                        if dark {
                            preference.dark
                        } else {
                            preference.light
                        },
                        width,
                        window,
                        cx,
                    )),
            );
        }
        view.child(self.appearance_controls(cx)).into_any_element()
    }
}
