//! A continuous source → content → destination → generation form.
use super::*;
use crate::{preferences::ServicePurpose, theme::*};
use course2md::subtitle::{SubtitleEvidence, SubtitleReadError, SubtitleTrack};
use gpui_component::{
    button::*,
    checkbox::Checkbox,
    menu::{DropdownMenu, PopupMenuItem},
    switch::Switch,
};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

fn help(text: impl Into<SharedString>) -> Div {
    let text = text.into();
    div()
        .text_sm()
        .text_color(rgb(MUTED))
        .child(accessible_text(text_id("help", &text), text))
}
fn issue(message: impl Into<SharedString>) -> Div {
    let message = message.into();
    div()
        .text_sm()
        .text_color(rgb(0xa32626))
        .whitespace_normal()
        .child(accessible_text(text_id("issue", &message), message))
}
fn text_id(kind: &str, value: &str) -> SharedString {
    use std::hash::{Hash, Hasher};
    let mut hash = std::collections::hash_map::DefaultHasher::new();
    value.hash(&mut hash);
    format!("import-{kind}-{:x}", hash.finish()).into()
}
/// Decorative platform capability marker (brand dot + label); not interactive.
fn platform_badge(name: &'static str, color: u32) -> Div {
    h_flex()
        .gap(px(5.))
        .items_center()
        .flex_shrink_0()
        .child(div().size(px(8.)).rounded_full().bg(rgb(color)))
        .child(
            div()
                .text_size(TEXT_AUX)
                .text_color(rgb(GRAY))
                .child(name),
        )
}
/// One labeled region inside the workbench box, separated by the card hairline.
fn box_section(label: &'static str) -> Div {
    v_flex()
        .w_full()
        .min_w_0()
        .gap_2()
        .pt_4()
        .border_t_1()
        .border_color(rgb(CARD_LINE))
        .child(
            accessible_text(text_id("box-section", label), label)
                .text_size(TEXT_AUX)
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgb(GRAY)),
        )
}
/// One fact row inside the plan confirmation inset.
fn plan_row(value: impl Into<SharedString>, warning: bool) -> Stateful<Div> {
    let value = value.into();
    accessible_text(text_id("plan-row", &value), value)
        .w_full()
        .min_w_0()
        .whitespace_normal()
        .text_size(TEXT_BODY)
        .text_color(rgb(if warning { WARNING } else { INK }))
        .when(warning, |row| row.font_weight(FontWeight::SEMIBOLD))
}

fn uses_speech(source: &source::Source, source_mode: usize) -> bool {
    source_mode == 2
        || (source_mode == 0
            && source.selected_subtitle.is_none()
            && source.subtitle_request.is_none()
            && source.subtitle_read_error.is_none()
            && matches!(
                source.subtitles,
                SubtitleEvidence::NoneFound | SubtitleEvidence::Unsupported { .. }
            ))
}

struct PlanDialog {
    desktop: Entity<Desktop>,
    _observation: Subscription,
}

impl Render for PlanDialog {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let lines = self.desktop.read(cx).generation_plan();
        let available =
            (window.bounds().size.height - window.rem_size() * 8. - px(64.)).max(px(120.));
        v_flex()
            .id("import-full-plan-dialog")
            .role(Role::Dialog)
            .aria_label("本次笔记的完整计划")
            .min_w_0()
            .gap_4()
            .child(
                v_flex()
                    .id("import-full-plan-content")
                    .max_h(available)
                    .overflow_y_scroll()
                    .min_w_0()
                    .gap_3()
                    .children(lines.into_iter().enumerate().map(|(index, line)| {
                        accessible_text(("import-full-plan-line", index), line).whitespace_normal()
                    })),
            )
            .child(
                control("close-import-full-plan")
                    .label("返回生成页")
                    .on_click(|_, window, cx| window.close_dialog(cx)),
            )
    }
}

impl Desktop {
    fn import_base_config(&self) -> course2md::settings::ConfigFile {
        self.workspace
            .as_ref()
            .and_then(|workspace| workspace.state.draft())
            .and_then(|draft| draft.base_config.clone())
            .unwrap_or_else(|| self.preferences.defaults_config())
    }

    pub fn confirm_subtitle(
        &mut self,
        track: SubtitleTrack,
        explicit: bool,
        cx: &mut Context<Self>,
    ) {
        let Some(source) = self.source_preview.clone() else {
            return;
        };
        if let Some(cancel) = self.subtitle_cancel.take() {
            cancel.store(true, Ordering::Relaxed);
        }
        self.subtitle_generation = self.subtitle_generation.wrapping_add(1);
        let generation = self.subtitle_generation;
        let source_generation = self.preview_generation;
        let token = self
            .workspace
            .as_ref()
            .and_then(|workspace| workspace.state.draft())
            .map(|draft| (draft.id.clone(), draft.revision));
        let source_id = source.identity.clone();
        let cancel = Arc::new(AtomicBool::new(false));
        self.subtitle_cancel = Some(cancel.clone());
        self.subtitle_loading = true;
        self.subtitle_error = None;
        if explicit {
            // The requested text source takes effect before I/O. A failed
            // caption read must never silently keep the previous ASR choice.
            self.task_options.source_mode = 1;
        }
        if let Some(source) = &mut self.source_preview {
            source.subtitle_request = Some(track.clone());
            source.subtitle_read_error = None;
        }
        if !self.save_current_draft(cx) {
            self.subtitle_loading = false;
            self.subtitle_cancel = None;
            return;
        }
        self.preview_workers += 1;
        let task = cx
            .background_executor()
            .spawn(async move { source::read_subtitle(&source, &track, cancel) });
        cx.spawn(async move |this, cx| {
            let result = task.await;
            let _ = this.update(cx, |this, cx| {
                this.preview_workers = this.preview_workers.saturating_sub(1);
                if this.subtitle_generation != generation
                    || this.preview_generation != source_generation
                {
                    return;
                }
                if !this
                    .source_preview
                    .as_ref()
                    .is_some_and(|source| source.identity == source_id)
                {
                    return;
                }
                if let Some((id, revision)) = &token {
                    if !this
                        .workspace
                        .as_ref()
                        .and_then(|workspace| workspace.state.draft())
                        .is_some_and(|draft| &draft.id == id && &draft.revision == revision)
                    {
                        return;
                    }
                }
                this.subtitle_loading = false;
                this.subtitle_cancel = None;
                match result {
                    Ok(subtitle) => {
                        if let Some(source) = &mut this.source_preview {
                            source.selected_subtitle = Some(subtitle);
                            source.subtitle_request = None;
                            source.subtitle_read_error = None;
                        }
                        if explicit {
                            this.task_options.source_mode = 1;
                        }
                        this.save_current_draft(cx);
                    }
                    Err(SubtitleReadError::Cancelled) => {}
                    Err(error) => {
                        this.subtitle_error = Some(error.to_string());
                        if let Some(source) = &mut this.source_preview {
                            source.subtitle_read_error = Some(error);
                        }
                        this.save_current_draft(cx);
                    }
                }
                cx.notify();
            });
        })
        .detach();
        cx.notify();
    }

    fn choose_attached_subtitle(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(source) = self.source_preview.clone() else {
            return;
        };
        let source_generation = self.preview_generation;
        let prompt = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: Some("选择 SRT 或 VTT 字幕".into()),
        });
        cx.spawn_in(window, async move |this, cx| {
            let result = prompt.await;
            let _ = this.update_in(cx, |this, _, cx| {
                if this.preview_generation != source_generation
                    || !this
                        .source_preview
                        .as_ref()
                        .is_some_and(|current| current.identity == source.identity)
                {
                    return;
                }
                match result {
                    Ok(Ok(Some(paths))) => {
                        if let Some(path) = paths.into_iter().next() {
                            if !course2md::subtitle::is_subtitle_file(&path) {
                                this.subtitle_error = Some("请选择 SRT 或 VTT 字幕文件".into());
                            } else {
                                let track = course2md::subtitle::file_track(path, None);
                                if let Some(source) = &mut this.source_preview {
                                    match &mut source.subtitles {
                                        SubtitleEvidence::Found { tracks, .. } => {
                                            if !tracks
                                                .iter()
                                                .any(|candidate| candidate.id == track.id)
                                            {
                                                tracks.push(track.clone());
                                            }
                                        }
                                        _ => {
                                            source.subtitles = SubtitleEvidence::Found {
                                                tracks: vec![track.clone()],
                                                warning: None,
                                            }
                                        }
                                    }
                                }
                                this.confirm_subtitle(track, true, cx);
                            }
                        }
                    }
                    Ok(Ok(None)) => {}
                    Ok(Err(error)) => {
                        this.subtitle_error = Some(format!("无法打开字幕选择器：{error:#}"))
                    }
                    Err(error) => this.subtitle_error = Some(error.to_string()),
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn retry_subtitles(&mut self, cx: &mut Context<Self>) {
        let Some(source) = self.source_preview.clone() else {
            return;
        };
        if let Some(cancel) = self.subtitle_cancel.take() {
            cancel.store(true, Ordering::Relaxed);
        }
        self.subtitle_generation = self.subtitle_generation.wrapping_add(1);
        let generation = self.subtitle_generation;
        let source_generation = self.preview_generation;
        let identity = source.identity.clone();
        let cancel = Arc::new(AtomicBool::new(false));
        self.subtitle_cancel = Some(cancel.clone());
        self.subtitle_loading = true;
        self.subtitle_error = None;
        if let Some(source) = &mut self.source_preview {
            source.subtitle_read_error = Some(SubtitleReadError::Failed {
                message: "字幕重新读取尚未完成。可以重试，或继续使用已确认的正文。".into(),
            });
        }
        if !self.save_current_draft(cx) {
            self.subtitle_loading = false;
            self.subtitle_cancel = None;
            return;
        }
        let token = self
            .workspace
            .as_ref()
            .and_then(|workspace| workspace.state.draft())
            .map(|draft| (draft.id.clone(), draft.revision));
        self.preview_workers += 1;
        let task = cx
            .background_executor()
            .spawn(async move { source::refresh_subtitles(&source, cancel) });
        cx.spawn(async move |this, cx| {
            let result = task.await;
            let _ = this.update(cx, |this, cx| {
                this.preview_workers = this.preview_workers.saturating_sub(1);
                if this.subtitle_generation != generation
                    || this.preview_generation != source_generation
                    || token.as_ref().is_some_and(|(id, revision)| !this
                        .workspace.as_ref().and_then(|workspace| workspace.state.draft())
                        .is_some_and(|draft| &draft.id == id && &draft.revision == revision))
                    || !this
                        .source_preview
                        .as_ref()
                        .is_some_and(|source| source.identity == identity)
                {
                    return;
                }
                this.subtitle_loading = false;
                this.subtitle_cancel = None;
                match result {
                    Ok(mut evidence) => {
                        if let SubtitleEvidence::Found { tracks, .. } = &mut evidence {
                            let original = this
                                .source_preview
                                .as_ref()
                                .and_then(|source| source.original_language.as_deref());
                            course2md::subtitle::sort_tracks(
                                tracks,
                                &this.preferences.generation().preferred_subtitle_languages,
                                "zh-Hans",
                                original,
                            );
                        }
                        let old_id = this
                            .source_preview
                            .as_ref()
                            .and_then(|source| source.selected_subtitle.as_ref())
                            .map(|subtitle| subtitle.track_id.clone());
                        let track = match &old_id {
                            Some(id) => evidence.tracks().iter().find(|track| &track.id == id),
                            None => evidence.tracks().first(),
                        }.cloned();
                        if old_id.is_some() && track.is_none() {
                            this.subtitle_error = Some("原来选择的字幕已无法重新读取。已确认的正文仍保留；可以继续使用它，或明确选择其他文字来源。".into());
                        }
                        if let Some(source) = &mut this.source_preview {
                            source.subtitles = evidence;
                            source.subtitle_read_error = this.subtitle_error.clone().map(|message| SubtitleReadError::Failed { message });
                        }
                        this.save_current_draft(cx);
                        if let Some(track) = track {
                            this.confirm_subtitle(track, false, cx);
                        }
                    }
                    Err(error) => {
                        this.subtitle_error =
                            Some(format!("字幕未读取成功，尚不能确认是否可用：{error:#}"));
                        if let Some(source) = &mut this.source_preview {
                            source.subtitle_read_error = this.subtitle_error.clone().map(|message| SubtitleReadError::Failed { message });
                        }
                        this.save_current_draft(cx);
                    }
                }
                cx.notify();
            });
        })
        .detach();
        cx.notify();
    }

    fn use_speech(&mut self, cx: &mut Context<Self>) {
        if let Some(cancel) = self.subtitle_cancel.take() {
            cancel.store(true, Ordering::Relaxed);
        }
        self.subtitle_generation = self.subtitle_generation.wrapping_add(1);
        self.subtitle_loading = false;
        self.subtitle_error = None;
        self.task_options.source_mode = 2;
        if let Some(source) = &mut self.source_preview {
            source.subtitle_request = None;
            source.subtitle_read_error = None;
        }
        self.save_current_draft(cx);
        cx.notify();
    }

    fn use_confirmed_subtitle(&mut self, cx: &mut Context<Self>) {
        if !self
            .source_preview
            .as_ref()
            .is_some_and(|source| source.selected_subtitle.is_some())
        {
            return;
        }
        if let Some(cancel) = self.subtitle_cancel.take() {
            cancel.store(true, Ordering::Relaxed);
        }
        self.subtitle_generation = self.subtitle_generation.wrapping_add(1);
        self.subtitle_loading = false;
        self.subtitle_error = None;
        self.task_options.source_mode = 1;
        if let Some(source) = &mut self.source_preview {
            source.subtitle_request = None;
            source.subtitle_read_error = None;
        }
        self.save_current_draft(cx);
        cx.notify();
    }

    fn select_source_input(&mut self, input: String, window: &mut Window, cx: &mut Context<Self>) {
        self.inputs[&Field::Source].update(cx, |state, cx| state.set_value(input, window, cx));
        self.inspect_source(cx);
    }

    fn drop_source_files(
        &mut self,
        files: &[PathBuf],
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.online {
            self.switch_draft_kind(false, window, cx);
        }
        match files {
            [path] => self.select_source_input(path.display().to_string(), window, cx),
            [] => {}
            _ => {
                self.invalidate_source();
                self.source_collection_title = Some("拖入了多个文件，请选择本次处理的视频".into());
                self.source_candidates = files
                    .iter()
                    .map(|path| source::SourceCandidate {
                        input: path.display().to_string(),
                        title: path
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .into_owned(),
                        identity: None,
                        duration: None,
                    })
                    .collect();
                cx.notify();
            }
        }
    }

    /// Box top: source-kind segments, the link input or local drop zone, reading
    /// state, collection candidates and source errors.
    pub fn box_source_input(&self, cx: &mut Context<Self>) -> Div {
        let mut view = v_flex().w_full().min_w_0().gap_4();
        view = view.child(
            h_flex().w_full().justify_center().child(
                seg_track().children([
                    seg_item("source-kind-online", self.online)
                        .icon(icons::link())
                        .label("视频链接")
                        .accessibility_label("视频链接")
                        .on_click(cx.listener(|this, _, window, cx| {
                            this.switch_draft_kind(true, window, cx)
                        })),
                    seg_item("source-kind-local", !self.online)
                        .icon(icons::movie())
                        .label("本地文件")
                        .accessibility_label("本地文件")
                        .on_click(cx.listener(|this, _, window, cx| {
                            this.switch_draft_kind(false, window, cx)
                        })),
                ]),
            ),
        );
        if self.online {
            let validation = if self.source_preview.is_none() {
                self.source_validation.clone()
            } else {
                None
            };
            view = view.child(
                v_flex()
                    .gap_2()
                    .child(
                        Input::new(&self.inputs[&Field::Source])
                            .aria_label("视频链接")
                            .appearance(false)
                            .min_h(rems(2.571))
                            .h_auto()
                            .text_size(TEXT_BODY)
                            .prefix(
                                icons::link()
                                    .size(px(16.))
                                    .text_color(rgb(FAINT)),
                            )
                            .suffix(
                                h_flex()
                                    .gap_3()
                                    .flex_shrink_0()
                                    .child(platform_badge("YouTube", 0xe23d28))
                                    .child(platform_badge("Bilibili", 0xf0699a)),
                            ),
                    )
                    .when_some(validation, |view, message| view.child(issue(message))),
            );
        } else if self.source_preview.is_none() {
            view = view.child(
                v_flex()
                    .id("video-drop-zone")
                    .gap_3()
                    .p_5()
                    .rounded(RADIUS_CARD)
                    .border_1()
                    .border_color(rgb(CONTROL))
                    .child(accessible_text(
                        "import-drop-instruction",
                        "把视频拖到这里，或选择文件。",
                    ))
                    .child(
                        outline_pill("choose-video")
                            .self_start()
                            .label(if self.value(Field::Source, cx).is_empty() {
                                "选择视频"
                            } else {
                                "更换视频"
                            })
                            .on_click(
                                cx.listener(|this, _, window, cx| this.pick(false, window, cx)),
                            ),
                    )
                    .when(!self.value(Field::Source, cx).is_empty(), |view| {
                        view.child(help(self.value(Field::Source, cx)))
                    })
                    .on_drop(
                        cx.listener(|this, paths: &gpui::ExternalPaths, window, cx| {
                            this.drop_source_files(paths.paths(), window, cx)
                        }),
                    ),
            );
        }
        if self.preview_cancel.is_some() {
            view = view.child(
                h_flex()
                    .gap_3()
                    .items_center()
                    .child(
                        gpui_component::Icon::default()
                            .path("icons/loader-circle.svg")
                            .size(px(16.))
                            .text_color(rgb(GRAY)),
                    )
                    .child(help("正在读取视频信息与字幕…"))
                    .child(
                        quiet("cancel-source-read")
                            .label("取消读取")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.invalidate_source();
                                cx.notify();
                            })),
                    ),
            );
        }
        if let Some(title) = &self.source_collection_title {
            view = view.child(
                v_flex()
                    .gap_2()
                    .child(
                        accessible_text("import-collection-title", title.clone())
                            .font_weight(FontWeight::MEDIUM),
                    )
                    .child(help("本次处理 1 个视频，选择后会读取它的完整信息。"))
                    .child(
                        v_flex()
                            .id("source-candidates")
                            .max_h(px(280.))
                            .overflow_y_scroll()
                            .rounded(RADIUS_CARD)
                            .border_1()
                            .border_color(rgb(CARD_LINE))
                            .children(self.source_candidates.iter().enumerate().map(
                                |(index, candidate)| {
                                    let input = candidate.input.clone();
                                    quiet(("source-candidate", index))
                                        .w_full()
                                        .h_auto()
                                        .py_3()
                                        .rounded(RADIUS_CARD)
                                        .justify_start()
                                        .accessibility_label(format!("读取 {}", candidate.title))
                                        .child(
                                            v_flex()
                                                .min_w_0()
                                                .gap_1()
                                                .child(candidate.title.clone())
                                                .when(candidate.title != candidate.input, |view| {
                                                    view.child(help(candidate.input.clone()))
                                                }),
                                        )
                                        .on_click(cx.listener(move |this, _, window, cx| {
                                            this.select_source_input(input.clone(), window, cx)
                                        }))
                                },
                            )),
                    ),
            );
        }
        if let Some(error) = &self.preview_error {
            view =
                view.child(
                    v_flex()
                        .gap_2()
                        .child(issue(error.lines().next().unwrap_or(error).to_owned()))
                        .child(
                            h_flex()
                                .gap_2()
                                .child(outline_pill("retry-source").label("重新读取").on_click(
                                    cx.listener(|this, _, _, cx| this.inspect_source(cx)),
                                ))
                                .when(
                                    self.online
                                        && source::is_login_failure(error)
                                        && course2md::auth::is_bilibili_url(
                                            &self.value(Field::Source, cx),
                                        ),
                                    |row| {
                                        row.child(
                                            outline_pill("source-login-repair")
                                                .label("登录 Bilibili")
                                                .on_click(cx.listener(|this, _, window, cx| {
                                                    this.open_account_dialog(window, cx)
                                                })),
                                        )
                                    },
                                )
                                .child(
                                    quiet("source-error-details")
                                        .label("详细原因")
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.show_preview_details = !this.show_preview_details;
                                            cx.notify();
                                        })),
                                ),
                        )
                        .when(self.show_preview_details, |view| {
                            view.child(help(error.clone()))
                        }),
                );
        }
        view
    }

    /// Confirmed single video: cover only with real data, identity facts and the
    /// source-details disclosure.
    fn box_selected_video(&self, cx: &mut Context<Self>) -> Div {
        let Some(source) = &self.source_preview else {
            return v_flex();
        };
        let mut selected = h_flex().gap_3().items_start();
        if let Some(cover) = &source.cover {
            selected = selected.child(
                img(cover.clone())
                    .w(rems(6.857))
                    .h(rems(3.857))
                    .object_fit(ObjectFit::Cover)
                    .rounded(RADIUS_SMALL)
                    .flex_shrink_0(),
            );
        }
        selected = selected.child(
            v_flex()
                .min_w_0()
                .flex_1()
                .gap_1()
                .child(
                    accessible_text("import-selected-title", source.title.clone())
                        .font_weight(FontWeight::SEMIBOLD)
                        .whitespace_normal()
                        .text_ellipsis()
                        .line_clamp(2),
                )
                .when(!source.detail().is_empty(), |view| {
                    view.child(help(source.detail()))
                })
                .child(
                    div().child(
                        badge(BadgeKind::Neutral).child("本次处理 1 个视频"),
                    ),
                ),
        );
        box_section("所选视频")
            .child(
                v_flex()
                    .id("import-selected-source")
                    .gap_2()
                    .on_drop(
                        cx.listener(|this, paths: &gpui::ExternalPaths, window, cx| {
                            this.drop_source_files(paths.paths(), window, cx)
                        }),
                    )
                    .child(selected)
                    .child(
                        h_flex()
                            .gap_2()
                            .flex_wrap()
                            .child(
                                quiet("source-details")
                                    .self_start()
                                    .label(if self.show_preview_details {
                                        "收起来源详情"
                                    } else {
                                        "来源详情"
                                    })
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.show_preview_details = !this.show_preview_details;
                                        cx.notify();
                                    })),
                            )
                            .when(self.online, |row| {
                                row.child(
                                    quiet("reread-source")
                                        .label("重新读取视频信息")
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.inspect_source(cx)
                                        })),
                                )
                            })
                            .when(!self.online, |row| {
                                row.child(
                                    quiet("change-selected-video")
                                        .label("更换视频")
                                        .on_click(cx.listener(|this, _, window, cx| {
                                            this.pick(false, window, cx)
                                        })),
                                )
                            }),
                    )
                    .when(self.show_preview_details, |view| {
                        view.when(!source.online, |view| {
                            view.child(help(source.input.clone()))
                        })
                        .when(
                            source.online && course2md::auth::is_bilibili_url(&source.input),
                            |view| view.child(self.source_account_row(cx)),
                        )
                    }),
            )
    }

    fn text_source_view(&self, cx: &mut Context<Self>) -> Div {
        let Some(source) = &self.source_preview else {
            return v_flex();
        };
        let speech = self.import_uses_speech();
        let description = if self.subtitle_loading {
            "正在确认字幕正文…".to_owned()
        } else if !speech
            && (source.subtitle_request.is_some() || source.subtitle_read_error.is_some())
        {
            match &source.selected_subtitle {
                Some(subtitle) => format!("字幕尚未重新确认；已读的{}仍保留", subtitle.label),
                None => "所选字幕尚未确认".into(),
            }
        } else if speech {
            "识别视频声音".into()
        } else if let Some(subtitle) = &source.selected_subtitle {
            subtitle.label.clone()
        } else {
            match &source.subtitles {
                SubtitleEvidence::Unchecked => "尚未检查可读取的字幕".into(),
                SubtitleEvidence::Found { .. } => "找到字幕，请确认要使用的文字".into(),
                SubtitleEvidence::NoneFound => "未找到可直接读取的字幕".into(),
                SubtitleEvidence::Failed { .. } => "字幕未读取成功，尚不能确认是否可用".into(),
                SubtitleEvidence::Unsupported { message } => message.clone(),
            }
        };
        let mut view = box_section("笔记内容").child(
            h_flex()
                .gap_3()
                .items_baseline()
                .child(
                    div()
                        .text_color(rgb(GRAY))
                        .flex_shrink_0()
                        .child("文字来源"),
                )
                .child(
                    accessible_text("import-text-source-state", description.clone())
                        .font_weight(FontWeight::SEMIBOLD)
                        .flex_1()
                        .min_w_0(),
                )
                .child(
                    quiet("change-text-source")
                        .label(if self.show_options {
                            "收起"
                        } else {
                            "更换"
                        })
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.show_options = !this.show_options;
                            cx.notify();
                        })),
                ),
        );
        if self.subtitle_loading {
            view = view.child(
                quiet("cancel-subtitle-read")
                    .self_start()
                    .label("取消读取字幕")
                    .on_click(cx.listener(|this, _, _, cx| {
                        if let Some(cancel) = this.subtitle_cancel.take() {
                            cancel.store(true, Ordering::Relaxed);
                        }
                        this.subtitle_generation = this.subtitle_generation.wrapping_add(1);
                        this.subtitle_loading = false;
                        if let Some(source) = &mut this.source_preview {
                            source.subtitle_request = None;
                            source.subtitle_read_error = None;
                        }
                        this.save_current_draft(cx);
                        cx.notify();
                    })),
            );
        }
        if let Some(error) = &self.subtitle_error {
            view = view.child(
                v_flex()
                    .gap_2()
                    .p_3()
                    .rounded(RADIUS_CARD)
                    .bg(rgb(WARNING_BG))
                    .child(
                        accessible_text("subtitle-failed", "字幕未读取成功，尚不能确认是否可用。")
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(rgb(WARNING)),
                    )
                    .child(issue(error.clone())),
            );
            if let Some(old) = &source.selected_subtitle {
                view = view.child(
                    outline_pill("use-previous-subtitle")
                        .self_start()
                        .label(format!("继续使用已确认的{}", old.label))
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.use_confirmed_subtitle(cx);
                        })),
                );
            }
        }
        if self.show_options
            || (!speech && source.selected_subtitle.is_none())
            || self.subtitle_error.is_some()
        {
            let mut tracks = source.subtitles.tracks().to_vec();
            if let Some(pending) = &source.subtitle_request
                && !tracks.iter().any(|track| track.id == pending.id)
            {
                tracks.push(pending.clone());
            }
            let cached_id = source
                .selected_subtitle
                .as_ref()
                .map(|subtitle| subtitle.track_id.clone());
            let mut choices: Vec<_> = tracks
                .iter()
                .map(|track| {
                    let mut label = track.label();
                    if !speech
                        && source
                            .subtitle_request
                            .as_ref()
                            .is_some_and(|pending| pending.id == track.id)
                    {
                        label.push_str(if self.subtitle_loading {
                            "（正在读取正文）"
                        } else {
                            "（正文尚未确认）"
                        });
                    }
                    (format!("track:{}", track.id), label)
                })
                .collect();
            if let Some(cached) = &source.selected_subtitle
                && !tracks.iter().any(|track| track.id == cached.track_id)
            {
                choices.push((
                    format!("track:{}", cached.track_id),
                    format!("{}（已读正文）", cached.label),
                ));
            }
            choices.push(("speech".into(), "识别视频声音".into()));
            let selected = if speech {
                Some("speech".to_owned())
            } else {
                source
                    .subtitle_request
                    .as_ref()
                    .map(|track| track.id.as_str())
                    .or(cached_id.as_deref())
                    .map(|id| format!("track:{id}"))
            };
            let mut options = v_flex().gap_2().child(
                div()
                    .rounded(RADIUS_CARD)
                    .border_1()
                    .border_color(rgb(CARD_LINE))
                    .p_2()
                    .child(
                        SingleChoiceGroup::new("import-text-source", "笔记的文字来源")
                            .options(choices)
                            .when_some(selected, |group, value| group.selected(value))
                            .on_change(cx.listener(move |this, value: &SharedString, _, cx| {
                                if value.as_ref() == "speech" {
                                    this.use_speech(cx);
                                } else if let Some(id) = value.strip_prefix("track:") {
                                    if cached_id.as_deref() == Some(id) {
                                        this.use_confirmed_subtitle(cx);
                                    } else if let Some(track) =
                                        tracks.iter().find(|track| track.id == id)
                                    {
                                        this.confirm_subtitle(track.clone(), true, cx);
                                    }
                                }
                            })),
                    ),
            );
            if let SubtitleEvidence::Failed { message } = &source.subtitles {
                options = options.child(help(message.clone()));
            }
            if let SubtitleEvidence::Found {
                warning: Some(message),
                ..
            } = &source.subtitles
            {
                options = options.child(help(format!("部分字幕尚未确认：{message}")));
            }
            if let SubtitleEvidence::Unsupported { message } = &source.subtitles {
                options = options.child(help(message.clone()));
            }
            if matches!(source.subtitles, SubtitleEvidence::NoneFound) {
                options = options.child(help("未找到可直接读取的字幕，可以识别视频声音。"));
            }
            options = options.child(
                h_flex()
                    .gap_2()
                    .flex_wrap()
                    .child(
                        outline_pill("retry-subtitles")
                            .label("重新读取字幕")
                            .disabled(self.subtitle_loading)
                            .on_click(cx.listener(|this, _, _, cx| this.retry_subtitles(cx))),
                    )
                    .when(!source.online, |view| {
                        view.child(
                            outline_pill("attach-subtitles").label("选择字幕文件").on_click(
                                cx.listener(|this, _, window, cx| {
                                    this.choose_attached_subtitle(window, cx)
                                }),
                            ),
                        )
                    })
                    .when(
                        source.online
                            && course2md::auth::is_bilibili_url(&source.input)
                            && matches!(&source.subtitles, SubtitleEvidence::Failed { message } if source::is_login_failure(message)),
                        |view| {
                            view.child(
                                outline_pill("login-for-subtitles")
                                    .label("登录 Bilibili")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.open_account_dialog(window, cx)
                                    })),
                            )
                        },
                    ),
            );
            view = view.child(options);
        }
        if speech {
            view = view.child(self.import_speech_options(cx));
        }
        let ai_overridden = {
            let defaults = ConversionOptions::from_config(&self.preferences.defaults_config());
            (self.task_options.llm, self.task_options.summarize, self.task_options.vision)
                != (defaults.llm, defaults.summarize, defaults.vision)
        };
        view = view.child(
            v_flex()
                .gap_3()
                .pt_2()
                .child(crate::settings_ui::preference(
                    "AI 校对",
                    "修正识别错误和标点，保留原意",
                    coral_switch(
                        Switch::new("import-proofread")
                            .checked(self.task_options.llm)
                            .on_click(cx.listener(|this, value, _, cx| {
                                this.task_options.llm = *value;
                                this.save_current_draft(cx);
                                cx.notify();
                            })),
                    ),
                ))
                .child(if self.task_options.llm {
                    crate::settings_ui::preference(
                        "发送截图辅助校对",
                        "文字及对应截图会发送到所选服务",
                        coral_switch(
                            Switch::new("import-vision")
                                .checked(self.task_options.vision)
                                .on_click(cx.listener(|this, value, _, cx| {
                                    this.task_options.vision = *value;
                                    this.save_current_draft(cx);
                                    cx.notify();
                                })),
                        ),
                    )
                } else {
                    h_flex()
                        .w_full()
                        .min_h(rems(44.0 / 14.0))
                        .gap_4()
                        .items_center()
                        .child(
                            v_flex()
                                .flex_1()
                                .min_w_0()
                                .gap_1()
                                .child(
                                    div()
                                        .text_color(rgb(FAINT))
                                        .child("发送截图辅助校对"),
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(rgb(FAINT))
                                        .child("开启 AI 校对后可用；开启后文字及对应截图会发送到所选服务。"),
                                ),
                        )
                })
                .child(crate::settings_ui::preference(
                    "生成摘要",
                    "提炼课程要点，正文继续保留",
                    coral_switch(
                        Switch::new("import-summary")
                            .checked(self.task_options.summarize)
                            .on_click(cx.listener(|this, value, _, cx| {
                                this.task_options.summarize = *value;
                                this.save_current_draft(cx);
                                cx.notify();
                            })),
                    ),
                )),
        );
        if self.task_options.llm || self.task_options.summarize {
            match self.selected_task_service(ServicePurpose::Ai) {
                Some(service) => {
                    if self.task_options.llm {
                        view = view.child(help(format!(
                            "校对文字将发送到「{}」（{}）。",
                            service.config.name,
                            service.config.host()
                        )));
                    }
                }
                None => {
                    view = view.child(
                        v_flex()
                            .gap_2()
                            .p_3()
                            .rounded(RADIUS_CARD)
                            .bg(rgb(WARNING_BG))
                            .child(
                                accessible_text("ai-service-missing", "AI 服务尚未设置。")
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(rgb(WARNING)),
                            )
                            .child(help("使用 AI 校对或生成摘要需要先把文字发送到一个 AI 服务。"))
                            .child(
                                outline_pill("configure-ai-service")
                                    .self_start()
                                    .label("设置 AI 服务")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.open_task_service_editor(ServicePurpose::Ai, window, cx)
                                    })),
                            ),
                    );
                }
            }
            view = view.child(self.task_service_picker(ServicePurpose::Ai, cx));
            if self.task_options.llm {
                let prompt = self.import_base_config().llm.prompt;
                if prompt
                    .as_ref()
                    .is_some_and(|prompt| !prompt.trim().is_empty())
                {
                    view = view.child(help("本次校对会使用自定义规则。"));
                    if self.show_options {
                        view = view.child(help(prompt.unwrap_or_default()));
                    }
                }
            }
        }
        if ai_overridden {
            view = view.child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(badge(BadgeKind::Neutral).child("仅用于这次笔记"))
                    .child(
                        quiet("reset-task-overrides")
                            .label("恢复默认")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.reset_task_ai_overrides(cx)
                            })),
                    ),
            );
        }
        view
    }

    fn reset_task_ai_overrides(&mut self, cx: &mut Context<Self>) {
        let defaults = ConversionOptions::from_config(&self.preferences.defaults_config());
        self.task_options.llm = defaults.llm;
        self.task_options.summarize = defaults.summarize;
        self.task_options.vision = defaults.vision;
        self.save_current_draft(cx);
        cx.notify();
    }

    fn import_speech_options(&self, cx: &mut Context<Self>) -> Div {
        let cloud = self.task_options.provider == 5;
        let mut view = v_flex().gap_3().child(
            SingleChoiceGroup::new("import-speech-location", "在哪里识别视频声音")
                .options([("local", "在这台电脑上识别"), ("cloud", "使用识别服务")])
                .selected(if cloud { "cloud" } else { "local" })
                .on_change(cx.listener(|this, value: &SharedString, _, cx| {
                    this.task_options.provider = if value.as_ref() == "cloud" { 5 } else { 0 };
                    this.save_current_draft(cx);
                    cx.notify();
                })),
        );
        if cloud {
            return view.child(self.task_service_picker(ServicePurpose::Speech, cx));
        }
        view = view
            .child(help(format!(
                "识别方式：{}；音频在本机处理。",
                self.local_engine_name()
            )))
            .child(
                control("local-engine-choices")
                    .ghost()
                    .self_start()
                    .label("选择本机识别方式")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.show_engine_details = !this.show_engine_details;
                        cx.notify();
                    })),
            );
        if self.show_engine_details {
            view = view.child(
                SingleChoiceGroup::new("import-local-engine", "本机识别方式")
                    .options(
                        PROVIDERS[..5]
                            .iter()
                            .enumerate()
                            .filter(|(index, _)| {
                                *index == 0
                                    || *index == 3
                                    || *index == self.task_options.provider
                                    || self.environment.as_ref().is_some_and(|environment| {
                                        match index {
                                            1 => environment.apple,
                                            2 => environment.gpu.is_some(),
                                            4 => environment.npu,
                                            _ => false,
                                        }
                                    })
                            })
                            .map(|(index, (_, label))| {
                                (
                                    index.to_string(),
                                    if index == 0 {
                                        "应用推荐方式"
                                    } else {
                                        *label
                                    },
                                )
                            }),
                    )
                    .selected(self.task_options.provider.to_string())
                    .on_change(cx.listener(|this, value: &SharedString, _, cx| {
                        if let Ok(index) = value.parse::<usize>() {
                            this.task_options.provider = index;
                            this.save_current_draft(cx);
                            cx.notify();
                        }
                    })),
            );
        }
        let (provider, model, root) = self.import_model_request();
        view.child(self.model_readiness_panel(provider, Some(&model), &root, cx))
    }

    fn import_uses_speech(&self) -> bool {
        self.source_preview
            .as_ref()
            .is_some_and(|source| uses_speech(source, self.task_options.source_mode))
    }

    fn import_model_request(&self) -> (course2md::config::AsrProvider, String, PathBuf) {
        use course2md::config::AsrProvider;
        let provider = self.actual_local_provider();
        let config = self.import_base_config();
        let model = config
            .defaults
            .asr_model
            .filter(|model| !model.trim().is_empty())
            .unwrap_or_else(|| {
                if provider == AsrProvider::Npu {
                    course2md::npu::resolve_npu_model(None)
                } else {
                    "qwen3-1.7b".into()
                }
            });
        let root = course2md::config::model_dir_from(config.defaults.model_dir.as_deref());
        (provider, model, root)
    }

    fn actual_local_provider(&self) -> course2md::config::AsrProvider {
        use course2md::config::AsrProvider;
        match self.task_options.provider {
            1 => AsrProvider::Coreml,
            2 => AsrProvider::Gpu,
            3 => AsrProvider::Cpu,
            4 => AsrProvider::Npu,
            _ => self.recommended_local_provider(),
        }
    }
    fn local_engine_name(&self) -> &'static str {
        use course2md::config::AsrProvider;
        match self.actual_local_provider() {
            AsrProvider::Coreml => "Apple 原生",
            AsrProvider::Gpu => "GPU",
            AsrProvider::Cpu => "CPU",
            AsrProvider::Npu => "Intel NPU",
            AsrProvider::Api => "识别服务",
        }
    }

    fn import_destination(&self, cx: &mut Context<Self>) -> Div {
        let current = self
            .workspace
            .as_ref()
            .and_then(|workspace| workspace.state.draft())
            .map(|draft| draft.library_id.clone());
        let libraries = self
            .workspace
            .as_ref()
            .map(|workspace| workspace.state.libraries.clone())
            .unwrap_or_default();
        let selected = libraries
            .iter()
            .find(|library| Some(&library.id) == current.as_ref())
            .cloned();
        let mut view = box_section("名称与保存").child(self.input(Field::Title, "笔记名称", cx));
        view = view.child(
            accessible_text("import-destination-label", "保存到").font_weight(FontWeight::MEDIUM),
        );
        if libraries.len() > 1 {
            let entity = cx.entity().downgrade();
            let label = selected
                .as_ref()
                .map(|library| format!("{} · {}", library.name, library.root.display()))
                .unwrap_or_else(|| "选择保存位置".into());
            view = view.child(
                control("import-library")
                    .w_full()
                    .justify_start()
                    .label(label)
                    .dropdown_menu(move |menu, _, _| {
                        libraries.iter().fold(menu, |menu, library| {
                            let id = library.id.clone();
                            let entity = entity.clone();
                            menu.item(
                                PopupMenuItem::new(format!(
                                    "{} · {}",
                                    library.name,
                                    library.root.display()
                                ))
                                .checked(current.as_ref() == Some(&id))
                                .on_click(move |_, _, cx| {
                                    let _ = entity.update(cx, |this, cx| {
                                        if !this.save_current_draft(cx) {
                                            return;
                                        }
                                        if let Some(workspace) = &mut this.workspace {
                                            match workspace.transaction(|state| {
                                                if let Some(draft) = state.draft_mut() {
                                                    draft.library_id = id.clone();
                                                    draft.folder = None;
                                                }
                                                Ok(())
                                            }) {
                                                Ok(()) => this.target_folder = None,
                                                Err(error) => {
                                                    this.workspace_error =
                                                        Some(format!("保存位置尚未更新：{error:#}"))
                                                }
                                            }
                                        }
                                        cx.notify();
                                    });
                                }),
                            )
                        })
                    }),
            );
        } else if let Some(library) = &selected {
            view = view.child(help(library.name.clone()));
        }
        view = view.child(
            h_flex()
                .gap_2()
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .child(self.folder_picker(None, 0, cx)),
                )
                .child(outline_pill("import-new-folder").label("新建文件夹").on_click(
                    cx.listener(|this, _, window, cx| this.begin_folder(None, window, cx)),
                )),
        );
        if let Some(library) = selected {
            view = view.child(help(library.root.display().to_string()));
        }
        view
    }

    fn import_exports(&self, cx: &mut Context<Self>) -> Div {
        let labels = ["Markdown 包", "网页文件", "JSON 数据"];
        let selected: Vec<_> = labels
            .iter()
            .enumerate()
            .filter(|(index, _)| self.task_options.formats[*index])
            .map(|(_, label)| *label)
            .collect();
        let mut view = box_section("可选产物").child(
            quiet("show-export-options")
                .self_start()
                .label(if self.show_export_options {
                    "收起导出选项"
                } else {
                    "同时导出文件"
                })
                .icon(if self.show_export_options {
                    IconName::ChevronUp
                } else {
                    IconName::ChevronDown
                })
                .on_click(cx.listener(|this, _, _, cx| {
                    this.show_export_options = !this.show_export_options;
                    cx.notify();
                })),
        );
        if !selected.is_empty() {
            view = view.child(help(format!("同时导出：{}。", selected.join("、"))));
        }
        if self.show_export_options {
            view = view.child(help(
                "笔记会保存在课程库中。需要在其他应用使用时，可以同时导出；也可以完成后再导出。",
            ));
            for (index, (label, description)) in [
                ("Markdown 包", "在笔记软件中编辑，ZIP 包含文稿和图片"),
                ("网页文件", "在浏览器中阅读和分享，单个 HTML 内嵌图片"),
                ("JSON 数据", "供程序处理，包含正文、图片 ID 和时间信息"),
            ]
            .into_iter()
            .enumerate()
            {
                view = view.child(
                    v_flex()
                        .gap_1()
                        .child(
                            Checkbox::new(("import-export", index))
                                .label(label)
                                .checked(self.task_options.formats[index])
                                .min_h(px(32.))
                                .on_click(cx.listener(move |this, value, _, cx| {
                                    this.task_options.formats[index] = *value;
                                    this.save_current_draft(cx);
                                    cx.notify();
                                })),
                        )
                        .child(help(description)),
                );
            }
        }
        if !selected.is_empty() {
            view = view.child(help(
                "导出文件保存在本版笔记的 exports 文件夹；完成后可在任务中打开。",
            ));
        }
        if self.online {
            view = view.child(crate::settings_ui::preference(
                "保留视频供离线播放",
                "生成后保留下载的视频，会占用额外空间",
                coral_switch(
                    Switch::new("import-keep-video")
                        .checked(self.task_options.keep_video)
                        .on_click(cx.listener(|this, value, _, cx| {
                            this.task_options.keep_video = *value;
                            this.save_current_draft(cx);
                            cx.notify();
                        })),
                ),
            ));
        }
        view
    }

    fn existing_source_note(&self) -> Option<Course> {
        let source = self.source_preview.as_ref()?;
        let preferred = self
            .workspace
            .as_ref()
            .and_then(|workspace| {
                workspace
                    .state
                    .draft()
                    .and_then(|draft| workspace.state.library(&draft.library_id))
            })
            .map(|library| &library.root);
        self.courses
            .iter()
            .filter(|course| {
                course
                    .manifest
                    .as_ref()
                    .is_some_and(|manifest| manifest.source_id == source.identity)
            })
            .min_by_key(|course| !preferred.is_some_and(|root| course.dir.starts_with(root)))
            .cloned()
    }

    fn generation_summary(&self) -> Vec<String> {
        let Some(source) = &self.source_preview else {
            return Vec::new();
        };
        let mut lines = Vec::new();
        let subtitle = (self.task_options.source_mode != 2)
            .then(|| source.selected_subtitle.as_ref())
            .flatten();
        if self.subtitle_loading {
            lines.push("文字来源：正在确认字幕正文".into());
        } else if self.task_options.source_mode != 2
            && (source.subtitle_request.is_some() || source.subtitle_read_error.is_some())
        {
            lines.push("文字来源：所选字幕尚未确认".into());
        } else if let Some(subtitle) = subtitle {
            lines.push(format!("文字来源：已读取的{}", subtitle.label));
        } else if !self.import_uses_speech() {
            lines.push("文字来源尚未确认".into());
        } else if self.task_options.provider == 5 {
            if let Some(service) = self.selected_task_service(ServicePurpose::Speech) {
                lines.push(format!(
                    "音频发送到「{}」识别（{}）",
                    service.config.name,
                    service.config.host()
                ));
            } else {
                lines.push("语音识别服务尚未选择".into());
            }
        } else {
            lines.push(format!(
                "音频在这台电脑上识别（{}）",
                self.local_engine_name()
            ));
        }
        if self.task_options.llm || self.task_options.summarize {
            if let Some(service) = self.selected_task_service(ServicePurpose::Ai) {
                let content = if self.task_options.llm && self.task_options.vision {
                    "文字和截图"
                } else {
                    "文字"
                };
                let action = match (self.task_options.llm, self.task_options.summarize) {
                    (true, true) => "校对并生成摘要",
                    (true, false) => "校对",
                    _ => "生成摘要",
                };
                lines.push(format!(
                    "{content}发送到「{}」{action}（{}）",
                    service.config.name,
                    service.config.host()
                ));
            } else {
                lines.push("AI 服务尚未选择".into());
            }
        }
        if let Some(workspace) = &self.workspace {
            if let Some(draft) = workspace.state.draft() {
                if let Some(library) = workspace.state.library(&draft.library_id) {
                    let folder = match draft.folder {
                        None => "未分类".into(),
                        Some(id) => crate::organize::Library::load(&library.root)
                            .ok()
                            .and_then(|organization| organization.folders.get(&id).cloned())
                            .unwrap_or_else(|| "原文件夹已不可用，请重新选择".into()),
                    };
                    lines.push(format!("保存到：{} / {folder}", library.name));
                }
            }
        }
        lines
    }

    fn generation_plan(&self) -> Vec<String> {
        let Some(source) = &self.source_preview else {
            return Vec::new();
        };
        let mut lines = Vec::new();
        if let Some(draft) = self
            .workspace
            .as_ref()
            .and_then(|workspace| workspace.state.draft())
        {
            lines.push(format!("笔记名称：{}", draft.title));
        }
        if let Some(message) = self
            .ordinary_preferences_submit_issue()
            .map(|issue| issue.message)
            .or_else(|| self.submission_issue())
        {
            lines.push(format!("尚未就绪：{message}"));
        }
        lines.push(format!("本次处理 1 个视频：{}", source.title));
        lines.push(format!("视频来源：{}", source.input));
        lines.extend(self.generation_summary());
        if source.online {
            lines.push("下载本次选择的视频，提取画面。".into());
            if self.task_options.keep_video {
                lines.push("保留本次下载的视频，供以后查看。".into());
            } else {
                lines.push("不额外保留下载的视频。".into());
            }
        } else {
            lines.push("读取所选视频，提取画面；原视频留在原位置。".into());
        }
        if self.import_uses_speech() {
            if self.task_options.provider == 5 {
                if let Some(service) = self.selected_task_service(ServicePurpose::Speech) {
                    lines.push(format!(
                        "语音识别模型：{}；服务地址：{}",
                        service.config.model, service.config.endpoint
                    ));
                }
            } else {
                let (_, model, root) = self.import_model_request();
                lines.push(format!(
                    "本机识别模型：{model}；模型位置：{}",
                    root.display()
                ));
            }
        }
        if self.task_options.llm || self.task_options.summarize {
            if let Some(service) = self.selected_task_service(ServicePurpose::Ai) {
                lines.push(format!(
                    "AI 模型：{}；服务地址：{}",
                    service.config.model, service.config.endpoint
                ));
            }
            if self.task_options.llm {
                if let Some(prompt) = self
                    .import_base_config()
                    .llm
                    .prompt
                    .filter(|prompt| !prompt.trim().is_empty())
                {
                    lines.push(format!("本次校对使用的自定义规则：\n{prompt}"));
                }
            }
        }
        if let Some(library) = self.workspace.as_ref().and_then(|workspace| {
            workspace
                .state
                .draft()
                .and_then(|draft| workspace.state.library(&draft.library_id))
        }) {
            lines.push(format!("笔记保存位置：{}", library.root.display()));
        }
        lines.push("笔记保存在应用中，生成后即可阅读。".into());
        let formats: Vec<_> = ["Markdown 包", "网页文件", "JSON 数据"]
            .into_iter()
            .enumerate()
            .filter(|(index, _)| self.task_options.formats[*index])
            .map(|(_, label)| label)
            .collect();
        if !formats.is_empty() {
            lines.push(format!("同时导出：{}", formats.join("、")));
        }
        if let Some(task) = self.matching_current_task() {
            lines.push(format!("已有相同处理任务：《{}》。查看任务会保留该任务的名称和文件夹，本草稿中的更改尚未应用。", task.plan.title));
        } else if let Some(course) = self.existing_source_note() {
            lines.push(format!("这个视频已有笔记：《{}》。打开已有笔记会保留原名称和文件夹；本草稿中的更改可用于生成新版。", course.title));
        }
        lines
    }

    fn open_generation_plan(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.save_current_draft(cx) {
            return;
        }
        let desktop = cx.entity();
        let content = cx.new(|cx| PlanDialog {
            _observation: cx.observe(&desktop, |_, _, cx| cx.notify()),
            desktop,
        });
        let focus = window.focused(cx);
        window.open_dialog(cx, move |dialog, _, _| {
            let focus = focus.clone();
            dialog
                .title("本次笔记的完整计划")
                .w(px(680.))
                .margin_top(px(24.))
                .overlay_closable(false)
                .close_button(false)
                .child(content.clone())
                .on_close(move |_, window, cx| {
                    if let Some(focus) = &focus {
                        focus.focus(window, cx);
                    }
                })
        });
    }

    fn draft_picker(&self, cx: &mut Context<Self>) -> Div {
        let Some(workspace) = &self.workspace else {
            return v_flex();
        };
        let drafts: Vec<_> = workspace
            .state
            .drafts
            .iter()
            .filter(|draft| !draft.input.is_empty() || !draft.title.is_empty())
            .cloned()
            .collect();
        let current = workspace.state.current_draft.clone();
        let mut row = h_flex().gap_2().flex_wrap();
        if drafts.len() > 1 {
            let label = workspace
                .state
                .draft()
                .map(|draft| draft.label())
                .filter(|label| !label.is_empty())
                .unwrap_or_else(|| "新笔记".into());
            let entity = cx.entity().downgrade();
            row = row.child(
                control("import-drafts")
                    .label(format!("草稿：{label}"))
                    .dropdown_menu(move |menu, _, _| {
                        drafts.iter().fold(menu, |menu, draft| {
                            let id = draft.id.clone();
                            let entity = entity.clone();
                            menu.item(
                                PopupMenuItem::new(draft.label())
                                    .checked(id == current)
                                    .on_click(move |_, window, cx| {
                                        let _ = entity.update(cx, |this, cx| {
                                            this.select_draft(id.clone(), window, cx)
                                        });
                                    }),
                            )
                        })
                    }),
            );
        }
        if workspace
            .state
            .draft()
            .is_some_and(|draft| !draft.input.is_empty())
        {
            row = row
                .child(
                    control("new-import-draft")
                        .ghost()
                        .label("生成另一篇笔记")
                        .on_click(cx.listener(|this, _, window, cx| {
                            this.new_draft(this.online, false, window, cx)
                        })),
                )
                .child(
                    control("discard-import-draft")
                        .ghost()
                        .label("丢弃草稿")
                        .on_click(
                            cx.listener(|this, _, window, cx| {
                                this.discard_import_draft(window, cx)
                            }),
                        ),
                );
        }
        row
    }

    fn discard_import_draft(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(id) = self
            .workspace
            .as_ref()
            .map(|workspace| workspace.state.current_draft.clone())
        else {
            return;
        };
        let answer = window.prompt(
            PromptLevel::Warning,
            "丢弃这份草稿？",
            Some("已生成的笔记和已提交的任务会保留。"),
            &["丢弃草稿", "保留草稿"],
            cx,
        );
        cx.spawn_in(window, async move |this, cx| {
            if answer.await.ok() != Some(0) {
                return;
            }
            let _ = this.update_in(cx, |this, window, cx| {
                let defaults = ConversionOptions::from_config(&this.preferences.defaults_config());
                this.invalidate_source();
                if let Some(workspace) = &mut this.workspace {
                    match workspace.transaction(|state| {
                        state.drafts.retain(|draft| draft.id != id);
                        if let Some(draft) = state.drafts.last() {
                            state.current_draft = draft.id.clone();
                        } else {
                            state.fresh_draft(true, defaults, None);
                        }
                        Ok(())
                    }) {
                        Ok(()) => this.restore_draft(window, cx),
                        Err(error) => {
                            this.workspace_error = Some(format!("尚未丢弃草稿：{error:#}"))
                        }
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    /// Hero above the workbench box (workbench page only). Decorative wordmark.
    fn workbench_hero(&self) -> Div {
        v_flex()
            .w_full()
            .items_center()
            .gap_2()
            .pt_4()
            .child(
                h_flex().items_baseline().children([
                    div()
                        .text_size(TEXT_DISPLAY)
                        .font_weight(FontWeight::BOLD)
                        .child("course"),
                    div()
                        .text_size(TEXT_DISPLAY)
                        .font_weight(FontWeight::BOLD)
                        .text_color(rgb(ACCENT))
                        .child("2"),
                    div()
                        .text_size(TEXT_DISPLAY)
                        .font_weight(FontWeight::BOLD)
                        .child("md"),
                ]),
            )
            .child(
                div()
                    .text_color(rgb(GRAY))
                    .child("课程视频，整理成笔记。"),
            )
    }

    /// Box bottom row while no video is confirmed yet: subtitle-language
    /// preference on the left, the read action as the only primary.
    fn box_bottom_row(&self, cx: &mut Context<Self>) -> Div {
        h_flex()
            .w_full()
            .items_center()
            .gap_3()
            .child(self.subtitle_language_control(cx))
            .child(div().flex_1())
            .when(self.preview_cancel.is_none(), |row| {
                row.child(
                    primary_pill("read-source")
                        .icon(icons::arrow_forward())
                        .label("读取视频")
                        .on_click(cx.listener(|this, _, window, cx| {
                            if this.value(Field::Source, cx).is_empty() {
                                this.inputs[&Field::Source]
                                    .update(cx, |state, cx| state.focus(window, cx));
                            }
                            this.inspect_source(cx);
                        })),
                )
            })
    }

    /// Preferred subtitle language capsule; options commit through the settings path.
    fn subtitle_language_control(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let current = self
            .preferences
            .generation()
            .preferred_subtitle_languages
            .clone();
        let label = match current.as_slice() {
            [] => "跟随界面语言".to_owned(),
            [single] if single == "zh-Hans" => "简体中文".into(),
            [single] if single == "en" => "English".into(),
            other => format!("自定义：{}", other.join(", ")),
        };
        let entity = cx.entity().downgrade();
        control("subtitle-language")
            .h_auto()
            .min_h(rems(2.286))
            .rounded(RADIUS_PILL)
            .accessibility_label(format!("首选字幕语言：{label}"))
            .tooltip(label.clone())
            .child(
                div()
                    .min_w_0()
                    .whitespace_nowrap()
                    .text_ellipsis()
                    .child(format!("首选字幕语言：{label}")),
            )
            .child(Icon::new(IconName::ChevronDown).size_4().flex_shrink_0())
            .dropdown_menu(move |menu, _, _| {
                [
                    ("跟随界面语言", Vec::<String>::new()),
                    ("简体中文优先", vec!["zh-Hans".to_owned()]),
                    ("English 优先", vec!["en".to_owned()]),
                ]
                .into_iter()
                .fold(menu, |menu, (name, languages)| {
                    let checked = languages == current;
                    let entity = entity.clone();
                    menu.item(PopupMenuItem::new(name).checked(checked).on_click(
                        move |_, _, cx| {
                            let languages = languages.clone();
                            let _ = entity.update(cx, |this, cx| {
                                this.choose_preferred_subtitle_languages(languages, cx);
                            });
                        },
                    ))
                })
            })
    }

    pub fn new_page(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        if self.task_options.provider != 5 && self.import_uses_speech() {
            let (provider, model, root) = self.import_model_request();
            self.ensure_model_diagnostic(provider, Some(&model), &root, cx);
        }
        let mut view = v_flex().gap_6().w_full().min_w_0();
        view = view.child(self.workbench_hero());
        view = view.child(self.draft_picker(cx));
        let mut bx = v_flex()
            .w_full()
            .min_w_0()
            .p_6()
            .gap_4()
            .bg(rgb(SURFACE))
            .border_1()
            .border_color(rgb(HAIRLINE))
            .rounded(RADIUS_HERO)
            .shadow(shadow_hero());
        bx = bx.child(self.box_source_input(cx));
        if self.source_preview.is_none() {
            bx = bx.child(self.box_bottom_row(cx));
        } else {
            bx = bx
                .child(self.box_selected_video(cx))
                .child(self.text_source_view(cx))
                .child(self.import_destination(cx))
                .child(self.import_exports(cx))
                .child(self.plan_inset(cx));
        }
        view = view.child(bx);
        if let Some(error) = &self.workspace_error {
            view = view.child(issue(error.clone()));
        }
        view.into_any_element()
    }

    /// Plan confirmation inset at the box bottom: the facts and the one primary.
    fn plan_inset(&mut self, cx: &mut Context<Self>) -> Div {
        let busy = self.job.is_some()
            || self.workspace.as_ref().is_some_and(|workspace| {
                workspace.state.tasks.iter().any(|task| {
                    matches!(
                        task.state,
                        crate::workspace::TaskState::Queued | crate::workspace::TaskState::Running
                    )
                })
            });
        let reading = self.preview_cancel.is_some() || self.subtitle_loading;
        let preferences_issue = self.ordinary_preferences_submit_issue();
        let preferences_blocked = preferences_issue.is_some();
        let submission_error = preferences_issue
            .as_ref()
            .map(|issue| issue.message.clone())
            .or_else(|| {
                if self.source_preview.is_some() && !reading {
                    self.submission_issue()
                } else {
                    None
                }
            });
        let mut inset = v_flex()
            .w_full()
            .min_w_0()
            .flex_shrink_0()
            .gap_2()
            .p_4()
            .rounded(RADIUS_CARD)
            .bg(rgb(INSET))
            .border_1()
            .border_color(rgb(HAIRLINE))
            .child(
                accessible_text("plan-inset-label", "本次计划")
                    .text_size(TEXT_AUX)
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(rgb(GRAY)),
            );
        if let Some(source) = &self.source_preview {
            inset = inset.child(plan_row(
                if source.online {
                    "视频：下载所选视频并提取画面 · 本次处理 1 个视频".to_owned()
                } else {
                    "视频：读取所选视频并提取画面 · 原视频留在原位置".to_owned()
                },
                false,
            ));
        }
        for line in self.generation_summary() {
            let warning = line.starts_with("文字来源尚未确认")
                || line.contains("尚未选择")
                || line.contains("已不可用");
            inset = inset.child(plan_row(line, warning));
        }
        let mut actions = h_flex().gap_2().flex_wrap();
        if let Some(issue) = preferences_issue {
            actions = actions.child(
                outline_pill("repair-generation-preferences")
                    .label(if issue.can_retry {
                        "重试保存生成选项"
                    } else {
                        "保留原文件并重置生成选项"
                    })
                    .on_click(cx.listener(move |this, _, window, cx| {
                        if !this.save_current_draft(cx) {
                            return;
                        }
                        let saved = if issue.can_retry {
                            this.retry_ordinary_preferences(issue.group, cx)
                        } else {
                            this.restore_ordinary_preferences(issue.group, cx)
                        };
                        if saved {
                            let focus = this.import_submit_focus.clone();
                            window.defer(cx, move |window, cx| focus.focus(window, cx));
                        }
                    })),
            );
        }
        if let Some(library) = self
            .workspace
            .as_ref()
            .and_then(|workspace| {
                workspace
                    .state
                    .draft()
                    .and_then(|draft| workspace.state.library(&draft.library_id))
            })
            .filter(|library| crate::storage_ui::needs_reassociation(library))
        {
            let id = library.id.clone();
            actions = actions.child(
                outline_pill("reassociate-import-location")
                    .label("重新关联此保存位置")
                    .disabled(self.storage_ui.busy)
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.begin_library_reassociation(id.clone(), window, cx)
                    })),
            );
        }
        if let Some(task) = self.matching_current_task().cloned() {
            let id = task.id.clone();
            let location = self
                .workspace
                .as_ref()
                .and_then(|workspace| workspace.state.library(&task.plan.library_id))
                .map(|library| library.name.clone())
                .unwrap_or_else(|| "原保存位置".into());
            inset = inset
                .child(plan_row(
                    format!("已有相同处理任务：《{}》（{location}）", task.plan.title),
                    true,
                ))
                .child(plan_row(
                    "该任务的名称和文件夹保持不变，草稿中的更改尚未应用。",
                    false,
                ));
            actions = actions.child(
                primary_pill("show-matching-task")
                    .track_focus(&self.import_submit_focus)
                    .self_start()
                    .label("查看任务")
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.save_current_draft(cx);
                        this.select_task(&id, cx);
                        // M4: 任务详情并入工作台输入盒；先回到工作台。
                        this.page = Page::New;
                        cx.notify();
                    })),
            );
        } else if let Some(course) = self.existing_source_note() {
            let location = self
                .course_location(&course)
                .map(|library| library.name.clone())
                .unwrap_or_else(|| "课程库".into());
            inset = inset
                .child(plan_row(
                    format!("这个视频已有笔记：《{}》（{location}）", course.title),
                    false,
                ))
                .child(plan_row(
                    "打开已有笔记会保留原名称和文件夹；草稿更改可用于生成新版。",
                    false,
                ));
            actions = actions.child(
                h_flex()
                    .gap_2()
                    .flex_wrap()
                    .child(
                        primary_pill("open-existing-note")
                            .label("打开已有笔记")
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.save_current_draft(cx);
                                this.open_course(course.clone(), cx);
                            })),
                    )
                    .child(
                        outline_pill("generate-new-version")
                            .track_focus(&self.import_submit_focus)
                            .label("生成新版笔记")
                            .disabled(reading || self.workspace.is_none() || preferences_blocked)
                            .on_click(
                                cx.listener(|this, _, window, cx| this.enqueue_current(window, cx)),
                            ),
                    ),
            );
            if busy {
                inset = inset.child(plan_row(
                    "新版笔记会加入队列，等待当前任务完成。",
                    false,
                ));
            }
        } else {
            if busy {
                inset = inset.child(plan_row(
                    "加入后等待当前任务完成，可以继续准备其他笔记。",
                    false,
                ));
            }
            actions = actions.child(
                primary_pill("generate-note")
                    .track_focus(&self.import_submit_focus)
                    .self_start()
                    .icon(icons::arrow_forward())
                    .label(if busy { "加入队列" } else { "生成笔记" })
                    .disabled(reading || self.workspace.is_none() || preferences_blocked)
                    .on_click(cx.listener(|this, _, window, cx| {
                        if this.value(Field::Source, cx).is_empty() {
                            this.inputs[&Field::Source]
                                .update(cx, |state, cx| state.focus(window, cx));
                        } else if this.source_preview.is_some()
                            && this.value(Field::Title, cx).is_empty()
                        {
                            this.inputs[&Field::Title]
                                .update(cx, |state, cx| state.focus(window, cx));
                        }
                        this.enqueue_current(window, cx);
                    })),
            );
        }
        actions = actions.child(
            quiet("show-import-full-plan")
                .label("查看完整计划")
                .on_click(
                    cx.listener(|this, _, window, cx| this.open_generation_plan(window, cx)),
                ),
        );
        inset
            .when_some(submission_error, |view, message| {
                view.child(
                    accessible_text("import-submit-error", message)
                        .role(Role::Alert)
                        .text_sm()
                        .whitespace_normal()
                        .line_clamp(3)
                        .text_color(rgb(0xa32626)),
                )
            })
            .child(actions)
    }
}

#[cfg(test)]
mod tests {
    use super::uses_speech;
    use crate::source;
    use course2md::subtitle::{SubtitleEvidence, SubtitleReadError};

    #[test]
    fn caption_intent_and_unconfirmed_reads_never_imply_speech_recognition() {
        let mut source = source::Source {
            subtitles: SubtitleEvidence::NoneFound,
            ..Default::default()
        };
        assert!(uses_speech(&source, 0));
        assert!(!uses_speech(&source, 1));
        source.subtitle_request = Some(course2md::subtitle::file_track("lesson.srt".into(), None));
        assert!(!uses_speech(&source, 0));
        assert!(!uses_speech(&source, 1));
        source.subtitle_request = None;
        source.subtitle_read_error = Some(SubtitleReadError::Failed {
            message: "字幕文件无法读取".into(),
        });
        assert!(!uses_speech(&source, 0));
        assert!(uses_speech(&source, 2));
        source.subtitle_read_error = None;
        for evidence in [
            SubtitleEvidence::Unchecked,
            SubtitleEvidence::Failed {
                message: "请求失败".into(),
            },
        ] {
            source.subtitles = evidence;
            assert!(!uses_speech(&source, 0));
        }
    }
}
