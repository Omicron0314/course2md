//! Native pages: stable navigation and actions surround independently scrolling content.
use super::*;
use crate::theme::*;
use gpui_component::{button::*, progress::Progress};

fn muted(text: impl Into<SharedString>) -> Div {
    div().text_sm().text_color(rgb(MUTED)).child(text.into())
}
fn section(title: &str, description: &str) -> Div {
    v_flex()
        .gap_1()
        .child(
            div()
                .font_weight(FontWeight::SEMIBOLD)
                .child(title.to_owned()),
        )
        .when(!description.is_empty(), |view| {
            view.child(muted(description.to_owned()))
        })
}
fn card() -> Div {
    v_flex()
        .w_full()
        .min_w_0()
        .p_4()
        .gap_4()
        .bg(rgb(SURFACE))
        .border_1()
        .border_color(rgb(LINE))
        .rounded_md()
}

impl Desktop {
    fn active_work(&self) -> Vec<(&str, &activity::Activity)> {
        let preparing = self
            .progress
            .iter()
            .any(|(id, item)| id.starts_with("model") && !item.done);
        self.progress
            .iter()
            .filter(|(id, item)| {
                !item.done
                    && self.job.is_some()
                    && !(id.as_str() == "transcribe" && preparing && item.current == 0)
            })
            .map(|(id, item)| (id.as_str(), item))
            .collect()
    }
    fn task_summary(&self) -> String {
        if self.cancelling {
            return "正在取消…".into();
        }
        let active = self.active_work();
        match active.len() {
            0 => self.task_status.clone(),
            1 => format!(
                "{} · {}",
                activity::title(active[0].0),
                active[0].1.detail(active[0].0, true)
            ),
            count => format!(
                "{count} 项并行 · {}",
                active
                    .iter()
                    .map(|(id, _)| activity::title(id))
                    .collect::<Vec<_>>()
                    .join(" / ")
            ),
        }
    }
    fn task_notice(&self) -> String {
        if let Some(task) = self
            .active_task
            .as_deref()
            .and_then(|id| self.workspace.as_ref()?.state.task(id))
        {
            return format!("《{}》：{}", task.plan.title, self.task_summary());
        }
        if self.job.is_some() {
            return self.task_summary();
        }
        let mut unread: Vec<_> = self
            .workspace
            .as_ref()
            .into_iter()
            .flat_map(|w| &w.state.tasks)
            .filter(|t| t.unread)
            .collect();
        unread.sort_by_key(|t| std::cmp::Reverse(t.updated));
        unread
            .first()
            .map(|task| {
                let mut text = format!("《{}》：{}", task.plan.title, task.state.label());
                if unread.len() > 1 {
                    text.push_str(&format!("；另有 {} 项更新", unread.len() - 1));
                }
                text
            })
            .unwrap_or_default()
    }
    fn task_page(&mut self, cx: &mut Context<Self>) -> AnyElement {
        if self.progress.is_empty()
            && self.job.is_none()
            && self.logs.is_empty()
            && self.task_status.is_empty()
            && self.task_error.is_none()
        {
            return self.empty_state("暂无任务", "", cx);
        }
        let active = self.active_work();
        let active_ids: Vec<_> = active.iter().map(|(id, _)| *id).collect();
        let mut content = v_flex().gap_4();
        if let Some(error) = &self.task_error {
            content = content.child(
                card()
                    .bg(rgb(0xffefeb))
                    .text_color(rgb(0xa32626))
                    .child(error.clone()),
            );
        }
        if self.job.is_none() && self.task_error.is_none() {
            content = content.child(
                div()
                    .font_weight(FontWeight::MEDIUM)
                    .child(self.task_status.clone()),
            );
        }
        if self.job.is_some() && (active.len() != 1 || self.cancelling) {
            content = content.child(div().font_weight(FontWeight::MEDIUM).child(
                if self.cancelling {
                    "正在取消…".into()
                } else if active.len() > 1 {
                    format!("{} 项并行处理中", active.len())
                } else {
                    self.task_summary()
                },
            ));
        }
        let mut stages: Vec<_> = self.progress.iter().collect();
        stages.sort_by_key(|(id, _)| activity::stage_order(id));
        for (index, (id, item)) in stages.into_iter().enumerate() {
            let running = active_ids.contains(&id.as_str());
            let waiting = self.job.is_some() && !item.done && !running;
            content = content.child(
                v_flex()
                    .gap_2()
                    .py_2()
                    .child(
                        h_flex()
                            .gap_3()
                            .child(div().size(px(8.)).rounded_full().bg(rgb(if item.done {
                                SUCCESS
                            } else if running {
                                BLUE
                            } else {
                                MUTED
                            })))
                            .child(div().flex_1().font_weight(FontWeight::MEDIUM).child(
                                if item.workers > 1 {
                                    format!(
                                        "{} · 最多 {} 路并行",
                                        activity::title(id),
                                        item.workers
                                    )
                                } else {
                                    activity::title(id)
                                },
                            ))
                            .child(muted(if waiting {
                                "等待模型就绪".into()
                            } else {
                                item.detail(id, self.job.is_some())
                            })),
                    )
                    .when(running, |row| {
                        row.child(
                            Progress::new(("activity", index))
                                .accessibility_label(activity::title(id))
                                .loading(item.fraction().is_none())
                                .value(item.fraction().unwrap_or(0.) * 100.)
                                .h(px(4.)),
                        )
                    }),
            );
        }
        if let Some(done) = self.completed.clone() {
            let course = Course::from_completed(&done);
            content = content.child(
                h_flex()
                    .gap_4()
                    .child(div().flex_1().child(done.title))
                    .child(control("read-result").primary().label("阅读笔记").on_click(
                        cx.listener(move |this, _, _, cx| this.open_course(course.clone(), cx)),
                    )),
            );
        }
        if self.logs.is_empty() {
            return content.into_any_element();
        }
        content = content.child(
            h_flex()
                .gap_2()
                .child(
                    control("toggle-logs")
                        .ghost()
                        .icon(if self.show_logs {
                            IconName::ChevronUp
                        } else {
                            IconName::ChevronDown
                        })
                        .label(if self.show_logs {
                            "收起日志"
                        } else {
                            "日志"
                        })
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.show_logs = !this.show_logs;
                            cx.notify();
                        })),
                )
                .when(self.show_logs, |row| {
                    row.child(
                        control("copy-logs")
                            .ghost()
                            .label("复制")
                            .on_click(cx.listener(|this, _, _, cx| {
                                cx.write_to_clipboard(ClipboardItem::new_string(
                                    this.logs.iter().cloned().collect::<Vec<_>>().join("\n"),
                                ))
                            })),
                    )
                }),
        );
        if self.show_logs {
            content = content.child(
                card().child(
                    div()
                        .w_full()
                        .min_w_0()
                        .text_xs()
                        .whitespace_normal()
                        .child(gpui_base::SelectableText::new(
                            "logs-text",
                            self.logs.iter().cloned().collect::<Vec<_>>().join("\n"),
                        )),
                ),
            );
        }
        content.into_any_element()
    }
    fn empty_state(&self, title: &str, description: &str, cx: &mut Context<Self>) -> AnyElement {
        v_flex()
            .items_center()
            .justify_center()
            .py_16()
            .gap_4()
            .child(
                div()
                    .size(px(56.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded_md()
                    .bg(rgb(TINT))
                    .text_color(rgb(BLUE))
                    .child(Icon::new(IconName::BookOpen).size_6()),
            )
            .child(section(title, description).items_center())
            .child(
                control("empty-new")
                    .primary()
                    .label("生成笔记")
                    .on_click(cx.listener(|this, _, window, cx| this.begin_add(window, cx))),
            )
            .into_any_element()
    }
    fn page_title(&self) -> String {
        match self.page {
            Page::New => "生成笔记".into(),
            Page::Library => self
                .folder_filter
                .map(|id| {
                    if id == 0 {
                        "未分类".into()
                    } else {
                        self.library.folders.get(&id).cloned().unwrap_or_default()
                    }
                })
                .unwrap_or_else(|| "课程库".into()),
            Page::Task => "任务".into(),
            Page::Settings => "设置".into(),
            Page::Result => self
                .preview
                .as_ref()
                .map(|p| p.course.title.clone())
                .unwrap_or_else(|| "笔记".into()),
        }
    }
    fn page_header(&self, window: &mut Window, cx: &mut Context<Self>) -> Div {
        let mut row = h_flex()
            .w_full()
            .min_w_0()
            .gap_3()
            .h_auto()
            .min_h(rems(3.5))
            .flex_wrap()
            .when(self.page == Page::Result, |row| {
                row.child(
                    control("back")
                        .ghost()
                        .icon(IconName::ArrowLeft)
                        .accessibility_label("返回")
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.navigate(
                                if this.page == Page::New {
                                    Page::Library
                                } else {
                                    this.result_origin
                                },
                                cx,
                            )
                        })),
                )
            })
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
                    .text_size(rems(18. / 14.))
                    .font_weight(FontWeight::SEMIBOLD),
            );
        if self.page == Page::Library {
            row = row.child(
                control("add-course")
                    .primary()
                    .icon(IconName::Plus)
                    .label("生成笔记")
                    .on_click(
                        cx.listener(|this, _, window, cx| this.new_draft(true, false, window, cx)),
                    ),
            );
        } else if self.page == Page::Result {
            row = row.child(self.reader_toolbar(cx));
        }
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
        let settings_problem = self.settings_have_problem();
        let sidebar = v_flex()
            .w(px(208.))
            .h_full()
            .flex_shrink_0()
            .p_3()
            .gap_4()
            .bg(rgb(SIDEBAR))
            .border_r_1()
            .border_color(rgb(LINE))
            .child(
                v_flex().gap_1().children(
                    [
                        (Page::Library, "课程库", Icon::new(IconName::BookOpen)),
                        (Page::New, "生成笔记", Icon::new(IconName::Plus)),
                        (Page::Task, "任务", icons::task()),
                    ]
                    .into_iter()
                    .enumerate()
                    .map(|(index, (page, label, icon))| {
                        navigation(
                            control(("nav", index)),
                            self.page == page
                                && (page != Page::Library || self.folder_filter.is_none()),
                        )
                        .w_full()
                        .h(px(36.))
                        .justify_start()
                        .accessibility_label(label)
                        .child(
                            h_flex()
                                .w_full()
                                .gap_2()
                                .child(icon.size(px(20.)))
                                .child(label),
                        )
                        .selected(
                            self.page == page
                                && (page != Page::Library || self.folder_filter.is_none()),
                        )
                        .on_click(cx.listener(
                            move |this, _, _window, cx| {
                                if page == Page::New {
                                    this.navigate(Page::New, cx);
                                } else {
                                    if page == Page::Library {
                                        this.folder_filter = None;
                                    }
                                    this.navigate(page, cx);
                                }
                            },
                        ))
                    }),
                ),
            )
            .child(
                div()
                    .id("folder-scroll")
                    .min_h_0()
                    .flex_1()
                    .overflow_y_scroll()
                    .child(self.folder_sidebar(cx)),
            )
            .child(
                navigation(control("nav-settings"), self.page == Page::Settings)
                    .w_full()
                    .min_h(rems(2.6))
                    .justify_start()
                    .accessibility_label(if settings_problem {
                        "设置，未保存"
                    } else {
                        "设置"
                    })
                    .child(
                        h_flex()
                            .min_w_0()
                            .w_full()
                            .gap_2()
                            .child(Icon::new(IconName::Settings).size(px(20.)))
                            .child(
                                v_flex()
                                    .min_w_0()
                                    .gap_1()
                                    .child("设置")
                                    .when(settings_problem, |label| {
                                        label.child(div().text_xs().child("未保存"))
                                    }),
                            ),
                    )
                    .selected(self.page == Page::Settings)
                    .on_click(cx.listener(|this, _, _, cx| this.navigate(Page::Settings, cx))),
            );
        let body = v_flex()
            .flex_1()
            .min_w_0()
            .h_full()
            .child(
                div()
                    .px(px(24.))
                    .flex_shrink_0()
                    .child(self.page_header(window, cx)),
            )
            .when(self.page == Page::Library, |v| {
                v.child(div().px(px(24.)).child(self.library_toolbar(cx)))
            })
            .when_some(self.workspace_error.clone(), |v, message| {
                v.child(
                    div()
                        .px(px(24.))
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
                    div().px(px(24.)).pb_3().child(
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
                    .when(self.page != Page::Result, |view| {
                        view.overflow_y_scroll()
                            .track_scroll(&self.scrolls[self.page as usize])
                    })
                    .px(px(24.))
                    .pb_6()
                    .child(content),
            )
            .when(self.page == Page::New, |view| {
                view.child(div().px_6().pb_3().child(self.import_footer(cx)))
            });
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
            .child(
                h_flex()
                    .flex_1()
                    .min_h_0()
                    .w_full()
                    .child(sidebar)
                    .child(body),
            )
            .when(
                (self.job.is_some()
                    || self
                        .workspace
                        .as_ref()
                        .is_some_and(|w| w.state.tasks.iter().any(|task| task.unread)))
                    && self.page != Page::Task,
                |v| {
                    v.child(
                        h_flex()
                            .h_auto()
                            .min_h(rems(2.6))
                            .flex_shrink_0()
                            .px_4()
                            .gap_3()
                            .bg(rgb(SURFACE))
                            .border_t_1()
                            .border_color(rgb(LINE))
                            .child(
                                accessible_text("task-notification", self.task_notice())
                                    .role(Role::Status)
                                    .flex_1()
                                    .min_w_0()
                                    .whitespace_normal(),
                            )
                            .when(self.page != Page::Task, |v| {
                                v.child(
                                    control("status-task")
                                        .label(
                                            if self.kind == Kind::Models
                                                && self.active_task.is_none()
                                                && self.job.is_some()
                                            {
                                                "查看模型准备"
                                            } else {
                                                "查看任务"
                                            },
                                        )
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            if this.kind == Kind::Models
                                                && this.active_task.is_none()
                                                && this.job.is_some()
                                            {
                                                this.settings_tab = 3;
                                                this.navigate(Page::Settings, cx);
                                            } else {
                                                this.navigate(Page::Task, cx);
                                            }
                                        })),
                                )
                            }),
                    )
                },
            );
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
