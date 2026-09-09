//! One start intent continues source preparation through to a readable note.
use super::*;
use crate::{motion, preferences::ServicePurpose, theme::*};
use anyhow::Context as _;
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
        .min_w_0()
        .max_w_full()
        .whitespace_normal()
        .text_sm()
        .text_color(color(MUTED))
        .child(accessible_text(text_id("help", &text), text))
}
fn issue(message: impl Into<SharedString>) -> Div {
    let message = message.into();
    div()
        .text_sm()
        .text_color(color(DANGER))
        .whitespace_normal()
        .child(accessible_text(text_id("issue", &message), message))
}
fn text_id(kind: &str, value: &str) -> SharedString {
    use std::hash::{Hash, Hasher};
    let mut hash = std::collections::hash_map::DefaultHasher::new();
    value.hash(&mut hash);
    format!("import-{kind}-{:x}", hash.finish()).into()
}

fn service_destination(config: &crate::preferences::ServiceConfiguration) -> String {
    let host = config.host();
    if config.name == host {
        format!("「{host}」")
    } else {
        format!("「{}」（{host}）", config.name)
    }
}
/// Platform marks are brand assets; they do not belong inside the editable field.
fn platform_mark(name: &'static str, icon: Icon) -> Div {
    h_flex()
        .gap_2()
        .items_center()
        .flex_shrink_0()
        .child(icon.size(px(20.)).flex_shrink_0())
        .child(
            div()
                .text_size(TEXT_AUX)
                .text_color(color(GRAY))
                .child(name),
        )
}

/// Align an option's icon with the first text line, even when its hint wraps.
fn preference_icon(icon: Icon) -> Div {
    h_flex()
        .h(rems(1.5))
        .flex_shrink_0()
        .child(icon.size_5().text_color(color(GRAY)))
}
/// A shared heading for related conversion options.
pub(crate) fn box_section(label: &'static str) -> Div {
    let icon = match label {
        "所选视频" => icons::movie(),
        "笔记内容" | "文字来源" => icons::subtitles(),
        "名称与保存" => Icon::new(IconName::Folder),
        "导出与视频" => icons::download(),
        "本次任务" => icons::task(),
        _ => Icon::new(IconName::Info),
    };
    v_flex().w_full().min_w_0().gap_3().child(
        h_flex()
            .gap_2()
            .items_center()
            .child(icon.size(px(20.)).text_color(color(ACCENT_STRONG)))
            .child(
                accessible_text(text_id("box-section", label), label)
                    .text_size(TEXT_TITLE)
                    .font_weight(FontWeight::SEMIBOLD),
            ),
    )
}
/// A consequence or recovery fact kept beside its action.
pub(crate) fn conversion_fact(value: impl Into<SharedString>, warning: bool) -> Stateful<Div> {
    let value = value.into();
    accessible_text(text_id("conversion-fact", &value), value)
        .w_full()
        .min_w_0()
        .whitespace_normal()
        .text_size(TEXT_BODY)
        .text_color(color(if warning { WARNING } else { INK }))
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
                SubtitleEvidence::NoneFound
                    | SubtitleEvidence::Unsupported { .. }
                    | SubtitleEvidence::Failed { .. }
            ))
}

fn automatic_subtitle_fallback(
    source: &mut source::Source,
    source_mode: usize,
    error: Option<&str>,
) -> bool {
    if source_mode != 0
        || source.selected_subtitle.is_some()
        || matches!(
            source.subtitle_read_error,
            Some(SubtitleReadError::Cancelled)
        )
    {
        return false;
    }
    let failure = error.map(str::to_owned).or_else(|| {
        source
            .subtitle_read_error
            .as_ref()
            .map(ToString::to_string)
            .or_else(|| match &source.subtitles {
                SubtitleEvidence::Failed { message } => Some(message.clone()),
                _ => None,
            })
    });
    let Some(message) = failure else {
        return false;
    };
    source.subtitles = SubtitleEvidence::Failed { message };
    source.subtitle_request = None;
    source.subtitle_read_error = None;
    true
}

fn subtitle_needs_confirmation(source: &source::Source, source_mode: usize) -> bool {
    !uses_speech(source, source_mode)
        && (source.selected_subtitle.is_none()
            || source.subtitle_request.is_some()
            || source.subtitle_read_error.is_some())
}

/// Follow only this input's submitted recovery chain. A matching URL or the
/// selected queue item alone cannot establish that a task belongs to this form.
fn submitted_input_task<'a>(
    draft: &workspace::Draft,
    tasks: &'a [workspace::TaskRecord],
    input: &str,
) -> Option<&'a workspace::TaskRecord> {
    if draft.input != input || input.trim().is_empty() {
        return None;
    }
    let find = |id: &str| {
        let mut matches = tasks.iter().filter(|task| task.id == id);
        let task = matches.next()?;
        matches.next().is_none().then_some(task)
    };
    let mut task = find(draft.submitted_task.as_deref()?)?;
    let source_id = &task.plan.source_id;
    let online = task.plan.source.online;
    if source_id.is_empty()
        || task.plan.source.input != input
        || online != draft.online
        || draft
            .source
            .as_ref()
            .is_some_and(|source| source.identity != *source_id || source.online != online)
    {
        return None;
    }
    let mut visited = std::collections::BTreeSet::new();
    loop {
        if !visited.insert(task.id.as_str())
            || task.plan.source_id != *source_id
            || task.plan.source.identity != *source_id
            || task.plan.source.online != online
        {
            return None;
        }
        let Some(next) = task.handled_by.as_deref() else {
            return Some(task);
        };
        let next = find(next)?;
        if next.parent.as_deref() != Some(task.id.as_str()) {
            return None;
        }
        task = next;
    }
}

fn completed_input_task<'a>(
    draft: &workspace::Draft,
    tasks: &'a [workspace::TaskRecord],
    input: &str,
) -> Option<&'a workspace::TaskRecord> {
    submitted_input_task(draft, tasks, input)
        .filter(|task| task.state == workspace::TaskState::Complete && task.artifact.is_some())
}

/// A live navigation intent is separate from the saved task. Leaving the
/// workbench ends following without cancelling source preparation or work.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ConversionFollow {
    Preparing(u64),
    Task { id: String, source_revision: u64 },
}

impl ConversionFollow {
    pub(crate) fn submitted(self, revision: u64, id: String) -> Option<Self> {
        matches!(self, Self::Preparing(current) if current == revision).then_some(Self::Task {
            id,
            source_revision: revision,
        })
    }

    pub(crate) fn follows(&self, id: &str, revision: u64) -> bool {
        matches!(self, Self::Task { id: followed, source_revision }
            if followed == id && *source_revision == revision)
    }

    pub(crate) fn completed_task<'a>(
        &self,
        on_workbench: bool,
        revision: u64,
        draft: &workspace::Draft,
        tasks: &'a [workspace::TaskRecord],
        input: &str,
    ) -> Option<&'a workspace::TaskRecord> {
        let task = completed_input_task(draft, tasks, input)?;
        (on_workbench && self.follows(&task.id, revision)).then_some(task)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ConversionGate {
    Wait,
    Stop,
    Submit,
}

/// Normal asynchronous stages retain the first Start intent. Only an abandoned
/// source or an unresolved material choice ends this automatic continuation.
fn conversion_gate(
    requested_revision: u64,
    current_revision: u64,
    reading: bool,
    has_source: bool,
    needs_text_choice: bool,
    environment_ready: bool,
    existing_note: bool,
) -> ConversionGate {
    if requested_revision != current_revision {
        ConversionGate::Stop
    } else if reading {
        ConversionGate::Wait
    } else if !has_source {
        ConversionGate::Stop
    } else if needs_text_choice || !environment_ready {
        ConversionGate::Wait
    } else if existing_note {
        ConversionGate::Stop
    } else {
        ConversionGate::Submit
    }
}

impl Desktop {
    /// A start command owns preparation and submission for this source revision.
    pub fn start_conversion(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.clear_queued_message();
        if self.reading && self.page == Page::New {
            self.read_generation = self.read_generation.wrapping_add(1);
            self.reading = false;
        }
        let input = self.value(Field::Source, cx);
        if input != self.last_source_input {
            if !self.prepare_next_import(&input, window, cx) {
                return;
            }
            self.following_conversion = None;
            self.last_source_input = input.clone();
            self.invalidate_source();
        }
        if input.is_empty() {
            self.source_validation = Some(if self.online {
                "先粘贴视频链接".into()
            } else {
                "先选择一个视频".into()
            });
            if self.online {
                self.inputs[&Field::Source].update(cx, |input, cx| input.focus(window, cx));
            }
            cx.notify();
            return;
        }
        if self.source_preview.is_none() && self.preview_cancel.is_none() {
            self.inspect_source(window, cx);
        }
        self.pending_conversion = Some(self.preview_generation);
        self.following_conversion = (self.page == Page::New)
            .then_some(ConversionFollow::Preparing(self.preview_generation));
        self.advance_conversion(window, cx);
    }

    pub fn advance_conversion_when_ready(&mut self, cx: &mut Context<Self>) {
        if self.pending_conversion.is_none() {
            return;
        }
        let Some(handle) = cx.windows().first().copied() else {
            return;
        };
        let desktop = cx.entity().downgrade();
        cx.defer(move |cx| {
            let _ = cx.update_window(handle, |_, window, cx| {
                let _ = desktop.update(cx, |desktop, cx| desktop.advance_conversion(window, cx));
            });
        });
    }

    pub fn advance_conversion(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(generation) = self.pending_conversion else {
            return;
        };
        let reading = self.preview_cancel.is_some() || self.subtitle_loading;
        if generation == self.preview_generation
            && !reading
            && let Some(source) = &mut self.source_preview
            && automatic_subtitle_fallback(
                source,
                self.task_options.source_mode,
                self.subtitle_error.as_deref(),
            )
        {
            self.subtitle_error = None;
            if !self.save_current_draft(cx) {
                self.pending_conversion = None;
                return;
            }
        }
        match conversion_gate(
            generation,
            self.preview_generation,
            reading,
            self.source_preview.is_some(),
            self.subtitle_attention_required(),
            self.environment.is_some(),
            self.existing_source_note().is_some(),
        ) {
            ConversionGate::Wait => cx.notify(),
            ConversionGate::Stop => {
                self.pending_conversion = None;
                cx.notify();
            }
            ConversionGate::Submit => self.enqueue_current(window, cx),
        }
    }

    fn subtitle_issue(
        &self,
        kind: &str,
        message: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Div {
        let summary = if source::is_login_failure(message) {
            "字幕需要登录后才能读取；可以登录，或改用语音识别"
        } else if message.contains("WARNING:") || message.contains("ERROR:") {
            "暂时无法读取字幕；可以重试，或改用语音识别"
        } else {
            message.lines().next().unwrap_or(message)
        };
        let details = (summary != message).then_some(message).unwrap_or("");
        let id = text_id(kind, message);
        let expanded = self.expanded_subtitle_issue.as_ref() == Some(&id);
        v_flex()
            .min_w_0()
            .gap_2()
            .child(help(summary.to_owned()))
            .when(!details.trim().is_empty(), |view| {
                view.child(
                    quiet(id.clone())
                        .icon(if expanded {
                            IconName::ChevronUp
                        } else {
                            IconName::ChevronDown
                        })
                        .self_start()
                        .label(if expanded {
                            "收起技术详情"
                        } else {
                            "技术详情"
                        })
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.expanded_subtitle_issue =
                                if expanded { None } else { Some(id.clone()) };
                            cx.notify();
                        })),
                )
                .child(motion::disclosure(
                    text_id("subtitle-technical", message),
                    expanded,
                    v_flex().child(help(details.to_owned())),
                    window,
                    cx,
                ))
            })
    }

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
                        .is_some_and(|workspace| workspace.state.matches_input(id, *revision))
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
                    Err(SubtitleReadError::Cancelled) => {
                        this.pending_conversion = None;
                    }
                    Err(error) => {
                        this.subtitle_error = Some(error.to_string());
                        if let Some(source) = &mut this.source_preview {
                            source.subtitle_read_error = Some(error);
                        }
                        this.save_current_draft(cx);
                    }
                }
                this.advance_conversion_when_ready(cx);
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
                        .workspace.as_ref()
                        .is_some_and(|workspace| workspace.state.matches_input(id, *revision)))
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
                        this.subtitle_error = Some(format!("{error:#}"));
                        if let Some(source) = &mut this.source_preview {
                            source.subtitle_read_error = this.subtitle_error.clone().map(|message| SubtitleReadError::Failed { message });
                        }
                        this.save_current_draft(cx);
                    }
                }
                this.advance_conversion_when_ready(cx);
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
        self.advance_conversion_when_ready(cx);
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
        self.advance_conversion_when_ready(cx);
        cx.notify();
    }

    fn select_source_input(&mut self, input: String, window: &mut Window, cx: &mut Context<Self>) {
        self.inputs[&Field::Source].update(cx, |state, cx| state.set_value(input, window, cx));
        self.start_conversion(window, cx);
    }

    fn drop_source_files(
        &mut self,
        files: &[PathBuf],
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.online {
            self.switch_source_kind(false, window, cx);
        }
        match files {
            [path] => {
                self.inputs[&Field::Source].update(cx, |state, cx| {
                    state.set_value(path.display().to_string(), window, cx)
                });
            }
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

    /// Source navigation is separate from the editable surface.
    fn source_kind_tabs(&self, cx: &mut Context<Self>) -> Div {
        div().w_full().child(
            SingleChoiceGroup::new("source-kind", "视频来源")
                .options([("online", "视频链接"), ("local", "本地文件")])
                .icon("online", icons::link())
                .icon("local", icons::movie())
                .selected(if self.online { "online" } else { "local" })
                .on_change(cx.listener(|this, value: &SharedString, window, cx| {
                    this.switch_source_kind(value.as_ref() == "online", window, cx);
                })),
        )
    }

    pub(super) fn subtitle_attention_required(&self) -> bool {
        self.subtitle_loading
            || self.subtitle_error.is_some()
            || self.source_preview.as_ref().is_some_and(|source| {
                subtitle_needs_confirmation(source, self.task_options.source_mode)
            })
    }

    /// Visible field, local file target, and status belonging to this source.
    pub fn box_source_input(&self, window: &mut Window, cx: &mut Context<Self>) -> Div {
        let mut view = v_flex().w_full().min_w_0().gap_4();
        if self.online {
            let validation = self
                .source_preview
                .is_none()
                .then(|| self.source_validation.clone())
                .flatten();
            view = view.child(
                v_flex()
                    .gap_2()
                    .child(
                        accessible_text("import-url-label", "视频链接")
                            .font_weight(FontWeight::MEDIUM),
                    )
                    .child(
                        h_flex()
                            .w_full()
                            .min_w_0()
                            .gap(px(12.))
                            .items_center()
                            .child(
                                div().flex_1().min_w_0().child(
                                    text_input(&self.inputs[&Field::Source])
                                        .aria_label("视频链接")
                                        .w_full()
                                        .min_w_0()
                                        .text_size(TEXT_BODY)
                                        .prefix(
                                            icons::link()
                                                .size(rems(18. / 14.))
                                                .text_color(color(GRAY)),
                                        )
                                        .when(validation.is_some(), |input| {
                                            input.border_color(color(DANGER))
                                        }),
                                ),
                            )
                            .when(self.can_start_input(cx), |row| {
                                row.child(self.box_bottom_row(cx))
                            }),
                    )
                    .when_some(validation, |view, message| view.child(issue(message))),
            );
            view = view.child(
                h_flex()
                    .gap_4()
                    .flex_wrap()
                    .items_center()
                    .child(platform_mark(
                        "YouTube",
                        icons::youtube().text_color(rgb(0xff0033)),
                    ))
                    .child(platform_mark(
                        "Bilibili",
                        icons::bilibili().text_color(rgb(0x00a1d6)),
                    )),
            );
        } else {
            let input = self.value(Field::Source, cx);
            if input.is_empty() {
                view = view.child(
                    v_flex()
                        .id("video-drop-zone")
                        .gap_4()
                        .p_6()
                        .items_center()
                        .rounded(RADIUS_CARD)
                        .bg(color(INSET))
                        .border_1()
                        .border_color(color(CONTROL))
                        .child(
                            icons::file_upload()
                                .size(rems(32. / 14.))
                                .text_color(color(ACCENT_STRONG)),
                        )
                        .child(
                            accessible_text("import-drop-instruction", "拖入本地视频")
                                .text_size(TEXT_BODY)
                                .font_weight(FontWeight::MEDIUM),
                        )
                        .child(
                            primary_pill("choose-video")
                                .icon(IconName::FolderOpen)
                                .label("选择视频")
                                .on_click(
                                    cx.listener(|this, _, window, cx| this.pick(false, window, cx)),
                                ),
                        )
                        .on_drop(
                            cx.listener(|this, paths: &gpui::ExternalPaths, window, cx| {
                                this.drop_source_files(paths.paths(), window, cx)
                            }),
                        ),
                );
            } else {
                let filename = PathBuf::from(&input)
                    .file_name()
                    .map(|name| name.to_string_lossy().into_owned())
                    .filter(|name| !name.is_empty())
                    .unwrap_or_else(|| input.clone());
                view = view.child(
                    h_flex()
                        .id("selected-local-video")
                        .w_full()
                        .min_w_0()
                        .items_center()
                        .flex_wrap()
                        .gap_3()
                        .p_4()
                        .rounded(RADIUS_CARD)
                        .bg(color(SURFACE))
                        .border_1()
                        .border_color(color(HAIRLINE))
                        .child(
                            semantic_label(
                                "selected-local-video-name",
                                filename,
                                icons::movie()
                                    .size(rems(20. / 14.))
                                    .text_color(color(ACCENT_STRONG)),
                            )
                            .flex_1()
                            .min_w(rems(18.))
                            .max_w_full(),
                        )
                        .child(
                            h_flex()
                                .gap_2()
                                .flex_wrap()
                                .items_center()
                                .child(
                                    outline_pill("choose-video")
                                        .icon(IconName::FolderOpen)
                                        .label("更换视频")
                                        .on_click(cx.listener(|this, _, window, cx| {
                                            this.pick(false, window, cx)
                                        })),
                                )
                                .when(self.can_start_input(cx), |row| {
                                    row.child(self.box_bottom_row(cx))
                                }),
                        )
                        .on_drop(
                            cx.listener(|this, paths: &gpui::ExternalPaths, window, cx| {
                                this.drop_source_files(paths.paths(), window, cx)
                            }),
                        ),
                );
                if self.source_preview.is_none()
                    && let Some(error) = &self.source_validation
                {
                    view = view.child(issue(error.clone()));
                }
            }
        }
        if self.preview_cancel.is_some() {
            view = view.child(motion::enter(
                text_id("reading", &self.preview_generation.to_string()),
                h_flex()
                    .w_full()
                    .gap_3()
                    .p_3()
                    .items_center()
                    .rounded(RADIUS_SMALL)
                    .bg(color(INSET))
                    .child(motion::spinner("source-reading-spinner", cx))
                    .child(
                        accessible_text("import-reading-state", "正在读取视频信息与字幕…")
                            .flex_1()
                            .min_w_0(),
                    )
                    .child(
                        quiet("cancel-source-read")
                            .icon(IconName::Close)
                            .label("取消")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.invalidate_source();
                                cx.notify();
                            })),
                    ),
                cx,
            ));
        }
        if let Some(title) = &self.source_collection_title {
            view = view.child(motion::enter(
                text_id("source-candidates", title),
                v_flex()
                    .gap_3()
                    .child(
                        accessible_text("import-collection-title", title.clone())
                            .font_weight(FontWeight::SEMIBOLD),
                    )
                    .child(help("选择本次要整理的视频"))
                    .child(
                        v_flex()
                            .id("source-candidates")
                            .max_h(px(280.))
                            .overflow_y_scroll()
                            .rounded(RADIUS_CARD)
                            .border_1()
                            .border_color(color(CARD_LINE))
                            .children(self.source_candidates.iter().enumerate().map(
                                |(index, candidate)| {
                                    let input = candidate.input.clone();
                                    let untitled = candidate.title.trim().is_empty()
                                        || candidate.title.trim() == candidate.input.trim();
                                    let title = if untitled {
                                        format!("视频 {}", index + 1)
                                    } else {
                                        candidate.title.clone()
                                    };
                                    quiet(("source-candidate", index))
                                        .w_full()
                                        .h_auto()
                                        .py_3()
                                        .rounded(RADIUS_SMALL)
                                        .justify_start()
                                        .icon(icons::movie())
                                        .tooltip(candidate.input.clone())
                                        .accessibility_label(format!("选择 {title}"))
                                        .child(
                                            v_flex()
                                                .flex_1()
                                                .min_w_0()
                                                .items_start()
                                                .gap_1()
                                                .child(
                                                    div()
                                                        .w_full()
                                                        .min_w_0()
                                                        .whitespace_normal()
                                                        .text_ellipsis()
                                                        .line_clamp(2)
                                                        .font_weight(FontWeight::MEDIUM)
                                                        .child(title),
                                                )
                                                .when(untitled, |view| {
                                                    view.child(
                                                        div()
                                                            .w_full()
                                                            .min_w_0()
                                                            .text_size(TEXT_AUX)
                                                            .text_color(color(MUTED))
                                                            .text_ellipsis()
                                                            .child(candidate.input.clone()),
                                                    )
                                                }),
                                        )
                                        .child(icons::arrow_forward().size(px(18.)).flex_shrink_0())
                                        .on_click(cx.listener(move |this, _, window, cx| {
                                            this.select_source_input(input.clone(), window, cx)
                                        }))
                                },
                            )),
                    ),
                cx,
            ));
        }
        if let Some(error) = &self.preview_error
            && !self.source_candidates.is_empty()
        {
            view = view.child(help(error.clone()));
        }
        if let Some(error) = &self.preview_error
            && self.source_candidates.is_empty()
        {
            let details = motion::disclosure(
                "source-error-disclosure",
                self.show_preview_details,
                v_flex().child(help(error.clone())),
                window,
                cx,
            );
            view = view.child(motion::enter(
                text_id("source-error", error),
                v_flex()
                    .gap_3()
                    .p_4()
                    .rounded(RADIUS_CARD)
                    .bg(color(DANGER_BG))
                    .child(
                        h_flex()
                            .gap_2()
                            .items_start()
                            .child(
                                Icon::new(IconName::CircleX)
                                    .size(px(20.))
                                    .text_color(color(DANGER))
                                    .flex_shrink_0(),
                            )
                            .child(
                                issue(if source::is_login_failure(error) {
                                    "视频需要登录后读取，请登录后重试转换".to_owned()
                                } else if error.contains("WARNING:") || error.contains("ERROR:") {
                                    "暂时无法读取这个视频，请检查链接或稍后重试".to_owned()
                                } else {
                                    error.lines().next().unwrap_or(error).to_owned()
                                })
                                .flex_1()
                                .min_w_0(),
                            ),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .flex_wrap()
                            .child(
                                outline_pill("retry-source")
                                    .icon(icons::refresh())
                                    .label("重试转换")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.start_conversion(window, cx)
                                    })),
                            )
                            .when(
                                self.online
                                    && source::is_login_failure(error)
                                    && course2md::auth::is_bilibili_url(
                                        &self.value(Field::Source, cx),
                                    ),
                                |row| {
                                    row.child(
                                        outline_pill("source-login-repair")
                                            .icon(icons::bilibili())
                                            .label("登录 Bilibili")
                                            .on_click(cx.listener(|this, _, window, cx| {
                                                this.open_account_dialog(window, cx)
                                            })),
                                    )
                                },
                            )
                            .child(
                                quiet("source-error-details")
                                    .icon(if self.show_preview_details {
                                        IconName::ChevronUp
                                    } else {
                                        IconName::ChevronDown
                                    })
                                    .label(if self.show_preview_details {
                                        "收起详情"
                                    } else {
                                        "技术详情"
                                    })
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.show_preview_details = !this.show_preview_details;
                                        cx.notify();
                                    })),
                            ),
                    )
                    .child(details),
                cx,
            ));
        }
        view
    }

    /// The video identity remains compact; paths and account tools are details.
    fn box_selected_video(&self, window: &mut Window, cx: &mut Context<Self>) -> Div {
        let Some(source) = &self.source_preview else {
            return v_flex();
        };
        let can_close_input = self.source_editor_open
            && self.value(Field::Source, cx) == source.input
            && self
                .workspace
                .as_ref()
                .and_then(|workspace| workspace.state.draft())
                .is_none_or(|input| input.submitted_task.is_none());
        let mut selected = h_flex().gap_4().items_start();
        if let Some(cover) = &source.cover {
            selected = selected.child(
                img(cover.clone())
                    .w(rems(8.))
                    .h(rems(4.5))
                    .object_fit(ObjectFit::Cover)
                    .rounded(RADIUS_SMALL)
                    .flex_shrink_0(),
            );
        }
        selected = selected.child(
            v_flex()
                .min_w_0()
                .flex_1()
                .gap_2()
                .child(
                    accessible_text("import-selected-title", source.title.clone())
                        .text_size(TEXT_TITLE)
                        .font_weight(FontWeight::SEMIBOLD)
                        .whitespace_normal()
                        .text_ellipsis()
                        .line_clamp(2),
                )
                .when(!source.detail().is_empty(), |view| {
                    view.child(help(source.detail()))
                }),
        );
        let details = v_flex().gap_3().child(help(source.input.clone())).when(
            source.online && course2md::auth::is_bilibili_url(&source.input),
            |view| view.child(self.source_account_row(cx)),
        );
        v_flex().w_full().min_w_0().child(
            v_flex()
                .id("import-selected-source")
                .gap_3()
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
                            quiet("change-source")
                                .icon(if can_close_input {
                                    icons::chevron_up()
                                } else {
                                    icons::movie()
                                })
                                .label(if can_close_input {
                                    "收起输入"
                                } else {
                                    "更换视频"
                                })
                                .on_click(cx.listener(move |this, _, window, cx| {
                                    this.source_editor_open = !can_close_input;
                                    if !can_close_input {
                                        this.scrolls[Page::New as usize]
                                            .set_offset(point(px(0.), px(0.)));
                                        if this.online {
                                            this.inputs[&Field::Source]
                                                .update(cx, |input, cx| input.focus(window, cx));
                                        }
                                    }
                                    cx.notify();
                                })),
                        )
                        .child(
                            quiet("source-details")
                                .icon(if self.show_preview_details {
                                    IconName::ChevronUp
                                } else {
                                    IconName::ChevronDown
                                })
                                .label(if self.show_preview_details {
                                    "收起详情"
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
                                    .icon(icons::refresh())
                                    .label("重新读取")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.inspect_source(window, cx)
                                    })),
                            )
                        }),
                )
                .child(motion::disclosure(
                    "source-details-disclosure",
                    self.show_preview_details,
                    details,
                    window,
                    cx,
                )),
        )
    }

    fn text_source_view(&self, window: &mut Window, cx: &mut Context<Self>) -> Div {
        let Some(source) = &self.source_preview else {
            return v_flex();
        };
        let speech = self.import_uses_speech();
        let failure = self.subtitle_error.as_deref().or_else(|| {
            if subtitle_needs_confirmation(source, self.task_options.source_mode)
                && let SubtitleEvidence::Failed { message } = &source.subtitles
            {
                Some(message.as_str())
            } else {
                None
            }
        });
        let needs_text_choice =
            subtitle_needs_confirmation(source, self.task_options.source_mode) || failure.is_some();
        let description = if self.subtitle_loading {
            "正在读取字幕…".to_owned()
        } else if !speech
            && (source.subtitle_request.is_some() || source.subtitle_read_error.is_some())
        {
            match &source.selected_subtitle {
                Some(_) => "字幕尚未确认，已读内容仍保留".into(),
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
                SubtitleEvidence::NoneFound | SubtitleEvidence::Unsupported { .. } => {
                    "尚未选择文字来源".into()
                }
                SubtitleEvidence::Failed { .. } => "字幕尚未确认".into(),
            }
        };
        let mut view = v_flex().w_full().min_w_0().gap_3().child(
            h_flex()
                .gap_3()
                .items_center()
                .child(if self.subtitle_loading {
                    motion::spinner("subtitle-reading-spinner", cx)
                } else {
                    (if speech {
                        icons::mic()
                    } else {
                        icons::subtitles()
                    })
                    .size(px(20.))
                    .text_color(color(GRAY))
                    .into_any_element()
                })
                .child(
                    accessible_text("import-text-source-state", description)
                        .font_weight(FontWeight::MEDIUM)
                        .flex_1()
                        .min_w_0()
                        .whitespace_normal(),
                )
                .when(!needs_text_choice, |row| {
                    row.child(
                        quiet("change-text-source")
                            .icon(if self.show_options {
                                IconName::ChevronUp
                            } else {
                                IconName::ChevronDown
                            })
                            .label(if self.show_options {
                                "收起"
                            } else {
                                "更换"
                            })
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.show_options = !this.show_options;
                                cx.notify();
                            })),
                    )
                }),
        );
        if self.subtitle_loading {
            view = view.child(
                quiet("cancel-subtitle-read")
                    .icon(IconName::Close)
                    .self_start()
                    .label("取消读取字幕")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.pending_conversion = None;
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
        if let Some(error) = failure {
            let failure_details = self.subtitle_issue("subtitle-read", error, window, cx);
            view = view.child(motion::enter(
                text_id("subtitle-failure", error),
                v_flex()
                    .gap_3()
                    .p_4()
                    .rounded(RADIUS_CARD)
                    .bg(color(WARNING_BG))
                    .child(
                        h_flex()
                            .gap_2()
                            .items_start()
                            .child(
                                Icon::new(IconName::TriangleAlert)
                                    .size(px(20.))
                                    .text_color(color(WARNING))
                                    .flex_shrink_0(),
                            )
                            .child(
                                accessible_text("subtitle-failed", "字幕未读取成功")
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(color(WARNING)),
                            ),
                    )
                    .child(failure_details),
                cx,
            ));
            if let Some(old) = &source.selected_subtitle {
                view = view.child(
                    outline_pill("use-previous-subtitle")
                        .self_start()
                        .icon(icons::subtitles())
                        .label("使用已读字幕")
                        .tooltip(old.label.clone())
                        .on_click(cx.listener(|this, _, _, cx| this.use_confirmed_subtitle(cx))),
                );
            }
        }
        let show_text_choices = self.show_options || needs_text_choice;
        if speech && !show_text_choices {
            match &source.subtitles {
                SubtitleEvidence::NoneFound => view = view.child(help("未找到可直接读取的字幕")),
                SubtitleEvidence::Unsupported { message } => {
                    view = view.child(help(message.clone()))
                }
                _ => {}
            }
        }
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
                .map(|track| (format!("track:{}", track.id), track.label()))
                .collect();
            if let Some(cached) = &source.selected_subtitle
                && !tracks.iter().any(|track| track.id == cached.track_id)
            {
                choices.push((format!("track:{}", cached.track_id), cached.label.clone()));
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
                div().child(
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
            if self.subtitle_error.is_none()
                && let SubtitleEvidence::Found {
                    warning: Some(message),
                    ..
                } = &source.subtitles
            {
                options = options.child(self.subtitle_issue(
                    "subtitle-warning",
                    &format!("部分字幕尚未确认：{message}"),
                    window,
                    cx,
                ));
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
                            .icon(icons::refresh())
                            .label("重新读取字幕")
                            .disabled(self.subtitle_loading)
                            .on_click(cx.listener(|this, _, _, cx| this.retry_subtitles(cx))),
                    )
                    .when(!source.online, |view| {
                        view.child(
                            outline_pill("attach-subtitles")
                                .icon(icons::file_upload())
                                .label("选择字幕文件")
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.choose_attached_subtitle(window, cx)
                                })),
                        )
                    })
                    .when(
                        source.online
                            && course2md::auth::is_bilibili_url(&source.input)
                            && self
                                .subtitle_error
                                .as_deref()
                                .or_else(|| match &source.subtitles {
                                    SubtitleEvidence::Failed { message } => Some(message.as_str()),
                                    _ => None,
                                })
                                .is_some_and(source::is_login_failure),
                        |view| {
                            view.child(
                                outline_pill("login-for-subtitles")
                                    .icon(icons::bilibili())
                                    .label("登录 Bilibili")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.open_account_dialog(window, cx)
                                    })),
                            )
                        },
                    ),
            );
            view = view.child(motion::disclosure(
                "text-source-choices",
                show_text_choices,
                options,
                window,
                cx,
            ));
        }
        view
    }

    fn import_content_options(&self, window: &mut Window, cx: &mut Context<Self>) -> Div {
        let speech = if self.source_preview.is_some() {
            self.import_uses_speech()
        } else {
            self.task_options.source_mode != 1
        };
        let speech_options = self.import_speech_options(window, cx);
        let mut view = v_flex()
            .w_full()
            .min_w_0()
            .gap_3()
            .child(
                accessible_text("conversion-text-mode-label", "文字来源")
                    .font_weight(FontWeight::MEDIUM),
            )
            .child(
                SingleChoiceGroup::new("conversion-text-mode", "文字来源方式")
                    .options([("0", "自动选择"), ("1", "仅字幕"), ("2", "语音识别")])
                    .selected(self.task_options.source_mode.to_string())
                    .on_change(cx.listener(|this, value: &SharedString, _, cx| {
                        let Ok(mode) = value.parse::<usize>() else {
                            return;
                        };
                        if mode == 2 {
                            this.use_speech(cx);
                        } else {
                            this.task_options.source_mode = mode;
                            this.save_current_draft(cx);
                            this.advance_conversion_when_ready(cx);
                            cx.notify();
                        }
                    })),
            )
            .child(motion::disclosure(
                "speech-options",
                speech,
                speech_options,
                window,
                cx,
            ));
        let vision_options = h_flex()
            .gap_3()
            .items_start()
            .line_height(rems(1.5))
            .child(preference_icon(icons::image()))
            .child(
                crate::settings_ui::preference(
                    "发送截图辅助校对",
                    "文字及对应截图会发送到所选服务",
                    coral_switch(
                        Switch::new("import-vision")
                            .checked(self.task_options.vision)
                            .on_click(cx.listener(|this, value, _, cx| {
                                this.task_options.vision = *value;
                                if this.save_current_draft(cx) {
                                    this.advance_conversion_when_ready(cx);
                                }
                                cx.notify();
                            })),
                    ),
                )
                .flex_1()
                .min_w_0(),
            );
        let vision_disclosure = motion::disclosure(
            "ai-vision-options",
            self.task_options.llm,
            vision_options,
            window,
            cx,
        );
        let ai_overridden = {
            let defaults = ConversionOptions::from_config(&self.preferences.defaults_config());
            (
                self.task_options.llm,
                self.task_options.summarize,
                self.task_options.vision,
            ) != (defaults.llm, defaults.summarize, defaults.vision)
        };
        view = view.child(
            v_flex()
                .gap_3()
                .pt_2()
                .child(
                    h_flex()
                        .gap_3()
                        .items_start()
                        .line_height(rems(1.5))
                        .child(preference_icon(icons::auto_fix()))
                        .child(
                            crate::settings_ui::preference(
                                "AI 校对",
                                "修正识别错误和标点，保留原意",
                                coral_switch(
                                    Switch::new("import-proofread")
                                        .checked(self.task_options.llm)
                                        .on_click(cx.listener(|this, value, _, cx| {
                                            this.task_options.llm = *value;
                                            if this.save_current_draft(cx) {
                                                this.advance_conversion_when_ready(cx);
                                            }
                                            cx.notify();
                                        })),
                                ),
                            )
                            .flex_1()
                            .min_w_0(),
                        ),
                )
                .child(vision_disclosure)
                .child(
                    h_flex()
                        .gap_3()
                        .items_start()
                        .line_height(rems(1.5))
                        .child(preference_icon(icons::summarize()))
                        .child(
                            crate::settings_ui::preference(
                                "生成摘要",
                                "提炼课程要点，正文继续保留",
                                coral_switch(
                                    Switch::new("import-summary")
                                        .checked(self.task_options.summarize)
                                        .on_click(cx.listener(|this, value, _, cx| {
                                            this.task_options.summarize = *value;
                                            if this.save_current_draft(cx) {
                                                this.advance_conversion_when_ready(cx);
                                            }
                                            cx.notify();
                                        })),
                                ),
                            )
                            .flex_1()
                            .min_w_0(),
                        ),
                ),
        );
        let ai_enabled = self.task_options.llm || self.task_options.summarize;
        let mut ai_options = v_flex().gap_3();
        {
            match self.selected_task_service(ServicePurpose::Ai) {
                Some(service) => {
                    if self.task_options.llm {
                        ai_options = ai_options.child(help(format!(
                            "校对文字将发送到{}。",
                            service_destination(&service.config)
                        )));
                    }
                }
                None => {
                    ai_options = ai_options.child(
                        v_flex()
                            .gap_2()
                            .p_3()
                            .rounded(RADIUS_CARD)
                            .bg(color(WARNING_BG))
                            .child(
                                accessible_text("ai-service-missing", "AI 服务尚未设置。")
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(color(WARNING)),
                            )
                            .child(help("选择用于校对和摘要的服务")),
                    );
                }
            }
            ai_options = ai_options.child(self.task_service_picker(ServicePurpose::Ai, cx));
            if self.task_options.llm {
                let prompt = self.import_base_config().llm.prompt;
                if prompt
                    .as_ref()
                    .is_some_and(|prompt| !prompt.trim().is_empty())
                {
                    ai_options = ai_options.child(help("本次校对使用已保存的自定义规则。"));
                }
            }
        }
        view = view.child(motion::disclosure(
            "ai-service-options",
            ai_enabled,
            ai_options,
            window,
            cx,
        ));
        if ai_overridden {
            view = view.child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(badge(BadgeKind::Neutral).child("仅用于这次笔记"))
                    .child(
                        quiet("reset-task-overrides")
                            .icon(icons::refresh())
                            .label("恢复默认")
                            .on_click(
                                cx.listener(|this, _, _, cx| this.reset_task_ai_overrides(cx)),
                            ),
                    ),
            );
        }
        view
    }

    fn reset_task_ai_overrides(&mut self, cx: &mut Context<Self>) {
        if !self.save_current_draft(cx) {
            return;
        }
        let defaults = ConversionOptions::from_config(&self.preferences.defaults_config());
        let Some(workspace) = &mut self.workspace else {
            return;
        };
        if let Err(error) = workspace.transaction(|state| {
            state
                .draft_mut()
                .context("当前视频输入暂时不可用")?
                .reset_ai_overrides(&defaults);
            Ok(())
        }) {
            self.workspace_error = Some(format!("本次选项尚未恢复默认：{error:#}"));
            cx.notify();
            return;
        }
        self.task_options.llm = defaults.llm;
        self.task_options.summarize = defaults.summarize;
        self.task_options.vision = defaults.vision;
        self.advance_conversion_when_ready(cx);
        cx.notify();
    }

    fn import_speech_options(&self, window: &mut Window, cx: &mut Context<Self>) -> Div {
        let cloud = self.task_options.provider == 5;
        let mut view = v_flex()
            .gap_3()
            .p_4()
            .rounded(RADIUS_CARD)
            .bg(color(INSET))
            .child(
                SingleChoiceGroup::new("import-speech-location", "在哪里识别视频声音")
                    .options([("local", "本机识别"), ("cloud", "识别服务")])
                    .selected(if cloud { "cloud" } else { "local" })
                    .on_change(cx.listener(|this, value: &SharedString, _, cx| {
                        this.task_options.provider = if value.as_ref() == "cloud" { 5 } else { 0 };
                        if this.save_current_draft(cx) {
                            this.advance_conversion_when_ready(cx);
                        }
                        cx.notify();
                    })),
            );
        if cloud {
            return view.child(motion::enter(
                "cloud-speech-service",
                v_flex()
                    .gap_3()
                    .child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .child(icons::cloud().size(px(20.)).text_color(color(GRAY)))
                            .child(help("音频发送到所选识别服务")),
                    )
                    .child(self.task_service_picker(ServicePurpose::Speech, cx)),
                cx,
            ));
        }
        view = view
            .child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(icons::computer().size(px(20.)).text_color(color(GRAY)))
                    .child(help("音频在这台电脑上处理")),
            )
            .child(
                control("local-engine-choices")
                    .ghost()
                    .self_start()
                    .icon(if self.show_engine_details {
                        IconName::ChevronUp
                    } else {
                        IconName::ChevronDown
                    })
                    .label(if self.show_engine_details {
                        "收起识别选项"
                    } else {
                        "识别选项"
                    })
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.show_engine_details = !this.show_engine_details;
                        cx.notify();
                    })),
            );
        let engine_options = v_flex()
            .gap_2()
            .child(help(format!("当前方式：{}", self.local_engine_name())))
            .child(
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
                            if this.save_current_draft(cx) {
                                this.advance_conversion_when_ready(cx);
                            }
                            cx.notify();
                        }
                    })),
            );
        view = view.child(motion::disclosure(
            "local-engine-options",
            self.show_engine_details,
            engine_options,
            window,
            cx,
        ));
        let (provider, model, root) = self.import_model_request();
        let readiness = self.model_readiness_panel(provider, Some(&model), &root, window, cx);
        view.child(motion::enter("local-speech-readiness", readiness, cx))
    }

    fn import_uses_speech(&self) -> bool {
        self.source_preview
            .as_ref()
            .is_some_and(|source| uses_speech(source, self.task_options.source_mode))
    }

    fn missing_import_service(&self) -> Option<ServicePurpose> {
        let separate_subtitle = self.task_options.source_mode != 2
            && self
                .workspace
                .as_ref()
                .and_then(|workspace| workspace.state.draft())
                .is_some_and(|draft| draft.subtitle.is_some());
        [
            (
                ServicePurpose::Speech,
                self.task_options.provider == 5 && self.import_uses_speech() && !separate_subtitle,
            ),
            (
                ServicePurpose::Ai,
                self.task_options.llm || self.task_options.summarize,
            ),
        ]
        .into_iter()
        .find_map(|(purpose, required)| {
            (required
                && self.selected_task_service(purpose).is_none_or(|version| {
                    self.preferences
                        .service_stopped_in_snapshot(&version.service_id)
                }))
            .then_some(purpose)
        })
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
        let mut view = box_section("名称与保存").child(crate::focus_scroll::RevealFocus::new(
            ("import-title-focus", self.validation_attempt),
            self.input(Field::Title, "笔记名称", cx),
            self.scrolls[Page::New as usize].clone(),
        ));
        view = view.child(
            accessible_text("import-destination-label", "保存到").font_weight(FontWeight::MEDIUM),
        );
        if libraries.len() > 1 {
            let entity = cx.entity().downgrade();
            let label = selected
                .as_ref()
                .map(|library| library.name.clone())
                .unwrap_or_else(|| "选择保存位置".into());
            view = view.child(
                control("import-library")
                    .icon(IconName::FolderOpen)
                    .w_full()
                    .min_w_0()
                    .accessibility_label(format!("保存到：{label}"))
                    .tooltip(label.clone())
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .whitespace_nowrap()
                            .text_ellipsis()
                            .child(label),
                    )
                    .child(Icon::new(IconName::ChevronDown).size_4().flex_shrink_0())
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
                                                Ok(()) => {
                                                    this.target_folder = None;
                                                    this.advance_conversion_when_ready(cx);
                                                }
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
            view = view.child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(
                        Icon::new(IconName::FolderOpen)
                            .size(px(18.))
                            .text_color(color(GRAY)),
                    )
                    .child(help(library.name.clone()).flex_1()),
            );
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
                .child(
                    outline_pill("import-new-folder")
                        .icon(icons::create_new_folder())
                        .label("新建文件夹")
                        .on_click(
                            cx.listener(|this, _, window, cx| this.begin_folder(None, window, cx)),
                        ),
                ),
        );
        view
    }

    fn import_exports(&self, window: &mut Window, cx: &mut Context<Self>) -> Div {
        let labels = ["Markdown 包", "网页文件", "JSON 数据"];
        let selected: Vec<_> = labels
            .iter()
            .enumerate()
            .filter(|(index, _)| self.task_options.formats[*index])
            .map(|(_, label)| *label)
            .collect();
        let mut view = box_section("导出与视频").child(
            h_flex()
                .gap_3()
                .items_center()
                .flex_wrap()
                .child(
                    quiet("show-export-options")
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
                )
                .when(!selected.is_empty(), |row| {
                    row.child(help(selected.join(" · ")))
                }),
        );
        let mut options = v_flex()
            .gap_3()
            .p_4()
            .rounded(RADIUS_CARD)
            .bg(color(INSET))
            .child(help("可同时导出到其他应用使用，也可以生成后再导出。"));
        for (index, (label, description)) in [
            ("Markdown 包", "便于在笔记软件中编辑，包含文稿和图片"),
            ("网页文件", "单个文件，便于阅读与分享"),
            ("JSON 数据", "供其他程序读取和处理"),
        ]
        .into_iter()
        .enumerate()
        {
            // The icon, indicator and title share a first-line height. The
            // description lives in the title's column, independent of the
            // Checkbox component's own indicator/label spacing.
            let first_line_height = rems(32. / 14.);
            let icon = match index {
                0 => Icon::new(IconName::File),
                1 => icons::web(),
                _ => icons::code(),
            };
            options = options.child(
                h_flex()
                    .min_w_0()
                    .gap_2()
                    .items_start()
                    .child(
                        h_flex()
                            .debug_selector(move || format!("import-export-icon-{index}").into())
                            .h(first_line_height)
                            .flex_shrink_0()
                            .child(icon.size(px(20.)).text_color(color(GRAY))),
                    )
                    .child(
                        Checkbox::new(("import-export", index))
                            .debug_selector(move || {
                                format!("import-export-checkbox-{index}").into()
                            })
                            .accessibility_label(label)
                            .checked(self.task_options.formats[index])
                            .h(first_line_height)
                            .items_center()
                            .flex_shrink_0()
                            .on_click(cx.listener(move |this, value, _, cx| {
                                this.task_options.formats[index] = *value;
                                this.save_current_draft(cx);
                                cx.notify();
                            })),
                    )
                    .child(
                        v_flex()
                            .id(("import-export-text", index))
                            .gap_1()
                            .flex_1()
                            .min_w_0()
                            .cursor_pointer()
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.task_options.formats[index] =
                                    !this.task_options.formats[index];
                                this.save_current_draft(cx);
                                cx.notify();
                            }))
                            .child(
                                h_flex()
                                    .debug_selector(move || {
                                        format!("import-export-title-{index}").into()
                                    })
                                    .min_w_0()
                                    .min_h(first_line_height)
                                    .child(accessible_text(("import-export-label", index), label)),
                            )
                            .child(help(description).debug_selector(move || {
                                format!("import-export-description-{index}").into()
                            })),
                    ),
            );
        }
        view = view.child(motion::disclosure(
            "import-export-options",
            self.show_export_options,
            options,
            window,
            cx,
        ));
        if self.online {
            view = view.child(
                h_flex()
                    .gap_3()
                    .items_start()
                    .line_height(rems(1.5))
                    .child(preference_icon(icons::movie()))
                    .child(
                        crate::settings_ui::preference(
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
                        )
                        .flex_1()
                        .min_w_0(),
                    ),
            );
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

    fn can_start_input(&self, cx: &App) -> bool {
        (self.online || !self.value(Field::Source, cx).is_empty())
            && self.preview_error.is_none()
            && self.existing_source_note().is_none()
            && self.current_input_task(cx).is_none_or(|task| {
                matches!(
                    task.state,
                    workspace::TaskState::Complete | workspace::TaskState::Cancelled
                ) && !(self.reading
                    && self
                        .following_conversion
                        .as_ref()
                        .is_some_and(|follow| follow.follows(&task.id, self.preview_generation)))
            })
    }

    /// The single start command owns source preparation and conversion.
    fn box_bottom_row(&self, cx: &mut Context<Self>) -> Div {
        h_flex().items_center().child(
            primary_pill("start-conversion")
                .track_focus(&self.import_submit_focus)
                .icon(icons::arrow_forward())
                .label("开始转换")
                .disabled(self.pending_conversion.is_some())
                .on_click(cx.listener(|this, _, window, cx| this.start_conversion(window, cx))),
        )
    }

    fn generation_options_toggle(&self, cx: &mut Context<Self>) -> Div {
        h_flex().child(
            quiet("generation-options")
                .icon(if self.generation_options_open {
                    icons::chevron_up()
                } else {
                    icons::tune()
                })
                .label(if self.generation_options_open {
                    "收起高级选项"
                } else {
                    "高级选项"
                })
                .on_click(cx.listener(|this, _, _, cx| {
                    this.generation_options_open = !this.generation_options_open;
                    cx.notify();
                })),
        )
    }

    fn conversion_defaults_summary(&self) -> Div {
        let mut summary = vec![match self.task_options.source_mode {
            1 => "使用视频字幕生成笔记".to_owned(),
            2 => "识别视频声音生成笔记".to_owned(),
            _ => "优先使用字幕，字幕不可用时自动识别视频声音".to_owned(),
        }];
        if self.task_options.provider == 5
            && self.task_options.source_mode != 1
            && let Some(service) = self.selected_task_service(ServicePurpose::Speech)
        {
            summary.push(format!(
                "需要识别声音时，音频会发送到「{}」",
                service.config.name,
            ));
        }
        if (self.task_options.llm || self.task_options.summarize)
            && let Some(service) = self.selected_task_service(ServicePurpose::Ai)
        {
            summary.push(format!(
                "{}会发送到「{}」{}",
                if self.task_options.llm && self.task_options.vision {
                    "文字与截图"
                } else {
                    "文字"
                },
                service.config.name,
                match (self.task_options.llm, self.task_options.summarize) {
                    (true, true) => "进行校对并生成摘要",
                    (true, false) => "进行校对",
                    _ => "生成摘要",
                },
            ));
        }
        info_callout("conversion-defaults-summary", summary.join("\n"))
    }

    pub(super) fn current_input_task(&self, cx: &App) -> Option<&workspace::TaskRecord> {
        let state = &self.workspace.as_ref()?.state;
        submitted_input_task(state.draft()?, &state.tasks, &self.value(Field::Source, cx))
    }

    pub fn new_page(&mut self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let opening_result = self.reading && self.following_conversion.is_some();
        let linked_task = self.current_input_task(cx).cloned().filter(|task| {
            (opening_result && task.state == workspace::TaskState::Complete)
                || !task.state.finished()
                || (task.state == workspace::TaskState::Partial
                    && task.artifact.as_ref().is_some_and(|path| {
                        !crate::task_ui::task_component_failures(task, path).is_empty()
                    }))
        });
        if linked_task.is_none()
            && self.task_options.provider != 5
            && (self.import_uses_speech()
                || (self.generation_options_open && self.task_options.source_mode != 1))
        {
            let (provider, model, root) = self.import_model_request();
            self.ensure_model_diagnostic(provider, Some(&model), &root, cx);
        }
        let existing_note_decision = linked_task.is_none()
            && self.matching_current_task().is_none()
            && self.existing_source_note().is_some();
        let mut input = v_flex()
            .w_full()
            .min_w_0()
            .gap_4()
            .child(self.source_kind_tabs(cx))
            .child(self.box_source_input(window, cx));
        if existing_note_decision {
            input = input.child(self.conversion_recovery(cx));
        }
        let mut view = v_flex()
            .pt(px(24.))
            .gap_6()
            .w_full()
            .min_w_0()
            .child(
                accessible_text("workbench-title", "把视频整理成笔记")
                    .text_size(TEXT_DISPLAY)
                    .font_weight(FontWeight::SEMIBOLD),
            )
            .child(input);
        if let Some(task) = linked_task {
            let task_view = match task.state {
                workspace::TaskState::Complete if opening_result => {
                    self.box_task_opening_result(&task, cx)
                }
                workspace::TaskState::Queued
                | workspace::TaskState::Running
                | workspace::TaskState::Pausing => self.box_task_running(&task, window, cx),
                _ => self.box_task_attention(&task, cx),
            };
            view = view.child(
                task_view
                    .p_6()
                    .rounded(RADIUS_CARD)
                    .bg(color(SURFACE))
                    .border_1()
                    .border_color(color(HAIRLINE)),
            );
        } else {
            let text_required = self.subtitle_attention_required();
            if self.source_preview.is_some() && !existing_note_decision {
                let mut source = v_flex()
                    .w_full()
                    .min_w_0()
                    .gap_4()
                    .child(self.box_selected_video(window, cx));
                if text_required {
                    source = source
                        .child(box_section("文字来源").child(self.text_source_view(window, cx)));
                }
                source = source.child(self.conversion_recovery(cx));
                view = view.child(source);
            } else if text_required {
                view = view.child(box_section("文字来源").child(self.text_source_view(window, cx)));
            }
            if self.preview_cancel.is_none() && self.source_candidates.is_empty() {
                let options_open = self.generation_options_open;
                let mut content_options = box_section("笔记内容");
                if self.source_preview.is_some() && !text_required {
                    content_options = content_options.child(self.text_source_view(window, cx));
                }
                content_options = content_options.child(self.import_content_options(window, cx));
                let options = v_flex()
                    .w_full()
                    .min_w_0()
                    .gap_6()
                    .child(content_options)
                    .child(self.import_destination(cx))
                    .child(self.import_exports(window, cx));
                view = view
                    .child(
                        v_flex()
                            .w_full()
                            .min_w_0()
                            .gap_2()
                            .child(self.conversion_defaults_summary())
                            .child(self.generation_options_toggle(cx)),
                    )
                    .child(disclosure(
                        "generation-options-body",
                        options_open,
                        options,
                        window,
                        cx,
                    ));
            }
        }
        if let Some(recent) = self.recent_notes_section(cx) {
            view = view.child(recent);
        }
        if let Some(error) = &self.workspace_error {
            view = view.child(motion::enter(
                text_id("workspace-error", error),
                v_flex()
                    .p_4()
                    .rounded(RADIUS_CARD)
                    .bg(color(DANGER_BG))
                    .child(issue(error.clone())),
                cx,
            ));
        }
        view.into_any_element()
    }

    /// Only unresolved choices interrupt a submitted conversion.
    fn conversion_recovery(&mut self, cx: &mut Context<Self>) -> Div {
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
                if self.source_preview.is_some() && !reading && !self.subtitle_attention_required()
                {
                    self.submission_issue()
                } else {
                    None
                }
            });
        let mut inset = v_flex().w_full().min_w_0().gap_3();
        let mut actions = h_flex().gap_2().flex_wrap();
        if let Some(issue) = preferences_issue {
            actions = actions.child(
                outline_pill("repair-generation-preferences")
                    .icon(icons::refresh())
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
            .filter(|library| {
                self.cached_location_check(library)
                    .is_some_and(|check| check.needs_reassociation)
            })
        {
            let id = library.id.clone();
            actions = actions.child(
                outline_pill("reassociate-import-location")
                    .icon(IconName::FolderOpen)
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
                .child(conversion_fact(
                    format!("已有相同处理任务：《{}》（{location}）", task.plan.title),
                    true,
                ))
                .child(conversion_fact(
                    "查看现有任务；当前输入与选项尚未应用。",
                    false,
                ));
            actions = actions.child(
                primary_pill("show-matching-task")
                    .track_focus(&self.import_submit_focus)
                    .self_start()
                    .icon(icons::task())
                    .label("查看任务")
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.save_current_draft(cx);
                        this.select_task(&id, cx);
                        this.navigate(Page::Task, cx);
                        cx.notify();
                    })),
            );
        } else if let Some(course) = self.existing_source_note() {
            let location = self
                .course_location(&course)
                .map(|library| library.name.clone())
                .unwrap_or_else(|| "课程库".into());
            inset = inset.child(info_callout(
                "existing-note-notice",
                format!("这个视频已有笔记，保存在「{location}」。生成新版会保留原笔记。"),
            ));
            actions = actions.child(
                h_flex()
                    .gap_2()
                    .flex_wrap()
                    .child(
                        primary_pill("open-existing-note")
                            .icon(IconName::BookOpen)
                            .label("打开已有笔记")
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.save_current_draft(cx);
                                this.open_course(course.clone(), cx);
                            })),
                    )
                    .child(
                        outline_pill("generate-new-version")
                            .icon(icons::refresh())
                            .track_focus(&self.import_submit_focus)
                            .label("生成新版笔记")
                            .disabled(reading || self.workspace.is_none() || preferences_blocked)
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.following_conversion =
                                    Some(ConversionFollow::Preparing(this.preview_generation));
                                this.enqueue_current(window, cx);
                            })),
                    ),
            );
            if busy {
                inset = inset.child(conversion_fact(
                    "新版笔记会加入队列，等待当前任务完成。",
                    false,
                ));
            }
        } else if let Some(purpose) = self.missing_import_service() {
            actions = actions.child(
                primary_pill("configure-conversion-service")
                    .icon(icons::settings())
                    .label(match purpose {
                        ServicePurpose::Speech => "设置语音服务",
                        ServicePurpose::Ai => "设置 AI 服务",
                    })
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.open_task_service_editor(purpose, window, cx);
                    })),
            );
        }
        if self.pending_conversion.is_some()
            && self.source_validation.is_some()
            && !reading
            && self.missing_import_service().is_none()
            && self.existing_source_note().is_none()
            && self.matching_current_task().is_none()
        {
            actions = actions.child(
                outline_pill("retry-conversion-validation")
                    .icon(icons::refresh())
                    .label("重试转换")
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.start_conversion(window, cx);
                    })),
            );
        }
        if self.pending_conversion.is_some() && !reading {
            actions = actions.child(
                quiet("cancel-pending-conversion")
                    .icon(icons::close())
                    .label("取消转换")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.pending_conversion = None;
                        this.following_conversion = None;
                        cx.notify();
                    })),
            );
        }
        inset
            .when_some(submission_error, |view, message| {
                view.child(
                    accessible_text("import-submit-error", message)
                        .role(Role::Alert)
                        .text_sm()
                        .whitespace_normal()
                        .text_color(color(DANGER)),
                )
            })
            .child(actions)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ConversionFollow, ConversionGate, automatic_subtitle_fallback, completed_input_task,
        conversion_gate, submitted_input_task, subtitle_needs_confirmation, uses_speech,
    };
    use crate::{source, workspace};
    use course2md::subtitle::{CachedSubtitle, SubtitleEvidence, SubtitleReadError};

    #[test]
    fn one_start_continues_metadata_subtitles_and_environment_without_a_confirmation_stage() {
        // The same Start revision survives each independently completing prerequisite.
        let start = 7;
        for (reading, source, text_choice, environment) in [
            (true, false, false, false),
            (true, true, true, true),
            (false, true, false, false),
            (false, true, true, true),
        ] {
            assert_eq!(
                conversion_gate(start, 7, reading, source, text_choice, environment, false),
                ConversionGate::Wait,
            );
        }
        assert_eq!(
            conversion_gate(start, 7, false, true, false, true, false),
            ConversionGate::Submit
        );
        // Source replacement and duplicate content require a fresh intent/choice.
        assert_eq!(
            conversion_gate(start, 8, true, false, false, true, false),
            ConversionGate::Stop
        );
        assert_eq!(
            conversion_gate(start, 7, false, true, false, true, true),
            ConversionGate::Stop
        );
        assert_eq!(
            conversion_gate(start, 7, false, false, false, true, false),
            ConversionGate::Stop
        );
    }

    #[test]
    fn completion_opens_only_the_result_still_followed_in_the_current_input() {
        let (draft, mut tasks) = submitted_recovery_chain();
        let followed = ConversionFollow::Preparing(7)
            .submitted(7, "recovery".into())
            .unwrap();
        assert!(
            ConversionFollow::Preparing(7)
                .submitted(8, "recovery".into())
                .is_none()
        );
        assert!(
            followed
                .completed_task(true, 7, &draft, &tasks, &draft.input)
                .is_none()
        );
        tasks[1].state = workspace::TaskState::Complete;
        assert!(
            followed
                .completed_task(true, 7, &draft, &tasks, &draft.input)
                .is_none()
        );
        tasks[1].artifact = Some("versions/recovery".into());
        assert_eq!(
            followed
                .completed_task(true, 7, &draft, &tasks, &draft.input)
                .unwrap()
                .id,
            "recovery"
        );
        assert!(
            followed
                .completed_task(false, 7, &draft, &tasks, &draft.input)
                .is_none()
        );
        assert!(
            followed
                .completed_task(true, 8, &draft, &tasks, &draft.input)
                .is_none()
        );
        assert!(
            followed
                .completed_task(true, 7, &draft, &tasks, "new video")
                .is_none()
        );
        let other = ConversionFollow::Task {
            id: "original".into(),
            source_revision: 7,
        };
        assert!(
            other
                .completed_task(true, 7, &draft, &tasks, &draft.input)
                .is_none()
        );
        tasks[1].state = workspace::TaskState::Partial;
        assert!(
            followed
                .completed_task(true, 7, &draft, &tasks, &draft.input)
                .is_none()
        );
    }

    #[test]
    fn completion_belongs_to_the_submitted_input_and_requires_a_readable_result() {
        let root = tempfile::tempdir().unwrap();
        let mut workspace = workspace::Workspace::open_at(
            root.path().join("workspace.json"),
            root.path().join("library"),
            Default::default(),
        )
        .unwrap();
        let state = &mut workspace.state;
        let source = source::Source {
            input: "https://www.bilibili.com/video/BVfixture".into(),
            identity: "video-fixture".into(),
            online: true,
            ..Default::default()
        };
        let plan = workspace::TaskPlan {
            operation: Default::default(),
            source: source.clone(),
            source_id: source.identity.clone(),
            title: "已提交的标题".into(),
            library_id: state.default_library.clone(),
            folder: None,
            options: Default::default(),
            subtitle: None,
            config: Default::default(),
            asr_service: None,
            ai_service: None,
        };
        let (id, _) = state.enqueue(plan.clone(), None).unwrap();
        let draft = state.draft_mut().unwrap();
        draft.change_source(source.input.clone());
        draft.submitted_task = Some(id.clone());
        assert!(
            completed_input_task(state.draft().unwrap(), &state.tasks, &source.input).is_none()
        );
        state.task_mut(&id).unwrap().state = workspace::TaskState::Complete;
        assert!(
            completed_input_task(state.draft().unwrap(), &state.tasks, &source.input).is_none()
        );
        state.task_mut(&id).unwrap().artifact = Some(root.path().join("published-version"));
        assert_eq!(
            completed_input_task(state.draft().unwrap(), &state.tasks, &source.input)
                .unwrap()
                .id,
            id,
        );
        // The text can change before its observer replaces the submitted draft.
        // That next input must remain visible even during this intermediate state.
        for next in ["", "https://www.bilibili.com/video/BVnext"] {
            assert!(completed_input_task(state.draft().unwrap(), &state.tasks, next).is_none());
        }
        state.task_mut(&id).unwrap().state = workspace::TaskState::Partial;
        assert!(
            completed_input_task(state.draft().unwrap(), &state.tasks, &source.input).is_none()
        );
        state.task_mut(&id).unwrap().state = workspace::TaskState::Complete;
        state.prepare_next_import("https://www.bilibili.com/video/BVnext", Default::default());
        assert!(
            completed_input_task(state.draft().unwrap(), &state.tasks, &source.input).is_none()
        );
        assert!(state.task(&id).unwrap().plan == plan);
    }

    fn submitted_recovery_chain() -> (workspace::Draft, Vec<workspace::TaskRecord>) {
        let source = source::Source {
            input: "https://www.bilibili.com/video/BVfixture".into(),
            identity: "video-fixture".into(),
            online: true,
            ..Default::default()
        };
        let mut draft = workspace::Draft::new(true, "library".into(), Default::default());
        draft.change_source(source.input.clone());
        draft.source = Some(source.clone());
        draft.submitted_task = Some("original".into());
        let mut original = workspace::TaskRecord {
            id: "original".into(),
            plan: workspace::TaskPlan {
                operation: Default::default(),
                source_id: source.identity.clone(),
                source,
                title: "已提交的课程".into(),
                library_id: "library".into(),
                folder: None,
                options: Default::default(),
                subtitle: None,
                config: Default::default(),
                asr_service: None,
                ai_service: None,
            },
            state: workspace::TaskState::Uncertain,
            intent: workspace::Intent::Pause,
            created: 0,
            updated: 0,
            parent: None,
            handled_by: None,
            work_dir: "work/original".into(),
            stages: Default::default(),
            error: Some("原摘要请求结果未确认".into()),
            artifact: Some("versions/original".into()),
            outcomes: None,
            unread: false,
            logs: Vec::new(),
            blocked: Vec::new(),
            resend: Vec::new(),
        };
        let mut recovery = original.clone();
        recovery.id = "recovery".into();
        recovery.parent = Some(original.id.clone());
        recovery.state = workspace::TaskState::Queued;
        recovery.intent = workspace::Intent::Run;
        recovery.error = None;
        recovery.artifact = None;
        recovery.work_dir = "work/recovery".into();
        recovery.plan.operation = course2md::execution::Operation::Reprocess {
            base_version_dir: "versions/original".into(),
            components: vec!["summary".into()],
            prior_work_dir: Some(original.work_dir.clone()),
        };
        original.handled_by = Some(recovery.id.clone());
        (draft, vec![original, recovery])
    }

    #[test]
    fn workbench_follows_the_same_recovery_for_queued_running_and_completed_results() {
        let (draft, mut tasks) = submitted_recovery_chain();
        let original = tasks[0].clone();
        for state in [workspace::TaskState::Queued, workspace::TaskState::Running] {
            tasks[1].state = state;
            assert_eq!(
                submitted_input_task(&draft, &tasks, &draft.input)
                    .unwrap()
                    .id,
                "recovery"
            );
            assert!(completed_input_task(&draft, &tasks, &draft.input).is_none());
        }
        tasks[1].state = workspace::TaskState::Complete;
        assert!(completed_input_task(&draft, &tasks, &draft.input).is_none());
        tasks[1].artifact = Some("versions/recovery".into());
        assert_eq!(
            completed_input_task(&draft, &tasks, &draft.input)
                .unwrap()
                .id,
            "recovery"
        );

        let mut final_attempt = tasks[1].clone();
        final_attempt.id = "final-attempt".into();
        final_attempt.parent = Some("recovery".into());
        final_attempt.state = workspace::TaskState::Running;
        final_attempt.artifact = None;
        tasks[1].state = workspace::TaskState::Uncertain;
        tasks[1].handled_by = Some(final_attempt.id.clone());
        tasks.push(final_attempt);
        assert_eq!(
            submitted_input_task(&draft, &tasks, &draft.input)
                .unwrap()
                .id,
            "final-attempt"
        );
        assert!(completed_input_task(&draft, &tasks, &draft.input).is_none());
        tasks[2].state = workspace::TaskState::Complete;
        tasks[2].artifact = Some("versions/final-attempt".into());
        assert_eq!(
            completed_input_task(&draft, &tasks, &draft.input)
                .unwrap()
                .id,
            "final-attempt"
        );
        assert!(tasks[0] == original);
        assert_eq!(draft.submitted_task.as_deref(), Some("original"));
    }

    #[test]
    fn recovery_lookup_rejects_broken_cycles_or_unrelated_task_links() {
        for case in [
            "missing",
            "self-cycle",
            "cycle",
            "wrong-parent",
            "other-source",
            "inconsistent-source",
            "other-kind",
            "duplicate-id",
            "unrelated-submission",
        ] {
            let (mut draft, mut tasks) = submitted_recovery_chain();
            match case {
                "missing" => tasks[0].handled_by = Some("missing".into()),
                "self-cycle" => {
                    tasks[0].handled_by = Some("original".into());
                    tasks[0].parent = Some("original".into());
                }
                "cycle" => {
                    tasks[1].handled_by = Some("original".into());
                    tasks[0].parent = Some("recovery".into());
                }
                "wrong-parent" => tasks[1].parent = Some("unrelated".into()),
                "other-source" => {
                    tasks[1].plan.source_id = "other-video".into();
                    tasks[1].plan.source.identity = "other-video".into();
                }
                "inconsistent-source" => tasks[1].plan.source.identity = "other-video".into(),
                "other-kind" => tasks[1].plan.source.online = false,
                "duplicate-id" => tasks.push(tasks[1].clone()),
                "unrelated-submission" => {
                    tasks[1].plan.source_id = "other-video".into();
                    tasks[1].plan.source.identity = "other-video".into();
                    draft.submitted_task = Some("recovery".into());
                }
                _ => unreachable!(),
            }
            assert!(
                submitted_input_task(&draft, &tasks, &draft.input).is_none(),
                "{case}"
            );
            assert!(
                completed_input_task(&draft, &tasks, &draft.input).is_none(),
                "{case}"
            );
        }
    }

    #[test]
    fn a_recovered_result_cannot_replace_new_or_changed_input() {
        let (mut draft, mut tasks) = submitted_recovery_chain();
        tasks[1].state = workspace::TaskState::Complete;
        tasks[1].artifact = Some("versions/recovery".into());
        for input in ["", "https://www.bilibili.com/video/BVnext"] {
            assert!(submitted_input_task(&draft, &tasks, input).is_none());
            assert!(completed_input_task(&draft, &tasks, input).is_none());
        }
        draft.source.as_mut().unwrap().identity = "newly-inspected-content".into();
        assert!(submitted_input_task(&draft, &tasks, &draft.input).is_none());
        draft.source = None;
        draft.input = "https://www.bilibili.com/video/BVnext".into();
        assert!(submitted_input_task(&draft, &tasks, &draft.input).is_none());
    }

    #[test]
    fn automatic_mode_uses_speech_after_login_or_download_failure() {
        let track = course2md::subtitle::file_track("lesson.srt".into(), None);
        for metadata_failed in [true, false] {
            let mut source = source::Source {
                subtitles: if metadata_failed {
                    SubtitleEvidence::Failed {
                        message: "WARNING: subtitles require login".into(),
                    }
                } else {
                    SubtitleEvidence::Found {
                        tracks: vec![track.clone()],
                        warning: None,
                    }
                },
                subtitle_request: (!metadata_failed).then(|| track.clone()),
                subtitle_read_error: (!metadata_failed).then(|| SubtitleReadError::Failed {
                    message: "字幕文件没有下载成功".into(),
                }),
                ..Default::default()
            };
            assert!(automatic_subtitle_fallback(&mut source, 0, None));
            assert!(uses_speech(&source, 0));
            assert!(!subtitle_needs_confirmation(&source, 0));
            assert!(source.subtitle_request.is_none());
            assert!(source.subtitle_read_error.is_none());
        }
    }

    #[test]
    fn explicit_subtitle_and_cancelled_reads_never_fall_back_to_speech() {
        let mut source = source::Source {
            subtitles: SubtitleEvidence::Failed {
                message: "字幕需要登录".into(),
            },
            subtitle_read_error: Some(SubtitleReadError::Failed {
                message: "字幕需要登录".into(),
            }),
            ..Default::default()
        };
        let original = source.clone();
        assert!(!automatic_subtitle_fallback(&mut source, 1, None));
        assert_eq!(source, original);
        assert!(subtitle_needs_confirmation(&source, 1));

        source.subtitle_read_error = Some(SubtitleReadError::Cancelled);
        let cancelled = source.clone();
        assert!(!automatic_subtitle_fallback(&mut source, 0, None));
        assert_eq!(source, cancelled);
        assert!(!uses_speech(&source, 0));
    }

    #[test]
    fn pending_or_failed_subtitles_remain_required_even_with_cached_text() {
        let track = course2md::subtitle::file_track("lesson.srt".into(), None);
        let mut source = source::Source {
            identity: "lesson".into(),
            subtitles: SubtitleEvidence::Found {
                tracks: vec![track.clone()],
                warning: None,
            },
            ..Default::default()
        };
        assert!(subtitle_needs_confirmation(&source, 0));
        source.selected_subtitle = Some(CachedSubtitle {
            source_identity: source.identity.clone(),
            track_id: track.id.clone(),
            label: "已读取字幕".into(),
            path: "lesson.srt".into(),
            events: Vec::new(),
        });
        assert!(!subtitle_needs_confirmation(&source, 0));

        source.subtitle_request = Some(track);
        assert!(subtitle_needs_confirmation(&source, 0));
        source.subtitle_request = None;
        source.subtitle_read_error = Some(SubtitleReadError::Failed {
            message: "无法读取新字幕，原正文仍保留".into(),
        });
        assert!(subtitle_needs_confirmation(&source, 0));

        // Cancelling the replacement or choosing the cached text restores the
        // confirmed source without opening unrelated generation preferences.
        source.subtitle_read_error = None;
        assert!(!subtitle_needs_confirmation(&source, 1));
        source.selected_subtitle = None;
        assert!(subtitle_needs_confirmation(&source, 1));
        assert!(!subtitle_needs_confirmation(&source, 2));
    }

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
        source.subtitles = SubtitleEvidence::Unchecked;
        assert!(!uses_speech(&source, 0));
        source.subtitles = SubtitleEvidence::Failed {
            message: "请求失败".into(),
        };
        assert!(uses_speech(&source, 0));
        assert!(!uses_speech(&source, 1));
    }
}
