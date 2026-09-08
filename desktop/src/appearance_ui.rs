//! Appearance preferences with real palette previews and immediate, durable selection.
use super::*;
use crate::palettes::{ALL, Appearance, PaletteId};
use crate::theme::*;
use gpui_component::button::ButtonVariants;

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
    pub(crate) fn appearance_page(
        &self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let preference = self.preferences.application().appearance.clone();
        let current = preference.resolve(theme::system_dark(cx));
        let dark = current.is_dark();
        let cards = ALL
            .into_iter()
            .filter(|id| id.is_dark() == dark)
            .collect::<Vec<_>>();
        // The settings shell is narrower than the window, and text can scale.
        // Size columns from that content area so names have room to wrap.
        let rem = f32::from(window.rem_size());
        let width = crate::views::shell_content_width(Page::Settings, window);
        let columns = (((width + 16.) / (19. * rem + 16.)).floor() as usize).clamp(1, 3);
        let mut grid = v_flex().gap(px(16.));
        for row in cards.chunks(columns) {
            let mut line = h_flex().w_full().gap(px(16.)).items_stretch();
            for &palette in row {
                let selected = palette == current;
                line = line.child(
                    control(("palette", palette as usize))
                        .ghost()
                        .h_auto()
                        .min_h(px(0.))
                        .flex_1()
                        .min_w_0()
                        .p(px(0.))
                        .rounded(RADIUS_CARD)
                        .border_2()
                        .border_color(color(if selected { ACCENT } else { HAIRLINE }))
                        .bg(color(SURFACE))
                        .overflow_hidden()
                        .accessibility_label(format!(
                            "{}，{}{}",
                            palette.name(),
                            if dark { "深色主题" } else { "浅色主题" },
                            if selected { "，已选择" } else { "" }
                        ))
                        .selected(selected)
                        .toggled(selected)
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
                                        .flex_1()
                                        .gap(px(8.))
                                        .p(px(12.))
                                        .items_center()
                                        .child(
                                            v_flex()
                                                .flex_1()
                                                .min_w_0()
                                                .gap(px(4.))
                                                .child(
                                                    div()
                                                        .text_size(TEXT_BODY)
                                                        .font_weight(FontWeight::SEMIBOLD)
                                                        .text_color(color(INK))
                                                        .child(palette.name()),
                                                )
                                                .child(
                                                    div()
                                                        .text_size(TEXT_AUX)
                                                        .text_color(color(GRAY))
                                                        .child(palette.description()),
                                                ),
                                        )
                                        .child(
                                            if selected {
                                                icons::check_circle().text_color(color(ACCENT))
                                            } else {
                                                icons::palette().text_color(color(FAINT))
                                            }
                                            .size(px(18.))
                                            .flex_shrink_0(),
                                        ),
                                ),
                        )
                        .on_click(cx.listener(move |this, _, window, cx| {
                            let mut next = this.application_edit_base();
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
        let modes = h_flex().w_full().gap(px(8.)).children(
            [
                (Appearance::System, "跟随系统", icons::computer()),
                (Appearance::Light, "浅色", icons::sun()),
                (Appearance::Dark, "深色", icons::moon()),
            ]
            .into_iter()
            .enumerate()
            .map(|(index, (mode, label, icon))| {
                seg_item(("appearance-mode", index), preference.mode == mode)
                    .flex_1()
                    .gap(px(8.))
                    .border_1()
                    .border_color(color(if preference.mode == mode {
                        ACCENT
                    } else {
                        SURFACE
                    }))
                    .when(preference.mode == mode, |button| {
                        button
                            .bg(color(ACCENT_SOFT))
                            .text_color(color(ACCENT_STRONG))
                    })
                    .icon(icon)
                    .label(label)
                    .on_click(cx.listener(move |this, _, window, cx| {
                        let mut next = this.application_edit_base();
                        next.appearance.mode = mode;
                        if this.commit_application(next, cx) {
                            window.refresh();
                        }
                    }))
            }),
        );
        v_flex()
            .w_full()
            .gap(px(24.))
            .child(
                accessible_text(
                    "appearance-description",
                    "为工作台和笔记选择你的配色。更改会自动保存。",
                )
                .text_color(color(GRAY)),
            )
            .child(
                v_flex()
                    .w_full()
                    .gap(px(12.))
                    .p(px(16.))
                    .rounded(RADIUS_CARD)
                    .bg(color(SURFACE))
                    .border_1()
                    .border_color(color(CARD_LINE))
                    .child(modes)
                    .child(
                        accessible_text(
                            "appearance-current",
                            if preference.mode == Appearance::System {
                                format!(
                                    "随系统切换：浅色使用 {}，深色使用 {}。",
                                    preference.light.name(),
                                    preference.dark.name()
                                )
                            } else {
                                format!("当前主题：{}", current.name())
                            },
                        )
                        .text_size(TEXT_AUX)
                        .text_color(color(GRAY)),
                    ),
            )
            .child(
                v_flex()
                    .gap(px(16.))
                    .child(
                        accessible_text(
                            "palette-group",
                            if dark { "深色主题" } else { "浅色主题" },
                        )
                        .text_size(TEXT_TITLE)
                        .font_weight(FontWeight::SEMIBOLD),
                    )
                    .child(grid),
            )
            .child(self.appearance_controls(cx))
            .into_any_element()
    }
}
