//! Native pages: stable navigation and actions surround independently scrolling content.
use super::*;
use crate::theme::*;
use gpui_component::button::*;

const SHELL_GUTTER: f32 = 24.;
const WIDE_COLUMN: Rems = rems(82.);
pub(super) const SETTINGS_SIDEBAR_WIDTH: f32 = 200.;
pub(super) const SETTINGS_COLUMN_GAP: f32 = 32.;
const SETTINGS_CONTENT_MAX_WIDTH: f32 = 920.;
const SETTINGS_SHELL_WIDTH: f32 =
    SETTINGS_SIDEBAR_WIDTH + SETTINGS_COLUMN_GAP + SETTINGS_CONTENT_MAX_WIDTH + SHELL_GUTTER * 2.;
const SETTINGS_SIDEBAR_BREAKPOINT: f32 = 1100.;

fn shell_column_for(page: Page) -> Div {
    shell_column_at(shell_column_width(page))
}

fn shell_column_width(page: Page) -> AbsoluteLength {
    match page {
        Page::Settings => px(SETTINGS_SHELL_WIDTH).into(),
        Page::Library | Page::Result => WIDE_COLUMN.into(),
        _ => COLUMN.into(),
    }
}

/// The max width includes both gutters; layout calculations use the same box.
pub(super) fn shell_content_width(page: Page, window: &Window) -> f32 {
    (f32::from(window.bounds().size.width).min(f32::from(
        shell_column_width(page).to_pixels(window.rem_size()),
    )) - SHELL_GUTTER * 2.)
        .max(0.)
}

pub(super) fn settings_uses_sidebar(window: &Window) -> bool {
    f32::from(window.bounds().size.width) >= SETTINGS_SIDEBAR_BREAKPOINT
}

/// Shared by the settings panel and its grids: exclude the real navigation and gutters.
pub(super) fn settings_content_width(window: &Window) -> f32 {
    let available_width = shell_content_width(Page::Settings, window);
    if settings_uses_sidebar(window) {
        (available_width - SETTINGS_SIDEBAR_WIDTH - SETTINGS_COLUMN_GAP)
            .clamp(0., SETTINGS_CONTENT_MAX_WIDTH)
    } else {
        available_width
    }
}

fn shell_column_at(width: AbsoluteLength) -> Div {
    div()
        .w_full()
        .min_w_0()
        .max_w(width)
        .mx_auto()
        .px(px(SHELL_GUTTER))
}

impl Desktop {
    /// Keep task results reachable while the user is on another page.
    fn shell_topbar(&self, window: &mut Window, cx: &mut Context<Self>) -> TitleBar {
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
        let index = match self.page {
            Page::Library | Page::Result => 1.,
            Page::Task => 2.,
            _ => 0.,
        };
        let position = crate::motion::value("navigation-indicator", index, window, cx);
        let task_label = if task_count == 0 {
            "任务".to_owned()
        } else {
            format!("任务 · {task_count}")
        };
        // Match the traffic-light reserve with an equal right-hand region.
        // The navigation is centered in the full window, not the padded TitleBar.
        let scale = self.preferences.application().font_scale;
        let side_width = (72. * scale + 16.).max(96.);
        let nav_width =
            (360. * scale).min((f32::from(window.bounds().size.width) - side_width * 2.).max(0.));
        let tab_width = nav_width / 3.;
        let nav = div()
            .relative()
            .w(px(nav_width))
            .h(px(36.))
            .flex_shrink_0()
            .when(self.page != Page::Settings, |view| {
                view.child(
                    div()
                        .absolute()
                        .left(px(tab_width * position))
                        .top(px(0.))
                        .w(px(tab_width))
                        .h(px(36.))
                        .rounded(RADIUS_SMALL)
                        .bg(color(ACCENT)),
                )
            })
            .child(
                h_flex().w_full().h_full().children(
                    [
                        (Page::New, "工作台".to_owned(), icons::dashboard()),
                        (Page::Library, "我的笔记".to_owned(), icons::book_open()),
                        (Page::Task, task_label, icons::task()),
                    ]
                    .into_iter()
                    .enumerate()
                    .map(|(index, (page, label, icon))| {
                        let active = self.page == page
                            || (page == Page::Library && self.page == Page::Result);
                        let coverage = if self.page == Page::Settings {
                            0.
                        } else {
                            (1. - (position - index as f32).abs()).clamp(0., 1.)
                        };
                        control(("shell-tab", index))
                            .ghost()
                            .w(px(tab_width))
                            .h(px(36.))
                            .min_h(px(36.))
                            .px(px(12.))
                            .rounded(RADIUS_SMALL)
                            .bg(gpui::transparent_black())
                            .text_color(theme::blend(color(GRAY), color(ON_PRIMARY), coverage))
                            .selected(active)
                            .toggled(active)
                            .icon(icon.size(px(18.)))
                            .label(label)
                            .when(active, |button| button.font_weight(FontWeight::SEMIBOLD))
                            .on_click(cx.listener(move |this, _, _, cx| {
                                if page == Page::Library {
                                    this.folder_filter = None;
                                }
                                this.navigate(page, cx);
                            }))
                    }),
                ),
            );
        let settings_problem = self.settings_have_problem();
        let settings_selected = self.page == Page::Settings;
        let settings_amount = crate::motion::value(
            "settings-navigation-selected",
            if settings_selected { 1. } else { 0. },
            window,
            cx,
        );
        let gear = quiet("shell-settings")
            .icon(if settings_problem {
                icons::warning().text_color(color(if settings_selected {
                    ON_PRIMARY
                } else {
                    WARNING
                }))
            } else {
                icons::settings()
            })
            .label("设置")
            .h(px(36.))
            .min_h(px(36.))
            .selected(settings_selected)
            .toggled(settings_selected)
            .bg(theme::blend(color(CANVAS), color(ACCENT), settings_amount))
            .text_color(theme::blend(
                color(GRAY),
                color(ON_PRIMARY),
                settings_amount,
            ))
            .when(settings_selected, |b| b.font_weight(FontWeight::SEMIBOLD))
            .accessibility_label(if settings_problem {
                "设置，未保存"
            } else {
                "设置"
            })
            .on_click(cx.listener(|this, _, window, cx| this.open_settings(window, cx)));
        TitleBar::new()
            .h(px(52.))
            .pl_0()
            .bg(color(CANVAS))
            .border_color(color(HAIRLINE))
            .child(
                h_flex()
                    .w_full()
                    .min_w_0()
                    .h_full()
                    .child(div().w(px(side_width)).flex_shrink_0())
                    .child(h_flex().flex_1().min_w_0().justify_center().child(nav))
                    .child(
                        h_flex()
                            .w(px(side_width))
                            .flex_shrink_0()
                            .pr(px(16.))
                            .justify_end()
                            .child(gear),
                    ),
            )
    }

    fn task_result_notice(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
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
        Some(crate::motion::enter(
            SharedString::from(format!("task-result-notice-{}", task.id)),
            shell_column_for(self.page).py_2().flex_shrink_0().child(
                h_flex()
                    .w_full()
                    .min_w_0()
                    .items_center()
                    .gap_2()
                    .p_3()
                    .rounded_md()
                    .bg(color(TINT))
                    .child(icons::info().text_color(color(ACCENT)))
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
                            .icon(icons::arrow_forward())
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
            cx,
        ))
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
            Page::Task => self.queue_page(window, cx),
            Page::Library => self.library_page(window, cx),
            Page::Settings => self.settings_page(window, cx),
            Page::Result => self.reader_page(window, cx),
        };
        let content = v_flex()
            .gap_4()
            .when(matches!(self.page, Page::Result | Page::Settings), |v| {
                v.h_full().min_h_0()
            })
            .when(self.reading, |v| {
                v.child(
                    h_flex()
                        .items_center()
                        .gap(px(8.))
                        .child(crate::motion::spinner("opening-note-spinner", cx))
                        .child(
                            accessible_text("opening-note", "正在打开笔记…")
                                .role(Role::Status)
                                .text_color(color(MUTED)),
                        ),
                )
            })
            .child(content);
        let content = crate::motion::enter(("page-enter", self.page as usize), content, cx);
        let topbar = self.shell_topbar(window, cx);
        let task_notice = self.task_result_notice(cx);
        let body = v_flex()
            .flex_1()
            .min_w_0()
            .min_h_0()
            .w_full()
            .when_some(task_notice, |body, notice| body.child(notice))
            .when(self.page == Page::Library, |v| {
                v.child(
                    shell_column_for(self.page)
                        .flex_shrink_0()
                        .child(self.page_header(window)),
                )
            })
            .when(self.page == Page::Library, |v| {
                v.child(shell_column_for(self.page).child(self.library_toolbar(cx)))
            })
            .when_some(self.workspace_error.clone(), |v, message| {
                v.child(crate::motion::enter(
                    SharedString::from(format!("workspace-error-{message}")),
                    shell_column_for(self.page)
                        .pb_3()
                        .text_color(color(DANGER))
                        .child(accessible_text("workspace-error", message).role(Role::Alert))
                        .child(
                            h_flex()
                                .flex_wrap()
                                .gap_2()
                                .mt_2()
                                .child(
                                    control("retry-workspace-records")
                                        .icon(icons::refresh())
                                        .label("重新读取并保存记录")
                                        .disabled(self.job.is_some() || self.storage_ui.busy)
                                        .on_click(cx.listener(|this, _, window, cx| {
                                            this.retry_workspace_records(window, cx)
                                        })),
                                )
                                .when(self.workspace.is_none(), |row| {
                                    row.child(
                                        control("rebuild-workspace-records")
                                            .icon(icons::restart())
                                            .label("保全原文件并重建记录")
                                            .disabled(self.job.is_some() || self.storage_ui.busy)
                                            .on_click(cx.listener(|this, _, window, cx| {
                                                this.rebuild_workspace_records(window, cx)
                                            })),
                                    )
                                }),
                        ),
                    cx,
                ))
            })
            .when_some(self.message.clone(), |v, message| {
                v.child(crate::motion::enter(
                    SharedString::from(format!("app-message-{message}")),
                    shell_column_for(self.page).pb_3().child(
                        h_flex()
                            .min_w_0()
                            .gap_3()
                            .p_3()
                            .rounded_md()
                            .bg(color(TINT))
                            .child(icons::info().text_color(color(ACCENT)))
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
                    cx,
                ))
            })
            .child(
                div()
                    .id(("page-scroll", self.page as usize))
                    .flex_1()
                    .min_h_0()
                    .min_w_0()
                    .w_full()
                    .when(
                        !matches!(self.page, Page::Result | Page::Settings),
                        |view| {
                            view.overflow_y_scroll()
                                .track_scroll(&self.scrolls[self.page as usize])
                        },
                    )
                    .child(
                        shell_column_for(self.page)
                            .when(matches!(self.page, Page::Result | Page::Settings), |v| {
                                v.h_full().min_h_0()
                            })
                            .when(self.page != Page::Settings, |v| v.pb_6())
                            .child(content),
                    ),
            );
        let background = v_flex()
            .size_full()
            .bg(color(CANVAS))
            .text_color(color(INK))
            .text_size(rems(1.))
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
