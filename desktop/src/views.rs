//! Native pages: stable navigation and actions surround independently scrolling content.
use super::*;
use crate::theme::*;
use gpui_component::button::*;

/// Every chrome row and page body sits in this one centered column.
fn shell_column() -> Div {
    div()
        .w_full()
        .min_w_0()
        .max_w(COLUMN)
        .mx_auto()
        .px(px(24.))
}

impl Desktop {
    /// App top bar under the title bar: wordmark, workspace/notes tabs, settings gear.
    fn shell_topbar(&self, cx: &mut Context<Self>) -> Div {
        let settings_problem = self.settings_have_problem();
        let wordmark = h_flex().flex_shrink_0().items_baseline().children([
            div()
                .text_size(TEXT_BODY)
                .font_weight(FontWeight::BOLD)
                .child("course"),
            div()
                .text_size(TEXT_BODY)
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(ACCENT))
                .child("2"),
            div()
                .text_size(TEXT_BODY)
                .font_weight(FontWeight::BOLD)
                .child("md"),
        ]);
        let tabs = h_flex()
            .flex_1()
            .min_w_0()
            .justify_center()
            .gap_6()
            .children(
                [(Page::New, "工作台"), (Page::Library, "我的笔记")]
                    .into_iter()
                    .enumerate()
                    .map(|(index, (page, label))| {
                        // 阅读页归入我的笔记 tab（mock v2 同样归位）。
                        let active =
                            self.page == page || (page == Page::Library && self.page == Page::Result);
                        control(("shell-tab", index))
                            .ghost()
                            .h_auto()
                            .px(px(2.))
                            .py(px(6.))
                            .selected(active)
                            .toggled(active)
                            .accessibility_label(label)
                            .child(
                                v_flex()
                                    .gap(px(4.))
                                    .items_center()
                                    .child(
                                        div()
                                            .text_color(rgb(if active { INK } else { GRAY }))
                                            .when(active, |text| {
                                                text.font_weight(FontWeight::SEMIBOLD)
                                            })
                                            .child(label),
                                    )
                                    .child(div().h(px(2.)).w_full().rounded_full().bg(
                                        if active {
                                            rgb(ACCENT)
                                        } else {
                                            rgba(0x00000000)
                                        },
                                    )),
                            )
                            .on_click(cx.listener(move |this, _, _, cx| {
                                if page == Page::Library {
                                    this.folder_filter = None;
                                }
                                this.navigate(page, cx);
                            }))
                    }),
            );
        let gear = control("shell-settings")
            .ghost()
            .h_auto()
            .min_h(rems(2.286))
            .min_w(rems(2.286))
            .rounded(RADIUS_PILL)
            .icon(IconName::Settings)
            .accessibility_label(if settings_problem {
                "设置，未保存"
            } else {
                "设置"
            })
            .selected(self.page == Page::Settings)
            .toggled(self.page == Page::Settings)
            .when(self.page == Page::Settings, |button| {
                button.bg(rgb(BADGE_PROGRESS_BG))
            })
            .when(settings_problem, |button| button.text_color(rgb(ACCENT_STRONG)))
            .on_click(cx.listener(|this, _, _, cx| this.navigate(Page::Settings, cx)));
        div()
            .w_full()
            .flex_shrink_0()
            .border_b_1()
            .border_color(rgb(HAIRLINE))
            .child(
                h_flex()
                    .w_full()
                    .min_w_0()
                    .max_w(COLUMN)
                    .mx_auto()
                    .px(px(24.))
                    .py(px(12.))
                    .items_center()
                    .child(wordmark)
                    .child(tabs)
                    .child(gear),
            )
    }
    fn page_title(&self) -> String {
        match self.page {
            Page::New => "生成笔记".into(),
            Page::Library => "我的笔记".into(),
            Page::Task => "任务".into(),
            Page::Settings => "设置".into(),
            Page::Result => self
                .preview
                .as_ref()
                .map(|p| p.course.title.clone())
                .unwrap_or_else(|| "笔记".into()),
        }
    }
    fn page_header(&self, window: &mut Window) -> Div {
        let row = h_flex()
            .w_full()
            .min_w_0()
            .gap_3()
            .h_auto()
            .min_h(rems(3.5))
            .flex_wrap()
            .child(
                accessible_text("page-title", self.page_title())
                    .role(Role::Heading)
                    .when(
                        f32::from(window.bounds().size.width) < 1000.
                            || self.preferences.application().font_scale >= 1.5,
                        |title| title.w_full().flex_shrink_0(),
                    )
                    .flex_1()
                    .min_w_0()
                    .whitespace_normal()
                    .text_size(TEXT_TITLE)
                    .font_weight(FontWeight::SEMIBOLD),
            );
        v_flex().gap_2().child(row)
    }
}

impl Render for Desktop {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        theme::apply_scale(self.preferences.application().font_scale, window, cx);
        let content = match self.page {
            Page::New => self.new_page(window, cx),
            Page::Task => self.queue_page(cx),
            Page::Library => self.library_page(window, cx),
            Page::Settings => self.settings_page(window, cx),
            Page::Result => self.reader_page(window, cx),
        };
        let content = v_flex()
            .gap_4()
            .when(self.page == Page::Result, |v| v.h_full().min_h_0())
            .when(self.reading, |v| {
                v.child(
                    accessible_text("opening-note", "正在打开笔记…")
                        .role(Role::Status)
                        .text_color(rgb(MUTED)),
                )
            })
            .child(content);
        let topbar = self.shell_topbar(cx);
        let body = v_flex()
            .flex_1()
            .min_w_0()
            .min_h_0()
            .w_full()
            .when(!matches!(self.page, Page::New | Page::Result), |v| {
                v.child(
                    shell_column()
                        .flex_shrink_0()
                        .child(self.page_header(window)),
                )
            })
            .when(self.page == Page::Library, |v| {
                v.child(shell_column().child(self.library_toolbar(cx)))
            })
            .when_some(self.workspace_error.clone(), |v, message| {
                v.child(
                    shell_column()
                        .pb_3()
                        .text_color(rgb(0xa32626))
                        .child(accessible_text("workspace-error", message).role(Role::Alert))
                        .child(
                            h_flex()
                                .flex_wrap()
                                .gap_2()
                                .mt_2()
                                .child(
                                    control("retry-workspace-records")
                                        .label("重新读取并保存记录")
                                        .disabled(self.job.is_some() || self.storage_ui.busy)
                                        .on_click(cx.listener(|this, _, window, cx| {
                                            this.retry_workspace_records(window, cx)
                                        })),
                                )
                                .when(self.workspace.is_none(), |row| {
                                    row.child(
                                        control("rebuild-workspace-records")
                                            .label("保全原文件并重建记录")
                                            .disabled(self.job.is_some() || self.storage_ui.busy)
                                            .on_click(cx.listener(|this, _, window, cx| {
                                                this.rebuild_workspace_records(window, cx)
                                            })),
                                    )
                                }),
                        ),
                )
            })
            .when_some(self.message.clone(), |v, message| {
                v.child(
                    shell_column().pb_3().child(
                        h_flex()
                            .min_w_0()
                            .gap_3()
                            .p_3()
                            .rounded_md()
                            .bg(rgb(TINT))
                            .child(
                                accessible_text("app-message", message)
                                    .role(Role::Status)
                                    .flex_1()
                                    .min_w_0()
                                    .whitespace_normal(),
                            )
                            .child(
                                control("dismiss-message")
                                    .ghost()
                                    .icon(IconName::Close)
                                    .accessibility_label("关闭提示")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.message = None;
                                        cx.notify();
                                    })),
                            ),
                    ),
                )
            })
            .child(
                div()
                    .id(("page-scroll", self.page as usize))
                    .flex_1()
                    .min_h_0()
                    .min_w_0()
                    .w_full()
                    .when(self.page != Page::Result, |view| {
                        view.overflow_y_scroll()
                            .track_scroll(&self.scrolls[self.page as usize])
                    })
                    .child(
                        shell_column()
                            .when(self.page == Page::Result, |v| v.h_full().min_h_0())
                            .pb_6()
                            .child(content),
                    ),
            );
        let background = v_flex()
            .size_full()
            .bg(rgb(CANVAS))
            .text_color(rgb(INK))
            .text_size(rems(1.))
            .when(!self.system_titlebar, |v| {
                v.child(
                    TitleBar::new().bg(rgb(SIDEBAR)).child(
                        h_flex().w_full().gap_3().child(
                            div()
                                .text_size(px(12.))
                                .text_color(rgb(MUTED))
                                .child("course2md"),
                        ),
                    ),
                )
            })
            .child(topbar)
            .child(body);
        div()
            .id("desktop-action-root")
            .track_focus(&self.root_focus)
            .tab_stop(false)
            .size_full()
            .on_action(cx.listener(|this, _: &NewNote, window, cx| {
                if !window.has_active_dialog(cx) {
                    this.new_note_from_action(window, cx);
                }
            }))
            .on_action(cx.listener(|this, _: &SearchContent, window, cx| {
                if window.has_active_dialog(cx) {
                    return;
                }
                if this.page == Page::Result {
                    this.open_reader_find(window, cx);
                } else {
                    this.navigate(Page::Library, cx);
                    this.inputs[&Field::Search].update(cx, |input, cx| input.focus(window, cx));
                }
            }))
            .on_action(cx.listener(|this, _: &OpenSettings, window, cx| {
                if !window.has_active_dialog(cx) {
                    this.navigate(Page::Settings, cx);
                }
            }))
            .on_action(cx.listener(|this, _: &OpenAbout, window, cx| {
                if !window.has_active_dialog(cx) {
                    this.settings_tab = 3;
                    this.navigate(Page::Settings, cx);
                }
            }))
            .child(a11y::ModalBackground::new(
                background,
                window.has_active_dialog(cx),
            ))
            .children(Root::render_dialog_layer(window, cx))
    }
}
