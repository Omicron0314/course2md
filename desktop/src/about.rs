//! Product identity and build provenance, kept separate from operational settings.
use super::*;
use crate::theme::{INK, LINE, MUTED, accessible_text, control};

impl Desktop {
    pub fn about_page(&self, _: &mut Context<Self>) -> AnyElement {
        let commit = env!("COURSE2MD_DESKTOP_COMMIT");
        v_flex()
            .w_full()
            .max_w(px(760.))
            .gap_6()
            .text_color(rgb(INK))
            .child(
                v_flex()
                    .gap_2()
                    .child(
                        accessible_text("about-name", "course2md")
                            .role(Role::Heading)
                            .text_xl()
                            .font_weight(FontWeight::SEMIBOLD),
                    )
                    .child(accessible_text("about-description", "把课程整理成笔记。").text_color(rgb(MUTED))),
            )
            .child(
                v_flex()
                    .gap_3()
                    .child(about_detail("版本", env!("CARGO_PKG_VERSION")))
                    .when(!commit.is_empty(), |view| {
                        view.child(about_detail("构建", commit))
                    }),
            )
            .child(
                h_flex()
                    .gap_3()
                    .flex_wrap()
                    .child(
                        control("about-project")

                            .label("项目主页")
                            .icon(IconName::ExternalLink)
                            .on_click(|_, _, cx| {
                                cx.open_url("https://github.com/mizorewww/course2md");
                            }),
                    )
                    .child(
                        control("about-issue")

                            .label("反馈问题")
                            .icon(IconName::ExternalLink)
                            .on_click(|_, _, cx| {
                                cx.open_url("https://github.com/mizorewww/course2md/issues");
                            }),
                    ),
            )
            .child(
                v_flex()
                    .pt_6()
                    .gap_3()
                    .border_t_1()
                    .border_color(rgb(LINE))
                    .child(accessible_text("about-license-title", "开源许可").role(Role::Heading).font_weight(FontWeight::SEMIBOLD))
                    .child(
                        h_flex()
                            .gap_3()
                            .flex_wrap()
                            .child(
                                control("about-license")

                                    .label("course2md · MIT")
                            .icon(IconName::ExternalLink)
                                    .on_click(|_, _, cx| {
                                        cx.open_url("https://github.com/mizorewww/course2md/blob/main/LICENSE");
                                    }),
                            )
                            .child(
                                control("about-icons-license")

                                    .label("Material Icons · Apache 2.0")
                            .icon(IconName::ExternalLink)
                                    .on_click(|_, _, cx| {
                                        cx.open_url("https://github.com/google/material-design-icons/blob/master/LICENSE");
                                    }),
                            ),
                    ),
            )
            .into_any_element()
    }
}

fn about_detail(label: &'static str, value: &'static str) -> Div {
    h_flex()
        .gap_4()
        .child(
            accessible_text(SharedString::from(format!("about-{label}-label")), label)
                .min_w(rems(4.6))
                .flex_shrink_0()
                .text_color(rgb(MUTED)),
        )
        .child(
            accessible_text(SharedString::from(format!("about-{label}-value")), value)
                .font_weight(FontWeight::MEDIUM),
        )
}
