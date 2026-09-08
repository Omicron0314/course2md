//! Product identity and build provenance, kept separate from operational settings.
use super::*;
use crate::settings_ui::settings_detail_group;
use crate::theme::*;

impl Desktop {
    pub fn about_page(&self, _: &mut Context<Self>) -> AnyElement {
        let commit = env!("COURSE2MD_DESKTOP_COMMIT");
        v_flex()
            .w_full()
            .min_w_0()
            .gap_4()
            .text_color(color(INK))
            .child(
                v_flex()
                    .w_full()
                    .min_w_0()
                    .gap_1()
                    .child(
                        h_flex()
                            .gap_3()
                            .items_baseline()
                            .flex_wrap()
                            .child(
                                accessible_text(
                                    "about-name",
                                    format!("course2md {}", env!("CARGO_PKG_VERSION")),
                                )
                                .role(Role::Heading)
                                .text_size(TEXT_BODY)
                                .font_weight(FontWeight::SEMIBOLD),
                            )
                            .when(!commit.is_empty(), |row| {
                                row.child(
                                    accessible_text("about-build", format!("构建 {commit}"))
                                        .text_size(TEXT_AUX)
                                        .text_color(color(MUTED)),
                                )
                            }),
                    )
                    .child(
                        accessible_text("about-description", "把课程整理成笔记。")
                            .text_size(TEXT_BODY)
                            .text_color(color(MUTED)),
                    ),
            )
            .child(
                h_flex()
                    .gap_2()
                    .flex_wrap()
                    .child(
                        quiet("about-project")
                            .icon(icons::external_link())
                            .label("项目主页")
                            .on_click(|_, _, cx| {
                                cx.open_url("https://github.com/mizorewww/course2md");
                            }),
                    )
                    .child(
                        quiet("about-issue")
                            .icon(icons::edit())
                            .label("发送反馈")
                            .on_click(|_, _, cx| {
                                cx.open_url("https://github.com/mizorewww/course2md/issues");
                            }),
                    ),
            )
            .child(
                settings_detail_group("about-license-title", "开源许可")
                    .child(
                        h_flex()
                            .gap_2()
                            .flex_wrap()
                            .child(
                                quiet("about-license")
                                    .icon(icons::external_link())
                                    .label("course2md · MIT")
                                    .on_click(|_, _, cx| {
                                        cx.open_url(
                                            "https://github.com/mizorewww/course2md/blob/main/LICENSE",
                                        );
                                    }),
                            )
                            .child(
                                quiet("about-icons-license")
                                    .icon(icons::external_link())
                                    .label("Material Icons · Apache 2.0")
                                    .on_click(|_, _, cx| {
                                        cx.open_url(
                                            "https://github.com/google/material-design-icons/blob/master/LICENSE",
                                        );
                                    }),
                            ),
                    ),
            )
            .into_any_element()
    }
}
