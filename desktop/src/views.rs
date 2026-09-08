//! Native pages: stable navigation and actions surround independently scrolling content.
use super::*;
use crate::theme::*;
use gpui_component::button::*;

/// Every chrome row and page body sits in this one centered column.
fn shell_column() -> Div {
    shell_column_at(COLUMN)
}

fn shell_column_for(page: Page) -> Div {
    if page == Page::Settings {
        shell_column_settings()
    } else {
        shell_column()
    }
}

/// Settings groups run on the narrower column from the mock (COLUMN_SETTINGS).
fn shell_column_settings() -> Div {
    shell_column_at(COLUMN_SETTINGS)
}

fn shell_column_at(width: Rems) -> Div {
    div().w_full().min_w_0().max_w(width).mx_auto().px(px(24.))
}

impl Desktop {
    /// Keep task results reachable while the user is on another page.
    fn shell_topbar(&self, cx: &mut Context<Self>) -> Div {
        let settings_problem = self.settings_have_problem();
        let task_count = self.workspace.as_ref().map_or(0, |workspace| {
            workspace
                .state
                .tasks
                .iter()
                .filter(|task| {
                    task.handled_by.is_none()
                        && (task.unread
                            || matches!(
                                task.state,
                                workspace::TaskState::NeedsAttention
                                    | workspace::TaskState::Uncertain
                                    | workspace::TaskState::Partial
                            ))
                })
                .count()
        });
        let task_label = if task_count == 0 {
            "任务".to_owned()
        } else {
            format!("任务（{task_count}）")
        };
        let wordmark = h_flex().flex_shrink_0().items_baseline().children([
            div()
                .text_size(TEXT_BODY)
                .font_weight(FontWeight::BOLD)
                .child("course"),
            div()
                .text_size(TEXT_BODY)
                .font_weight(FontWeight::BOLD)
                .text_color(color(ACCENT))
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
                [
                    (Page::New, "工作台"),
                    (Page::Library, "我的笔记"),
                    (Page::Task, task_label.as_str()),
                ]
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
                        .accessibility_label(label.to_owned())
                        .child(
                            v_flex()
                                .gap(px(4.))
                                .items_center()
                                .child(
                                    div()
                                        .text_color(color(if active { INK } else { GRAY }))
                                        .when(active, |text| text.font_weight(FontWeight::SEMIBOLD))
                                        .child(label.to_owned()),
                                )
                                .child(div().h(px(2.)).w_full().rounded_full().bg(if active {
                                    color(ACCENT)
                                } else {
                                    rgba(0x00000000)
                                })),
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
                button.bg(color(BADGE_PROGRESS_BG))
            })
            .when(settings_problem, |button| {
                button.text_color(color(ACCENT_STRONG))
            })
            .on_click(cx.listener(|this, _, window, cx| this.open_settings(window, cx)));
        div()
            .w_full()
            .flex_shrink_0()
            .border_b_1()
            .border_color(color(HAIRLINE))
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

    fn task_result_notice(&self, cx: &mut Context<Self>) -> Option<Div> {
        let task = self
            .workspace
            .as_ref()?
            .state
            .tasks
            .iter()
            .filter(|task| task.unread && task.handled_by.is_none())
            .max_by_key(|task| task.updated)?;
        let id = task.id.clone();
        let dismiss_id = id.clone();
        let action = match task.state {
            workspace::TaskState::Complete => "查看生成结果",
            workspace::TaskState::Partial => "查看未完成部分",
            workspace::TaskState::Uncertain => "确认请求结果",
            _ => "查看任务",
        };
        Some(
            shell_column_for(self.page).py_2().flex_shrink_0().child(
                h_flex()
                    .w_full()
                    .min_w_0()
                    .items_center()
                    .gap_2()
                    .p_3()
                    .rounded_md()
                    .bg(color(TINT))
                    .child(
                        accessible_text(
                            "background-task-result",
                            format!("《{}》：{}", task.plan.title, task.state.label(),),
                        )
                        .flex_1()
                        .min_w_0()
                        .line_clamp(2)
                        .text_ellipsis(),
                    )
                    .child(
                        control("open-task-result")
                            .label(action)
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.navigate(Page::Task, cx);
                                if this.page == Page::Task {
                                    this.select_task(&id, cx);
                                }
                            })),
                    )
                    .child(
                        control("dismiss-task-result")
                            .ghost()
                            .icon(IconName::Close)
                            .accessibility_label("关闭这条任务提示")
                            .on_click(cx.listener(move |this, _, _, cx| {
                                if let Some(workspace) = &mut this.workspace {
                                    if let Err(error) = workspace.transaction(|state| {
                                        if let Some(task) = state.task_mut(&dismiss_id) {
                                            task.unread = false;
                                        }
                                        Ok(())
                                    }) {
                                        this.workspace_error =
                                            Some(format!("任务提示状态尚未保存：{error:#}"));
                                    }
                                }
                                cx.notify();
                            })),
                    ),
            ),
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
        theme::apply_preference(&self.preferences.application().appearance, window, cx);
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
                        .text_color(color(MUTED)),
                )
            })
            .child(content);
        let topbar = self.shell_topbar(cx);
        let task_notice = self.task_result_notice(cx);
        let body = v_flex()
            .flex_1()
            .min_w_0()
            .min_h_0()
            .w_full()
            .when_some(task_notice, |body, notice| body.child(notice))
            .when(
                !matches!(self.page, Page::New | Page::Result | Page::Settings),
                |v| {
                    v.child(
                        shell_column_for(self.page)
                            .flex_shrink_0()
                            .child(self.page_header(window)),
                    )
                },
            )
            .when(self.page == Page::Library, |v| {
                v.child(shell_column_for(self.page).child(self.library_toolbar(cx)))
            })
            .when_some(self.workspace_error.clone(), |v, message| {
                v.child(
                    shell_column_for(self.page)
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
                    shell_column_for(self.page).pb_3().child(
                        h_flex()
                            .min_w_0()
                            .gap_3()
                            .p_3()
                            .rounded_md()
                            .bg(color(TINT))
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
                        shell_column_for(self.page)
                            .when(self.page == Page::Result, |v| v.h_full().min_h_0())
                            .pb_6()
                            .child(content),
                    ),
            );
        let background = v_flex()
            .size_full()
            .bg(color(CANVAS))
            .text_color(color(INK))
            .text_size(rems(1.))
            .when(!self.system_titlebar, |v| {
                v.child(
                    TitleBar::new().bg(color(SIDEBAR)).child(
                        h_flex().w_full().gap_3().child(
                            div()
                                .text_size(px(12.))
                                .text_color(color(MUTED))
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
                    this.open_settings(window, cx);
                }
            }))
            .on_action(cx.listener(|this, _: &OpenAbout, window, cx| {
                if !window.has_active_dialog(cx) {
                    this.settings_tab = 3;
                    this.open_settings(window, cx);
                }
            }))
            .child(a11y::ModalBackground::new(
                background,
                window.has_active_dialog(cx),
            ))
            .children(Root::render_dialog_layer(window, cx))
    }
}
