//! Product identity and build provenance, kept separate from operational settings.
use super::*;
use crate::theme::*;

impl Desktop {
    pub fn about_page(&self, _: &mut Context<Self>) -> AnyElement {
        let commit = env!("COURSE2MD_DESKTOP_COMMIT");
        v_flex()
            .w_full()
            .gap_2()
            .p_4()
            .bg(color(SURFACE))
            .border_1()
            .border_color(color(CARD_LINE))
            .rounded(RADIUS_CARD)
            .text_color(color(INK))
            .child(
                h_flex()
                    .gap_2()
                    .items_baseline()
                    .flex_wrap()
                    .child(
                        accessible_text(
                            "about-name",
                            format!("course2md {}", env!("CARGO_PKG_VERSION")),
                        )
                        .role(Role::Heading)
                        .font_weight(FontWeight::SEMIBOLD),
                    )
                    .when(!commit.is_empty(), |row| {
                        row.child(
                            accessible_text("about-build", format!("构建 {commit}"))
                                .text_size(TEXT_AUX)
                                .text_color(color(GRAY)),
                        )
                    }),
            )
            .child(
                accessible_text("about-description", "把课程整理成笔记。")
                    .text_size(TEXT_AUX)
                    .text_color(color(GRAY)),
            )
            .child(
                h_flex()
                    .gap_2()
                    .flex_wrap()
                    .child(div().flex_1())
                    .child(
                        quiet("about-issue").label("发送反馈").on_click(|_, _, cx| {
                            cx.open_url("https://github.com/mizorewww/course2md/issues");
                        }),
                    )
                    .child(
                        quiet("about-project").label("帮助").on_click(|_, _, cx| {
                            cx.open_url("https://github.com/mizorewww/course2md");
                        }),
                    ),
            )
            .child(
                h_flex()
                    .gap_2()
                    .flex_wrap()
                    .child(
                        accessible_text("about-license-title", "开源许可")
                            .text_size(TEXT_AUX)
                            .text_color(color(GRAY)),
                    )
                    .child(
                        quiet("about-license").label("course2md · MIT").on_click(|_, _, cx| {
                            cx.open_url(
                                "https://github.com/mizorewww/course2md/blob/main/LICENSE",
                            );
                        }),
                    )
                    .child(
                        quiet("about-icons-license")
                            .label("Material Icons · Apache 2.0")
                            .on_click(|_, _, cx| {
                                cx.open_url(
                                    "https://github.com/google/material-design-icons/blob/master/LICENSE",
                                );
                            }),
                    ),
            )
            .into_any_element()
    }
}
