#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
mod a11y;
mod about;
mod account_ui;
mod activity;
mod appearance_ui;
mod backend;
mod course_library;
mod credentials;
mod focus_scroll;
// Icon helpers ship ahead of the pages that reference them (M3+); same staged
// token allowance as the theme module.
#[allow(dead_code)]
mod icons;
mod import_ui;
mod legacy_settings;
mod library_ui;
mod motion;
mod notes;
mod organize;
mod palettes;
mod preferences;
mod reader_navigation;
mod reader_ui;
mod service_test;
mod settings_ui;
mod source;
mod storage;
mod storage_ui;
mod task_ui;
// Token items ship ahead of the page milestones that adopt them (M3+); the
// module-wide allowance keeps staged tokens from tripping the zero-warning bar.
#[allow(dead_code)]
mod theme;
mod views;
mod workspace;
use backend::{Completed, Course, Event, Job};
use gpui::{prelude::*, *};
use gpui_component::{
    input::{Input, InputEvent, InputState},
    *,
};
use std::{
    collections::{BTreeMap, VecDeque},
    path::PathBuf,
    time::{Duration, Instant},
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Page {
    New,
    Task,
    Library,
    Settings,
    Result,
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Convert,
    Doctor,
    Models,
}
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Field {
    Source,
    Title,
    FolderName,
    Output,
    Search,
    AsrUrl,
    AsrKey,
    AsrModel,
    LlmUrl,
    LlmKey,
    LlmModel,
}
const PROVIDERS: [(&str, &str); 6] = [
    ("", "自动"),
    ("coreml", "Apple 原生"),
    ("gpu", "GPU"),
    ("cpu", "CPU"),
    ("npu", "Intel NPU"),
    ("api", "云端 API"),
];

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
struct ConversionOptions {
    provider: usize,
    source_mode: usize,
    llm: bool,
    #[serde(default)]
    summarize: bool,
    #[serde(default)]
    vision: bool,
    keep_video: bool,
    resume: bool,
    formats: [bool; 3],
}

impl ConversionOptions {
    fn from_config(config: &course2md::settings::ConfigFile) -> Self {
        let provider = config
            .defaults
            .provider
            .map(|p| {
                PROVIDERS
                    .iter()
                    .position(|(id, _)| *id == p.as_str())
                    .unwrap_or(0)
            })
            .unwrap_or(0);
        let source_mode = match config.defaults.transcript_source.unwrap_or_default() {
            course2md::config::TranscriptSource::Auto => 0,
            course2md::config::TranscriptSource::Subtitle => 1,
            course2md::config::TranscriptSource::Asr => 2,
        };
        let llm = config.llm.enabled;
        let keep_video = config.defaults.keep_video.unwrap_or(false);
        let resume = true;
        let formats = config
            .defaults
            .formats
            .as_ref()
            .map(|formats| {
                use course2md::config::OutputFormat::*;
                [
                    formats.contains(&Md),
                    formats.contains(&Html),
                    formats.contains(&Json),
                ]
            })
            .unwrap_or([true, true, false]);
        Self {
            provider,
            source_mode,
            llm,
            summarize: config.llm.summarize,
            vision: config.llm.vision,
            keep_video,
            resume,
            formats,
        }
    }
}

impl Default for ConversionOptions {
    fn default() -> Self {
        let mut value = Self::from_config(&Default::default());
        value.formats = [false; 3];
        value
    }
}

struct Desktop {
    preferences: preferences::Store,
    settings_ui: settings_ui::State,
    storage_ui: storage_ui::State,
    reader_ui: reader_ui::State,
    workspace: Option<workspace::Workspace>,
    workspace_error: Option<String>,
    active_task: Option<String>,
    draft_loading: bool,
    draft_deadline: Option<Instant>,
    source_deadline: Option<Instant>,
    quit_deadline: Option<Instant>,
    account: account_ui::AccountUi,
    online: bool,
    last_source_input: String,
    completed_source: Option<String>,
    source_preview: Option<source::Source>,
    source_candidates: Vec<source::SourceCandidate>,
    source_collection_title: Option<String>,
    subtitle_loading: bool,
    subtitle_cancel: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,
    subtitle_error: Option<String>,
    subtitle_generation: u64,
    preview_cancel: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,
    preview_generation: u64,
    preview_workers: usize,
    preview_error: Option<String>,
    source_validation: Option<String>,
    import_submit_focus: FocusHandle,
    root_focus: FocusHandle,
    show_preview_details: bool,
    expanded_subtitle_issue: Option<SharedString>,
    library: organize::Library,
    library_root: PathBuf,
    library_error: Option<String>,
    library_issues: Vec<String>,
    library_materials: Vec<PathBuf>,
    library_indexes: BTreeMap<PathBuf, organize::Library>,
    library_generation: u64,
    folder_filter: Option<u64>, // None = all; 0 = unfiled
    target_folder: Option<u64>,
    folder_editor: Option<Option<u64>>,
    folder_origin: Option<library_ui::FolderOrigin>,
    folder_error: Option<String>,
    delete_folder: Option<u64>,
    page: Page,
    result_origin: Page,
    settings_origin: Option<Page>,
    settings_return_focus: Option<FocusHandle>,
    result_tab: usize,
    settings_tab: usize,
    show_options: bool,
    show_export_options: bool,
    show_logs: bool,
    show_engine_details: bool,
    environment: Option<backend::Environment>,
    scrolls: [ScrollHandle; 5],
    inputs: BTreeMap<Field, Entity<InputState>>,
    config: course2md::settings::ConfigFile,
    config_error: bool,
    task_options: ConversionOptions,
    settings_options: ConversionOptions,
    job: Option<Job>,
    kind: Kind,
    cancelling: bool,
    closing: bool,
    task_status: String,
    task_error: Option<String>,
    progress: BTreeMap<String, activity::Activity>,
    settings_snapshot: course2md::settings::ConfigFile,
    settings_deadline: Option<Instant>,
    settings_status: String,
    desktop_settings: course2md::settings::DesktopSettings,
    last_tick: Instant,
    logs: VecDeque<String>,
    pending_done: Option<Completed>,
    completed: Option<Completed>,
    courses: Vec<Course>,
    loading: bool,
    preview: Option<backend::Preview>,
    read_generation: u64,
    reading: bool,
    reader_scroll: ScrollHandle,
    reader_saved_offset: f32,
    exporting: bool,
    message: Option<String>,
    _subscriptions: Vec<Subscription>,
    _poll: Task<()>,
}

actions!(
    course2md_desktop,
    [Quit, OpenSettings, OpenAbout, NewNote, SearchContent]
);

impl Desktop {
    fn request_close(&mut self, cx: &mut Context<Self>) -> bool {
        if !self.flush_settings_for_exit(cx) || !self.save_current_draft(cx) {
            return false;
        }
        self.cancel_storage_for_close();
        if let Some(workspace) = &mut self.workspace {
            if let Err(error) = workspace.transaction(|state| {
                state.stop_session();
                Ok(())
            }) {
                self.workspace_error = Some(format!("退出意图尚未保存：{error:#}。窗口保持打开。"));
                cx.notify();
                return false;
            }
        }
        if let Some(cancel) = &self.preview_cancel {
            cancel.store(true, std::sync::atomic::Ordering::Relaxed);
        }
        if let Some(cancel) = &self.subtitle_cancel {
            cancel.store(true, std::sync::atomic::Ordering::Relaxed);
        }
        self.closing = true;
        self.quit_deadline = Some(Instant::now() + Duration::from_secs(10));
        self.refresh_dispatch_controls(cx);
        if self.active_task.is_none()
            && let Some(job) = &self.job
        {
            job.cancel();
        }
        self.message = Some("正在保存进度并退出…".into());
        cx.notify();
        self.job.is_none() && self.preview_workers == 0
    }
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let configuration_directory = course2md::config::config_dir();
        let legacy = legacy_settings::Inspection::inspect(course2md::settings::config_path());
        let config_error = legacy.problem.is_some();
        let message = None;
        let output = legacy_settings::startup_output(
            &configuration_directory,
            &legacy,
            backend::default_output,
        );
        let preferences = preferences::Store::open(
            configuration_directory.join("desktop-preferences"),
            credentials::system_vault(),
        );
        let mut config = preferences.defaults_config();
        config.defaults.out = Some(output.clone());
        let fields = [
            (
                Field::Source,
                "粘贴 YouTube 或 Bilibili 视频链接",
                String::new(),
            ),
            (Field::Title, "", String::new()),
            (
                Field::Output,
                "课程笔记保存位置",
                output.display().to_string(),
            ),
            (Field::Search, "搜索笔记标题", String::new()),
            (Field::FolderName, "文件夹名称", String::new()),
            (Field::AsrUrl, "", config.asr_api.base_url.clone()),
            (Field::AsrKey, "API Key", config.asr_api.api_key.clone()),
            (Field::AsrModel, "转写模型", config.asr_api.model.clone()),
            (Field::LlmUrl, "", config.llm.base_url.clone()),
            (Field::LlmKey, "API Key", config.llm.api_key.clone()),
            (Field::LlmModel, "模型名称", config.llm.model.clone()),
        ];
        let inputs: BTreeMap<_, _> = fields
            .into_iter()
            .map(|(field, placeholder, value)| {
                (
                    field,
                    cx.new(|cx| {
                        InputState::new(window, cx)
                            .placeholder(placeholder)
                            .default_value(value)
                            .masked(matches!(field, Field::AsrKey | Field::LlmKey))
                    }),
                )
            })
            .collect();
        let mut subscriptions: Vec<Subscription> = inputs
            .iter()
            .map(|(field, input)| {
                let field = *field;
                cx.observe(input, move |this: &mut Self, _, cx| {
                    if this.draft_loading {
                        return;
                    }
                    if field == Field::Source {
                        let value = this.value(Field::Source, cx);
                        if value != this.last_source_input {
                            let pasted =
                                value.len().saturating_sub(this.last_source_input.len()) > 2;
                            this.last_source_input = value;
                            this.invalidate_source();
                            this.source_deadline =
                                (pasted && this.online && !this.last_source_input.is_empty())
                                    .then(|| Instant::now() + Duration::from_millis(500));
                        }
                    }
                    if matches!(field, Field::Source | Field::Title) {
                        this.draft_deadline = Some(Instant::now() + Duration::from_millis(350));
                    }
                    if field == Field::Search {
                        this.scrolls[Page::Library as usize].set_offset(point(px(0.), px(0.)));
                    }
                    cx.notify();
                })
            })
            .collect();
        subscriptions.push(cx.subscribe(
            &inputs[&Field::Source],
            |this: &mut Self, _, event, cx| {
                if matches!(event, InputEvent::PressEnter { .. })
                    && this.page == Page::New
                    && this.online
                    && this.preview_cancel.is_none()
                {
                    this.inspect_source(cx);
                }
            },
        ));
        subscriptions.push(cx.subscribe(
            &inputs[&Field::FolderName],
            |this: &mut Self, _, event, cx| {
                if matches!(event, InputEvent::Change) {
                    this.folder_error = None;
                    cx.notify();
                }
                if matches!(event, InputEvent::PressEnter { .. }) {
                    this.save_folder(cx);
                }
            },
        ));
        let options = ConversionOptions::from_config(&config);
        let poll = cx.spawn_in(window, async |this, cx| {
            let mut busy = false;
            loop {
                smol::Timer::after(Duration::from_millis(if busy { 100 } else { 500 })).await;
                match this.update_in(cx, |this, _, cx| {
                    this.poll(cx);
                    if this
                        .settings_deadline
                        .is_some_and(|deadline| Instant::now() >= deadline)
                    {
                        this.settings_deadline = None;
                        this.save_settings(cx);
                    }
                    this.job.is_some() || this.settings_deadline.is_some()
                }) {
                    Ok(active) => busy = active,
                    Err(_) => break,
                }
            }
        });
        let (workspace, workspace_error) =
            match workspace::Workspace::open(output.clone(), options.clone()) {
                Ok(workspace) => (Some(workspace), None),
                Err(error) => (
                    None,
                    Some(format!("草稿与任务记录无法读取，原文件已保留：{error:#}")),
                ),
            };
        let settings_ui = settings_ui::State::new(window, cx);
        let reader_ui = reader_ui::State::new(window, cx);
        let mut this = Self {
            preferences,
            settings_ui,
            reader_ui,
            storage_ui: Default::default(),
            workspace,
            workspace_error,
            active_task: None,
            draft_loading: false,
            draft_deadline: None,
            source_deadline: None,
            quit_deadline: None,
            page: Page::Library,
            online: true,
            last_source_input: String::new(),
            completed_source: None,
            source_preview: None,
            source_candidates: Vec::new(),
            source_collection_title: None,
            subtitle_loading: false,
            subtitle_cancel: None,
            subtitle_error: None,
            subtitle_generation: 0,
            preview_cancel: None,
            preview_generation: 0,
            preview_workers: 0,
            preview_error: None,
            source_validation: None,
            import_submit_focus: cx.focus_handle(),
            root_focus: Self::install_root_focus(window, cx),
            show_preview_details: false,
            expanded_subtitle_issue: None,
            account: account_ui::AccountUi::default(),
            library: Default::default(),
            library_root: output,
            library_error: None,
            library_issues: Vec::new(),
            library_materials: Vec::new(),
            library_indexes: BTreeMap::new(),
            library_generation: 0,
            folder_filter: None,
            target_folder: None,
            folder_editor: None,
            folder_origin: None,
            folder_error: None,
            delete_folder: None,
            result_origin: Page::Library,
            settings_origin: None,
            settings_return_focus: None,
            result_tab: 0,
            settings_tab: 4,
            show_options: false,
            show_export_options: false,
            show_logs: false,
            show_engine_details: false,
            environment: None,
            scrolls: std::array::from_fn(|_| ScrollHandle::new()),
            inputs,
            settings_snapshot: config.clone(),
            desktop_settings: config.desktop.clone(),
            settings_deadline: None,
            settings_status: String::new(),
            last_tick: Instant::now(),
            config,
            config_error,
            task_options: options.clone(),
            settings_options: options,
            job: None,
            kind: Kind::Convert,
            cancelling: false,
            closing: false,
            task_error: None,
            task_status: String::new(),
            progress: BTreeMap::new(),
            logs: VecDeque::new(),
            pending_done: None,
            completed: None,
            courses: vec![],
            loading: false,
            preview: None,
            read_generation: 0,
            reading: false,
            reader_scroll: ScrollHandle::new(),
            reader_saved_offset: f32::NAN,
            exporting: false,
            message,
            _subscriptions: subscriptions,
            _poll: poll,
        };
        this.settings_snapshot = this.edited_settings(cx);
        this.restore_draft(window, cx);
        this.restore_storage_state(cx);
        cx.set_reduce_motion(this.desktop_settings.reduce_motion);
        this.refresh_account(cx);
        this.refresh_environment(cx);
        this.refresh_library(cx);
        this
    }
    fn refresh_environment(&mut self, cx: &mut Context<Self>) {
        self.environment = None;
        let task = cx
            .background_executor()
            .spawn(async { backend::Environment::detect() });
        cx.spawn(async move |this, cx| {
            let environment = task.await;
            let _ = this.update(cx, |this, cx| {
                this.environment = Some(environment);
                this.refresh_model_diagnostics(cx);
                cx.notify();
            });
        })
        .detach();
    }
    fn install_root_focus(window: &mut Window, cx: &mut Context<Self>) -> FocusHandle {
        let root = cx.focus_handle().tab_stop(false);
        // This is a real dispatch ancestor, not another stop in the control order.
        if window.focused(cx).is_none() {
            root.focus(window, cx);
        }
        cx.on_focus_lost(window, |this, window, cx| {
            let ancestor = window.focus_lost_restore_target(cx);
            if window.has_active_dialog(cx) {
                // A dialog can replace its own controls. Restore only inside its
                // active trap; never send focus back to the obscured main page.
                if let Some(trap) = gpui_base::active_focus_trap(window, cx) {
                    let target = ancestor
                        .filter(|focus| trap.contains(focus, window))
                        .unwrap_or(trap);
                    target.focus(window, cx);
                }
            } else {
                ancestor
                    .unwrap_or_else(|| this.root_focus.clone())
                    .focus(window, cx);
            }
        })
        .detach();
        root
    }

    fn new_note_from_action(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let previous = self
            .workspace
            .as_ref()
            .map(|workspace| workspace.state.current_draft.clone());
        self.new_draft(true, false, window, cx);
        if self.page == Page::New
            && self
                .workspace
                .as_ref()
                .is_some_and(|workspace| Some(&workspace.state.current_draft) != previous.as_ref())
        {
            self.inputs[&Field::Source].update(cx, |input, cx| input.focus(window, cx));
        }
    }

    fn navigate(&mut self, page: Page, cx: &mut Context<Self>) {
        self.save_reading_position(cx);
        self.read_generation = self.read_generation.wrapping_add(1);
        self.reading = false;
        if self.page == Page::New {
            self.save_current_draft(cx);
        }
        if self.page != page {
            self.show_engine_details = false;
        }
        self.page = page;
        self.message = None;
        if page == Page::Library {
            self.refresh_library(cx);
        }
        if page == Page::Task {
            let visible_unread = self.workspace.as_ref().and_then(|workspace| {
                let id = workspace.state.selected_task.as_ref()?;
                workspace
                    .state
                    .task(id)
                    .filter(|task| task.unread)
                    .map(|_| id.clone())
            });
            if let Some(id) = visible_unread {
                self.select_task(&id, cx);
            }
        }
        cx.notify();
    }
    fn value(&self, field: Field, cx: &App) -> String {
        self.inputs[&field].read(cx).value().trim().to_string()
    }
    fn field_error(&self, field: Field, cx: &App) -> Option<&'static str> {
        self.settings_status
            .starts_with("未保存：")
            .then(|| self.invalid_setting(cx))
            .flatten()
            .filter(|(invalid, _)| *invalid == field)
            .map(|(_, message)| message)
    }
    fn input(&self, field: Field, label: &'static str, cx: &App) -> Div {
        let error = self.field_error(field, cx);
        v_flex()
            .gap_2()
            .w_full()
            .child(
                theme::accessible_text(("field-label", field as usize), label)
                    .text_sm()
                    .font_weight(FontWeight::MEDIUM),
            )
            .child(
                Input::new(&self.inputs[&field])
                    .min_h(rems(2.6))
                    .aria_label(label)
                    .when(error.is_some(), |input| {
                        input.border_color(theme::color(theme::DANGER))
                    }),
            )
            .when_some(error, |v, message| {
                v.child(
                    div()
                        .text_sm()
                        .text_color(theme::color(theme::DANGER))
                        .child(message),
                )
            })
    }
    fn output(&self, _cx: &App) -> PathBuf {
        self.workspace
            .as_ref()
            .and_then(|w| w.state.library(&w.state.default_library))
            .map(|l| l.root.clone())
            .unwrap_or_else(|| self.library_root.clone())
    }
    fn pick(&mut self, directory: bool, window: &mut Window, cx: &mut Context<Self>) {
        let prompt = cx.prompt_for_paths(PathPromptOptions {
            files: !directory,
            directories: directory,
            multiple: false,
            prompt: Some(
                if directory {
                    "选择保存目录"
                } else {
                    "选择课程视频"
                }
                .into(),
            ),
        });
        cx.spawn_in(window, async move |this, cx| {
            let result = prompt.await;
            let _ = this.update_in(cx, |this, window, cx| {
                match result {
                    Ok(Ok(Some(paths))) => {
                        if let Some(path) = paths.first() {
                            let field = if directory {
                                Field::Output
                            } else {
                                Field::Source
                            };
                            this.inputs[&field].update(cx, |state, cx| {
                                state.set_value(path.display().to_string(), window, cx)
                            });
                            if !directory {
                                this.inspect_source(cx);
                            }
                        }
                    }
                    Ok(Ok(None)) => {}
                    Ok(Err(error)) => this.message = Some(format!("无法打开文件选择器：{error:#}")),
                    Err(error) => this.message = Some(error.to_string()),
                }
                cx.notify();
            });
        })
        .detach();
    }
    fn start(&mut self, kind: Kind, cx: &mut Context<Self>) {
        if kind == Kind::Convert {
            self.navigate(Page::New, cx);
            return;
        }
        if self.job.is_some() {
            self.message = Some("当前任务正在处理，结束后可以进行这项操作。".into());
            cx.notify();
            return;
        }
        let args = match kind {
            Kind::Doctor => vec!["doctor".into()],
            Kind::Models => self.model_preparation_args(),
            Kind::Convert => unreachable!(),
        };
        match Job::start(args) {
            Ok(job) => {
                self.job = Some(job);
                self.active_task = None;
                self.kind = kind;
                self.cancelling = false;
                self.logs.clear();
                self.progress.clear();
                self.pending_done = None;
                self.task_error = None;
                self.task_status = if kind == Kind::Doctor {
                    "正在检查运行环境"
                } else {
                    "正在准备识别模型"
                }
                .into();
            }
            Err(error) => {
                self.task_error = Some(format!("{error:#}"));
                if kind == Kind::Models {
                    self.model_preparation_finished(false, false, cx);
                } else {
                    self.message = self.task_error.clone();
                }
            }
        }
        cx.notify();
    }
    fn poll(&mut self, cx: &mut Context<Self>) {
        self.poll_storage(cx);
        self.save_reading_position(cx);
        if self
            .draft_deadline
            .is_some_and(|deadline| Instant::now() >= deadline)
        {
            self.save_current_draft(cx);
        }
        if self
            .source_deadline
            .is_some_and(|deadline| Instant::now() >= deadline)
        {
            self.source_deadline = None;
            if self.page == Page::New
                && self.online
                && source::validate_url(&self.value(Field::Source, cx)).is_ok()
            {
                self.inspect_source(cx);
            }
        }
        let events: Vec<_> = self
            .job
            .as_ref()
            .map(|job| job.events.try_iter().take(512).collect())
            .unwrap_or_default();
        let mut save = false;
        for event in events {
            if self.active_task.is_some() {
                self.record_task_event(&event);
                save = true;
            }
            match event {
                Event::Log { message } => self.logs.push_back(message),
                Event::Stage { stage, status } => {
                    if status == "start" {
                        self.progress.insert(stage, activity::Activity::new());
                    } else if status == "done" {
                        self.progress
                            .entry(stage)
                            .or_insert_with(activity::Activity::new)
                            .done = true;
                    }
                }
                Event::Progress {
                    stage,
                    current,
                    total,
                    message,
                } => self
                    .progress
                    .entry(stage)
                    .or_insert_with(activity::Activity::new)
                    .update(current, total, message),
                Event::Workers { stage, workers } => {
                    self.progress
                        .entry(stage)
                        .or_insert_with(activity::Activity::new)
                        .workers = workers
                }
                Event::Error { message } => {
                    self.task_error = Some(message.clone());
                    self.logs.push_back(message);
                }
                Event::Blocked { message, .. } => {
                    self.task_error = Some(message);
                }
                Event::Done(done) => self.pending_done = Some(done),
                Event::Exit { success, cancelled } => {
                    self.job = None;
                    self.cancelling = false;
                    if let Some(id) = self.active_task.take() {
                        self.finish_task(&id, success, cancelled, cx);
                        self.refresh_model_diagnostics(cx);
                    } else {
                        self.task_status = if success {
                            "已完成"
                        } else if cancelled {
                            "已停止"
                        } else {
                            "未完成"
                        }
                        .into();
                        if self.kind == Kind::Models {
                            self.model_preparation_finished(success, cancelled, cx);
                        } else if success {
                            self.refresh_environment(cx);
                        }
                        if self.kind != Kind::Models {
                            self.message = Some(
                                self.task_error
                                    .clone()
                                    .unwrap_or_else(|| self.task_status.clone()),
                            );
                        }
                    }
                }
            }
            while self.logs.len() > 400 {
                self.logs.pop_front();
            }
        }
        if save
            && let Some(workspace) = &self.workspace
            && let Err(error) = workspace.save()
        {
            self.workspace_error = Some(format!("任务进度尚未保存：{error:#}"));
        }
        if self.closing {
            if self.job.is_none() && self.preview_workers == 0 {
                cx.quit();
                return;
            }
            if self
                .quit_deadline
                .is_some_and(|deadline| Instant::now() >= deadline)
            {
                if let Some(job) = &self.job {
                    job.cancel();
                }
                cx.quit();
                return;
            }
        } else {
            self.start_next_task(cx);
        }
        if save || (self.job.is_some() && self.last_tick.elapsed() >= Duration::from_secs(1)) {
            self.last_tick = Instant::now();
            cx.notify();
        }
    }
    fn refresh_library(&mut self, cx: &mut Context<Self>) {
        let locations = self
            .workspace
            .as_ref()
            .map(|w| w.state.libraries.clone())
            .unwrap_or_else(|| {
                vec![workspace::LibraryLocation {
                    id: "legacy".into(),
                    name: "课程库".into(),
                    root: self.library_root.clone(),
                    previous_roots: Vec::new(),
                }]
            });
        self.library_generation = self.library_generation.wrapping_add(1);
        let generation = self.library_generation;
        self.loading = true;
        let task = cx.background_executor().spawn(async move {
            locations
                .into_iter()
                .map(|location| {
                    let scan = backend::scan_library(&location.root);
                    let organization = organize::Library::load(&location.root);
                    (location, scan, organization)
                })
                .collect::<Vec<_>>()
        });
        cx.spawn(async move |this, cx| {
            let results = task.await;
            let _ = this.update(cx, |this, cx| {
                if this.library_generation != generation {
                    return;
                }
                this.loading = false;
                this.courses.clear();
                this.library_issues.clear();
                this.library_materials.clear();
                this.library_indexes.clear();
                for (location, scan, organization) in results {
                    match scan {
                        Ok(scan) => {
                            this.courses.extend(scan.courses);
                            this.library_issues.extend(scan.issues);
                            this.library_materials.extend(scan.materials);
                        }
                        Err(error) => this
                            .library_issues
                            .push(format!("{}暂时无法读取：{error:#}", location.name)),
                    }
                    match organization {
                        Ok(library) => {
                            this.library_indexes.insert(location.root, library);
                        }
                        Err(error) => this.library_issues.push(format!(
                            "{}的文件夹记录暂时无法读取：{error:#}",
                            location.name
                        )),
                    }
                }
                this.courses.sort_by_key(|c| std::cmp::Reverse(c.modified));
                this.apply_course_title_aliases();
                if let Some(library) = this.library_indexes.get(&this.library_root) {
                    this.library = library.clone();
                    this.library_error = None;
                } else if let Some((root, library)) = this
                    .workspace
                    .as_ref()
                    .and_then(|workspace| workspace.state.libraries.first())
                    .and_then(|location| {
                        this.library_indexes
                            .get(&location.root)
                            .map(|library| (location.root.clone(), library.clone()))
                    })
                {
                    // The configured default is not a registered library; follow
                    // the workspace's first readable location instead.
                    this.library_root = root;
                    this.library = library;
                    this.library_error = None;
                } else {
                    this.library_error = Some("当前库的文件夹信息暂时无法读取".into());
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn course_location(&self, course: &Course) -> Option<&workspace::LibraryLocation> {
        self.workspace
            .as_ref()?
            .state
            .libraries
            .iter()
            .filter(|lib| organize::relative_key(&lib.root, &course.storage_dir()).is_ok())
            .max_by_key(|lib| lib.root.components().count())
    }
    fn course_folder(&self, course: &Course) -> Option<u64> {
        let root = &self.course_location(course)?.root;
        self.library_indexes
            .get(root)?
            .folder(root, &course.storage_dir())
    }
    fn open_course(&mut self, course: Course, cx: &mut Context<Self>) {
        self.save_reading_position(cx);
        cx.notify();
        self.reading = true;
        self.read_generation = self.read_generation.wrapping_add(1);
        let generation = self.read_generation;
        let origin = self.page;
        self.result_origin = origin;
        let task = cx
            .background_executor()
            .spawn(async move { backend::read_preview(course) });
        cx.spawn(async move |this, cx| {
            let result = task.await;
            let _ = this.update(cx, |this, cx| {
                if this.read_generation != generation || this.page != origin {
                    return;
                }
                this.reading = false;
                match result {
                    Ok(preview) => {
                        if let Some(workspace) = &mut this.workspace {
                            let artifact = preview.course.dir.clone();
                            if let Err(error) = workspace.transaction(|state| {
                                for task in &mut state.tasks {
                                    if task.artifact.as_ref() == Some(&artifact) {
                                        task.unread = false;
                                    }
                                }
                                Ok(())
                            }) {
                                this.workspace_error = Some(format!("阅读状态尚未保存：{error:#}"));
                            }
                        }
                        if this
                            .message
                            .as_deref()
                            .is_some_and(|m| m.starts_with("已加入任务"))
                        {
                            this.message = None;
                        }
                        this.result_tab = 0;
                        this.preview = Some(preview);
                        this.apply_course_title_aliases();
                        this.restore_reading_position(cx);
                        this.page = Page::Result;
                    }
                    Err(error) => this.message = Some(format!("读取笔记失败：{error:#}")),
                }
                cx.notify();
            });
        })
        .detach();
    }
}

impl Drop for Desktop {
    fn drop(&mut self) {
        if let Some(cancel) = &self.preview_cancel {
            cancel.store(true, std::sync::atomic::Ordering::Relaxed);
        }
    }
}

fn dispatch_desktop_action(
    view: &WeakEntity<Desktop>,
    cx: &mut App,
    action: impl FnOnce(&mut Desktop, &mut Window, &mut Context<Desktop>) + 'static,
) {
    let Some(handle) = cx.active_window() else {
        return;
    };
    let view = view.clone();
    // Global action listeners run while the dispatching window is borrowed.
    // Update it after dispatch completes so the native menu fallback can run too.
    cx.defer(move |cx| {
        let _ = handle.update(cx, |_, window, cx| {
            if !window.has_active_dialog(cx) {
                let _ = view.update(cx, |this, cx| action(this, window, cx));
            }
        });
    });
}

fn main() {
    a11y::init_validation_diagnostics();
    let app = gpui_platform::application().with_assets(icons::Assets);
    app.on_reopen(|cx| {
        cx.activate(true);
        for handle in cx.windows() {
            let _ = cx.update_window(handle, |_, window, _| window.activate_window());
        }
    });
    app.run(|cx| {
        gpui_component::init(cx);
        gpui_component::set_locale("zh-CN");
        theme::init(cx);
        // Debug validation uses the same native window and render path at an exact size.
        // Release builds always use the ordinary initial window size.
        let initial_size = if cfg!(debug_assertions) {
            std::env::var("COURSE2MD_VALIDATION_WINDOW")
                .ok()
                .and_then(|value| {
                    let (width, height) = value.split_once('x')?;
                    let width = width.parse::<f32>().ok()?;
                    let height = height.parse::<f32>().ok()?;
                    (width.is_finite() && height.is_finite() && width >= 860. && height >= 620.)
                        .then_some(size(px(width), px(height)))
                })
                .unwrap_or(size(px(1140.), px(820.)))
        } else {
            size(px(1140.), px(820.))
        };
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::centered(initial_size, cx)),
                window_min_size: Some(size(px(860.), px(620.))),
                titlebar: Some(TitlebarOptions {
                    traffic_light_position: Some(point(px(16.), px(19.))),
                    ..TitleBar::title_bar_options()
                }),
                ..TitleBar::window_options()
            },
            |window, cx| {
                window.set_window_title("course2md");
                window
                    .observe_window_appearance(|window, _| window.refresh())
                    .detach();
                let view = cx.new(|cx| Desktop::new(window, cx));
                let weak = view.downgrade();
                let quit_view = weak.clone();
                let settings_view = weak.clone();
                let about_view = weak.clone();
                let new_view = weak.clone();
                let search_view = weak.clone();
                cx.on_action(move |_: &OpenAbout, cx| {
                    dispatch_desktop_action(&about_view, cx, |this, window, cx| {
                        this.settings_tab = 3;
                        this.open_settings(window, cx);
                    });
                });
                cx.on_action(move |_: &OpenSettings, cx| {
                    dispatch_desktop_action(&settings_view, cx, |this, window, cx| {
                        this.open_settings(window, cx);
                    });
                });
                cx.on_action(move |_: &NewNote, cx| {
                    dispatch_desktop_action(&new_view, cx, |this, window, cx| {
                        this.new_note_from_action(window, cx);
                    });
                });
                cx.on_action(move |_: &SearchContent, cx| {
                    dispatch_desktop_action(&search_view, cx, |this, window, cx| {
                        if this.page == Page::Result {
                            this.open_reader_find(window, cx);
                        } else {
                            this.navigate(Page::Library, cx);
                            this.inputs[&Field::Search]
                                .update(cx, |input, cx| input.focus(window, cx));
                        }
                    });
                });
                cx.on_action(move |_: &Quit, cx| {
                    if quit_view
                        .update(cx, |this, cx| this.request_close(cx))
                        .unwrap_or(true)
                    {
                        cx.quit();
                    }
                });
                window.on_window_should_close(cx, move |_, cx| {
                    let saved = weak
                        .update(cx, |this, cx| {
                            this.flush_settings_for_exit(cx) && this.save_current_draft(cx)
                        })
                        .unwrap_or(true);
                    if saved {
                        cx.hide();
                    }
                    false
                });
                cx.new(|cx| Root::new(view, window, cx))
            },
        )
        .expect("无法创建 course2md 窗口");
        cx.on_window_closed(|cx, _| {
            if cx.windows().is_empty() {
                cx.quit();
            }
        })
        .detach();
        cx.bind_keys([
            // Focused controls activate on key-up. The dialog's generic Enter
            // confirmation otherwise closes it on key-down before that action
            // can run (including validation, cancellation, and image controls).
            KeyBinding::new("enter", NoAction, Some("Dialog")),
            KeyBinding::new("secondary-q", Quit, None),
            KeyBinding::new("secondary-,", OpenSettings, None),
            KeyBinding::new("secondary-n", NewNote, None),
            KeyBinding::new("secondary-f", SearchContent, None),
        ]);
        cx.set_menus([
            gpui::Menu::new("course2md").items([
                gpui::MenuItem::action("关于 course2md", OpenAbout),
                gpui::MenuItem::action("设置…", OpenSettings),
                gpui::MenuItem::action("退出 course2md", Quit),
            ]),
            gpui::Menu::new("文件").items([gpui::MenuItem::action("生成笔记", NewNote)]),
            gpui::Menu::new("查找")
                .items([gpui::MenuItem::action("搜索课程或当前笔记", SearchContent)]),
        ]);
        cx.activate(true);
    });
}
