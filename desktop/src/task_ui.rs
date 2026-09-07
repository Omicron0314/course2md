//! Persistent drafts and task execution. Navigation never supplies retry inputs.
use super::*;
use crate::{
    notes::default_output,
    preferences::ServiceRefs,
    theme::*,
    workspace::{Intent, TaskPlan, TaskRecord, TaskState},
};
use anyhow::{Context as _, Result, ensure};
use gpui_component::button::*;

fn update_draft_source_title(
    draft: &mut workspace::Draft,
    input: String,
    title: String,
    source: Option<source::Source>,
) {
    // A source edit can clear Draft::title while the input still contains the
    // previous automatic title. Capture its provenance before changing sources;
    // repeated autosaves while the new metadata is pending must retain it too.
    let unchanged_automatic_title = !draft.custom_title && draft.title == title;
    draft.change_source(input);
    if let Some(source) = source {
        draft.source = Some(source);
    }
    if unchanged_automatic_title {
        draft.title = title;
    } else if draft.title != title {
        draft.custom_title = !title.is_empty()
            && draft
                .source
                .as_ref()
                .is_none_or(|source| source.title != title);
        draft.title = title;
    }
}

impl Desktop {
    pub fn retry_workspace_records(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(workspace) = &self.workspace {
            match workspace.save() {
                Ok(()) => self.workspace_error = None,
                Err(error) => self.workspace_error = Some(format!("记录仍未保存：{error:#}")),
            }
        } else {
            let root = self
                .config
                .defaults
                .out
                .clone()
                .unwrap_or_else(default_output);
            match workspace::Workspace::open(
                root,
                ConversionOptions::from_config(&self.preferences.defaults_config()),
            ) {
                Ok(workspace) => {
                    self.message = workspace.recovery.clone();
                    self.workspace = Some(workspace);
                    self.workspace_error = None;
                    self.restore_draft(window, cx);
                    self.refresh_library(cx);
                }
                Err(error) => {
                    self.workspace_error = Some(format!("记录仍无法读取，原件已保留：{error:#}"))
                }
            }
        }
        cx.notify();
    }

    /// Called only by the explicit global recovery action; rebuilding never starts the queue.
    pub fn rebuild_workspace_records(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.job.is_some() || self.storage_ui.busy {
            self.workspace_error =
                Some("当前还有处理或文件移动在进行，请等候停止后再重建记录。".into());
            cx.notify();
            return;
        }
        let root = self
            .config
            .defaults
            .out
            .clone()
            .unwrap_or_else(default_output);
        let options = ConversionOptions::from_config(&self.preferences.defaults_config());
        let result = if let Some(workspace) = &self.workspace {
            workspace::Workspace::rebuild_at(workspace.storage_path().to_owned(), root, options)
        } else {
            workspace::Workspace::rebuild(root, options)
        };
        match result {
            Ok(workspace) => {
                self.message = workspace.recovery.clone();
                self.workspace = Some(workspace);
                self.workspace_error = None;
                self.restore_draft(window, cx);
                self.refresh_library(cx);
            }
            Err(error) => {
                self.workspace_error = Some(format!("记录尚未重建，原件仍保留：{error:#}"))
            }
        }
        cx.notify();
    }

    pub fn recommended_local_provider(&self) -> course2md::config::AsrProvider {
        use course2md::config::AsrProvider;
        match &self.environment {
            Some(environment) if environment.apple => AsrProvider::Coreml,
            Some(environment) if environment.gpu.is_some() && environment.llama => AsrProvider::Gpu,
            Some(environment) if environment.npu && !environment.llama => AsrProvider::Npu,
            Some(_) => AsrProvider::Cpu,
            None => course2md::config::default_provider_hint(),
        }
    }

    pub fn save_current_draft(&mut self, cx: &mut Context<Self>) -> bool {
        if self.draft_loading {
            return true;
        }
        let input = self.value(Field::Source, cx);
        let title = self.value(Field::Title, cx);
        let source = self.source_preview.clone();
        let options = self.task_options.clone();
        let folder = self.target_folder;
        let scroll = f32::from(self.scrolls[Page::New as usize].offset().y);
        let Some(workspace) = &mut self.workspace else {
            return false;
        };
        let result = workspace.transaction(|state| {
            let draft = state.draft_mut().context("没有可保存的草稿")?;
            update_draft_source_title(draft, input, title, source);
            use workspace::Override;
            for (changed, field) in [
                (
                    draft.options.provider != options.provider,
                    Override::Provider,
                ),
                (
                    draft.options.source_mode != options.source_mode,
                    Override::TextSource,
                ),
                (draft.options.llm != options.llm, Override::Proofread),
                (
                    draft.options.summarize != options.summarize,
                    Override::Summary,
                ),
                (draft.options.vision != options.vision, Override::Vision),
                (
                    draft.options.keep_video != options.keep_video,
                    Override::KeepVideo,
                ),
                (draft.options.formats != options.formats, Override::Formats),
            ] {
                if changed {
                    draft.overrides.insert(field);
                }
            }
            draft.options = options;
            draft.folder = folder;
            draft.scroll = scroll;
            draft.updated = workspace::now();
            Ok(())
        });
        self.draft_deadline = None;
        if let Err(error) = result {
            self.workspace_error =
                Some(format!("草稿尚未保存：{error:#}。当前内容仍保留在窗口中。"));
            cx.notify();
            return false;
        }
        self.workspace_error = None;
        true
    }

    pub fn restore_draft(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(draft) = self
            .workspace
            .as_ref()
            .and_then(|w| w.state.draft())
            .cloned()
        else {
            return;
        };
        self.draft_loading = true;
        self.online = draft.online;
        self.last_source_input = draft.input.clone();
        self.source_preview = draft.source.clone();
        self.task_options = draft.options.clone();
        self.target_folder = draft.folder;
        self.preview_error = None;
        self.source_validation = None;
        self.source_candidates.clear();
        self.source_collection_title = None;
        self.subtitle_error = draft.source.as_ref().and_then(|source| {
            source
                .subtitle_read_error
                .as_ref()
                .map(ToString::to_string)
                .or_else(|| {
                    source
                        .subtitle_request
                        .as_ref()
                        .map(|_| "所选字幕还未读取完成，请继续读取或选择其他文字来源。".into())
                })
        });
        self.subtitle_loading = false;
        self.inputs[&Field::Source].update(cx, |s, cx| s.set_value(draft.input, window, cx));
        self.inputs[&Field::Title].update(cx, |s, cx| s.set_value(draft.title, window, cx));
        self.scrolls[Page::New as usize].set_offset(point(px(0.), px(draft.scroll)));
        self.draft_loading = false;
        self.draft_deadline = None;
        self.source_deadline = None;
        cx.notify();
    }

    pub fn select_draft(&mut self, id: String, window: &mut Window, cx: &mut Context<Self>) {
        if !self.save_current_draft(cx) {
            return;
        }
        self.invalidate_source();
        if let Some(workspace) = &mut self.workspace {
            match workspace.transaction(|state| {
                ensure!(state.drafts.iter().any(|d| d.id == id), "此草稿已不存在");
                state.current_draft = id;
                Ok(())
            }) {
                Ok(()) => self.restore_draft(window, cx),
                Err(error) => self.workspace_error = Some(format!("无法切换草稿：{error:#}")),
            }
        }
    }

    pub fn new_draft(
        &mut self,
        online: bool,
        inherit_folder: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.save_current_draft(cx) {
            return;
        }
        let defaults = ConversionOptions::from_config(&self.preferences.defaults_config());
        self.invalidate_source();
        if let Some(workspace) = &mut self.workspace {
            let destination = if inherit_folder {
                self.folder_filter.filter(|id| *id != 0).and_then(|folder| {
                    workspace
                        .state
                        .libraries
                        .iter()
                        .find(|lib| lib.root == self.library_root)
                        .map(|lib| (lib.id.clone(), folder))
                })
            } else {
                None
            };
            match workspace.transaction(|state| {
                state.fresh_draft(online, defaults, destination);
                Ok(())
            }) {
                Ok(()) => {
                    self.restore_draft(window, cx);
                    self.page = Page::New;
                }
                Err(error) => self.workspace_error = Some(format!("尚未建立草稿：{error:#}")),
            }
        }
        cx.notify();
    }

    pub fn switch_draft_kind(&mut self, online: bool, window: &mut Window, cx: &mut Context<Self>) {
        if self.online == online {
            return;
        }
        if !self.save_current_draft(cx) {
            return;
        }
        let defaults = ConversionOptions::from_config(&self.preferences.defaults_config());
        self.invalidate_source();
        if let Some(workspace) = &mut self.workspace {
            match workspace.transaction(|state| {
                state.switch_source_kind(online, defaults);
                Ok(())
            }) {
                Ok(()) => self.restore_draft(window, cx),
                Err(error) => self.workspace_error = Some(format!("无法切换来源：{error:#}")),
            }
        }
    }

    fn build_plan(&self) -> Result<TaskPlan> {
        self.ordinary_preferences_ready_for_submit()?;
        let workspace = self.workspace.as_ref().context("草稿与任务记录尚未恢复")?;
        let draft = workspace.state.draft().context("没有当前草稿")?;
        ensure!(
            !draft.input.is_empty(),
            if draft.online {
                "先粘贴视频链接"
            } else {
                "先选择一个视频"
            }
        );
        ensure!(
            !self.subtitle_loading && self.preview_cancel.is_none(),
            "正在确认来源，请等待读取完成"
        );
        ensure!(
            self.source_preview.is_some(),
            "本次来源尚未确认，请重新读取视频"
        );
        ensure!(
            self.subtitle_error.is_none(),
            "当前字幕尚未确认，请重新读取字幕，或明确选择其他文字来源"
        );
        let source = draft.source.clone().context("请先读取并确认视频")?;
        let environment = self
            .environment
            .as_ref()
            .context("正在检查生成笔记需要的组件，请稍候")?;
        ensure!(
            environment.engine,
            "生成引擎无法启动。请在设置的应用与诊断中检查应用组件。"
        );
        ensure!(
            environment.ffmpeg && environment.ffprobe,
            "视频读取组件不可用。请在设置的应用与诊断中查看 FFmpeg 的安装方法。"
        );
        ensure!(
            !source.online || environment.ytdlp,
            "在线视频读取组件不可用。请在设置的应用与诊断中查看 yt-dlp 的安装方法。"
        );
        ensure!(
            draft.options.source_mode == 2
                || (source.subtitle_request.is_none() && source.subtitle_read_error.is_none()),
            "所选字幕还未确认，请继续读取或明确选择其他文字来源。"
        );
        ensure!(
            !source.identity.is_empty(),
            "视频身份尚未确认，请重新读取视频"
        );
        ensure!(!draft.title.trim().is_empty(), "请填写笔记名称");
        let library = workspace
            .state
            .library(&draft.library_id)
            .context("保存位置已不在课程库中，请选择保存位置")?;
        ensure!(
            library.root.is_dir(),
            "保存位置暂时不可访问。请连接对应磁盘，或在设置的存储中选择其他位置。草稿和已有任务仍保留。"
        );
        workspace::check_library(library)?;
        if let Some(folder) = draft.folder {
            ensure!(
                organize::Library::load(&library.root)?
                    .folders
                    .contains_key(&folder),
                "文件夹已删除。请选择其他文件夹，或保存到未分类。"
            );
        }
        let mut config = draft
            .base_config
            .clone()
            .unwrap_or_else(|| self.preferences.defaults_config());
        draft.options.apply_to(&mut config);
        config.defaults.model_dir = Some(course2md::config::model_dir_from(
            config.defaults.model_dir.as_deref(),
        ));
        let using_subtitle = draft.options.source_mode != 2
            && (source.selected_subtitle.is_some() || draft.subtitle.is_some());
        if using_subtitle {
            if let Some(subtitle) = &source.selected_subtitle {
                ensure!(
                    subtitle.source_identity == source.identity,
                    "所选字幕与当前视频不匹配。请重新读取字幕，已填写的内容会保留。"
                );
            }
            config.defaults.transcript_source = Some(course2md::config::TranscriptSource::Subtitle);
        } else {
            use course2md::subtitle::SubtitleEvidence;
            ensure!(
                draft.options.source_mode == 2
                    || (draft.options.source_mode == 0
                        && matches!(
                            source.subtitles,
                            SubtitleEvidence::NoneFound | SubtitleEvidence::Unsupported { .. }
                        )),
                "字幕还未确认。请读取字幕，或明确选择识别视频声音。"
            );
            config.defaults.transcript_source = Some(course2md::config::TranscriptSource::Asr);
            let environment = self
                .environment
                .as_ref()
                .context("正在检查本机识别能力，请稍候")?;
            let provider = config
                .defaults
                .provider
                .unwrap_or_else(|| self.recommended_local_provider());
            use course2md::config::AsrProvider;
            match provider {
                AsrProvider::Coreml => ensure!(
                    environment.apple,
                    "Apple 原生识别组件不可用。请选择其他本机识别方式，或在设置中检查应用组件。"
                ),
                AsrProvider::Gpu => ensure!(
                    environment.llama && environment.gpu.is_some(),
                    "没有检测到可用的 GPU 识别引擎。请选择 CPU 或其他本机识别方式。"
                ),
                AsrProvider::Cpu => ensure!(
                    environment.llama,
                    "CPU 识别引擎尚未安装。请在设置的应用与诊断中查看安装方法。"
                ),
                AsrProvider::Npu => ensure!(
                    environment.npu,
                    "没有检测到可用的 Intel NPU 识别环境。请选择其他本机识别方式。"
                ),
                AsrProvider::Api => (),
            }
            config.defaults.provider = Some(provider);
            if provider != AsrProvider::Api
                && config
                    .defaults
                    .asr_model
                    .as_deref()
                    .is_none_or(|model| model.trim().is_empty())
            {
                config.defaults.asr_model = Some(if provider == AsrProvider::Npu {
                    course2md::npu::resolve_npu_model(None)
                } else {
                    "qwen3-1.7b".into()
                });
            }
        }
        let defaults = self.preferences.default_refs();
        let refs = ServiceRefs {
            asr: draft.asr_service.clone().or(defaults.asr),
            llm: draft.ai_service.clone().or(defaults.llm),
        };
        let config = self.preferences.config_for_refs(&config, &refs)?;
        validate_plan_config(&source.input, &config)?;
        if !source.online {
            ensure!(
                std::path::Path::new(&source.input).is_file(),
                "原视频已移动或无法读取，请重新选择视频"
            );
        }
        Ok(TaskPlan {
            operation: Default::default(),
            source_id: source.identity.clone(),
            source,
            title: draft.title.clone(),
            library_id: draft.library_id.clone(),
            folder: draft.folder,
            options: draft.options.clone(),
            subtitle: draft.subtitle.clone(),
            config,
            asr_service: refs.asr,
            ai_service: refs.llm,
        })
    }

    /// The form and submission use the same static checks; no service request is made.
    pub fn submission_issue(&self) -> Option<String> {
        self.build_plan().err().map(|error| format!("{error:#}"))
    }

    pub fn matching_current_task(&self) -> Option<&TaskRecord> {
        let plan = self.build_plan().ok()?;
        self.workspace.as_ref()?.state.tasks.iter().find(|task| {
            !task.state.finished() && task.handled_by.is_none() && task.plan.same_work(&plan)
        })
    }

    pub fn enqueue_current(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.save_current_draft(cx) {
            return;
        }
        self.message = None;
        let plan = match self.build_plan() {
            Ok(plan) => plan,
            Err(error) => {
                self.source_validation = Some(format!("{error:#}"));
                let field = if self.source_preview.is_some()
                    && self.value(Field::Title, cx).trim().is_empty()
                {
                    Some(Field::Title)
                } else if self.source_preview.is_none() && self.online {
                    Some(Field::Source)
                } else {
                    None
                };
                if let Some(field) = field {
                    self.inputs[&field].update(cx, |input, cx| input.focus(window, cx));
                }
                self.show_options = true;
                cx.notify();
                return;
            }
        };
        let Some(workspace) = &mut self.workspace else {
            return;
        };
        let parent = workspace
            .state
            .draft()
            .and_then(|draft| draft.retry_of.clone());
        let result = workspace.transaction(|state| {
            let (id, created) = state.enqueue(plan, parent)?;
            if created && let Some(draft) = state.draft_mut() {
                draft.submitted_task = Some(id.clone());
            }
            Ok((id, created))
        });
        match result {
            Ok((id, created)) => {
                self.source_validation = None;
                self.message = Some(if created {
                    "已加入任务。你可以继续准备下一篇笔记。".into()
                } else {
                    "已有相同处理任务。该任务采用原来的名称和保存位置；本草稿中的更改仍保留。"
                        .into()
                });
                self.select_task(&id, cx);
                // M4b: 提交后留在工作台，进度在盒内任务卡呈现。
                self.page = Page::New;
                self.start_next_task(cx);
            }
            Err(error) => {
                self.workspace_error =
                    Some(format!("任务尚未加入队列：{error:#}。来源和选项已保留。"))
            }
        }
        cx.notify();
    }

    fn request_for(&self, task: &TaskRecord) -> Result<course2md::execution::Request> {
        let location = self
            .workspace
            .as_ref()
            .and_then(|w| w.state.library(&task.plan.library_id))
            .context("任务保存位置暂时不可用")?;
        workspace::check_library(location)?;
        ensure!(
            task.work_dir == location.root.join(".course2md/work").join(&task.id),
            "任务位置与课程库记录不匹配，请先恢复存储位置"
        );
        let refs = ServiceRefs {
            asr: task.plan.asr_service.clone(),
            llm: task.plan.ai_service.clone(),
        };
        let config = self
            .preferences
            .resolve_for_execution(&task.plan.config, &refs)?
            .into_config();
        let course_id = format!(
            "course-{}",
            &course2md::execution::digest(task.plan.source_id.as_bytes())[..32]
        );
        let subtitle_events = if task.plan.options.source_mode != 2 && task.plan.subtitle.is_none()
        {
            task.plan
                .source
                .selected_subtitle
                .as_ref()
                .map(|s| s.events.clone())
        } else {
            None
        };
        let service_versions = [("asr", refs.asr), ("llm", refs.llm)]
            .into_iter()
            .filter_map(|(purpose, reference)| reference.map(|r| (purpose.to_owned(), r)))
            .collect();
        let source = self
            .workspace
            .as_ref()
            .and_then(|workspace| workspace.state.reader_sources.get(&task.plan.source_id))
            .filter(|path| path.is_file())
            .map(|path| path.to_string_lossy().into_owned())
            .filter(|_| {
                !task.plan.source.online && !std::path::Path::new(&task.plan.source.input).is_file()
            })
            .unwrap_or_else(|| task.plan.source.input.clone());
        Ok(course2md::execution::Request {
            schema: 1,
            operation: task.plan.operation.clone(),
            task_id: task.id.clone(),
            version_id: task.id.clone(),
            course_dir: location.root.join(&course_id),
            course_id,
            source,
            source_id: task.plan.source_id.clone(),
            title: task.plan.title.clone(),
            author: task.plan.source.author.clone(),
            duration: task.plan.source.duration,
            subtitle: if task.plan.options.source_mode != 2 {
                task.plan.subtitle.clone()
            } else {
                None
            },
            subtitle_events,
            config,
            work_dir: task.work_dir.clone(),
            allow_unauthenticated_asr: task
                .plan
                .asr_service
                .as_deref()
                .and_then(|id| self.preferences.version(id))
                .is_some_and(|version| {
                    version.config.authentication == preferences::Authentication::None
                }),
            control_path: Some(task.work_dir.join("control.json")),
            service_versions,
        })
    }

    fn write_task_control(&self, task: &TaskRecord, resend: Vec<String>) -> Result<()> {
        let library = self
            .workspace
            .as_ref()
            .and_then(|workspace| workspace.state.library(&task.plan.library_id))
            .context("任务保存位置尚未恢复")?;
        workspace::check_library(library)?;
        ensure!(
            task.work_dir == library.root.join(".course2md/work").join(&task.id),
            "任务工作目录与保存位置不匹配"
        );
        let mut authorizations = task.resend.clone();
        for id in resend {
            if !authorizations.contains(&id) {
                authorizations.push(id);
            }
        }
        let stopped: Vec<_> = self
            .preferences
            .versions()
            .filter(|version| self.preferences.is_service_stopped(&version.service_id))
            .map(|version| version.id.clone())
            .collect();
        std::fs::create_dir_all(&task.work_dir)?;
        course2md::checkpoint::atomic_write(
            &task.work_dir.join("control.json"),
            &serde_json::to_vec(&serde_json::json!({
                "intent": task.intent, "stopped_services": stopped, "resend": authorizations,
            }))?,
        )
    }

    pub fn refresh_dispatch_controls(&mut self, cx: &mut Context<Self>) {
        if let Some(task) = self
            .active_task
            .as_deref()
            .and_then(|id| self.workspace.as_ref()?.state.task(id))
            .cloned()
            && let Err(error) = self.write_task_control(&task, Vec::new())
        {
            self.workspace_error = Some(format!("停止新请求的指令尚未保存：{error:#}"));
            if let Some(job) = &self.job {
                job.cancel();
            }
            cx.notify();
        }
    }

    pub fn start_next_task(&mut self, cx: &mut Context<Self>) {
        if self.job.is_some() || self.closing || self.storage_ui.busy {
            return;
        }
        let Some(task) = self
            .workspace
            .as_ref()
            .and_then(|w| w.state.next_task())
            .cloned()
        else {
            return;
        };
        let request = self.request_for(&task).and_then(|request| {
            request.validate()?;
            self.write_task_control(&task, Vec::new())?;
            Ok(request)
        });
        let result = request.and_then(|request| {
            // The durable running state must exist before the subprocess can do work.
            self.workspace
                .as_mut()
                .context("任务记录不可用")?
                .transaction(|state| {
                    let record = state.task_mut(&task.id).context("任务不存在")?;
                    record.state = TaskState::Running;
                    record.error = None;
                    record.updated = workspace::now();
                    Ok(())
                })?;
            Job::start_task(&request)
        });
        match result {
            Ok(job) => {
                self.job = Some(job);
                self.active_task = Some(task.id.clone());
                self.kind = Kind::Convert;
                self.cancelling = false;
                self.progress.clear();
                self.logs.clear();
                self.pending_done = None;
                self.task_error = None;
                self.task_status = format!("正在生成《{}》", task.plan.title);
            }
            Err(error) => {
                let message = format!("{error:#}");
                if let Some(workspace) = &mut self.workspace {
                    let _ = workspace
                        .transaction(|state| {
                            let record = state.task_mut(&task.id).context("任务不存在")?;
                            record.state = TaskState::NeedsAttention;
                            record.error = Some(message);
                            record.unread = true;
                            Ok(())
                        })
                        .map_err(|error| {
                            self.workspace_error = Some(format!("任务状态尚未保存：{error:#}"));
                        });
                }
            }
        }
        cx.notify();
    }

    pub fn set_task_intent(&mut self, id: String, intent: Intent, cx: &mut Context<Self>) {
        let active = self.active_task.as_deref() == Some(&id) && self.job.is_some();
        if active && intent == Intent::Run {
            self.message = Some("当前任务还在处理，请等候当前操作结束。".into());
            cx.notify();
            return;
        }
        let Some(workspace) = &mut self.workspace else {
            return;
        };
        match workspace.transaction(|state| {
            state.set_intent(&id, intent)?;
            if active && intent != Intent::Run {
                state.task_mut(&id).context("任务记录不存在")?.state = TaskState::Pausing;
            }
            Ok(())
        }) {
            Ok(()) => {
                if let Some(task) = self
                    .workspace
                    .as_ref()
                    .and_then(|w| w.state.task(&id))
                    .cloned()
                    && let Err(error) = self.write_task_control(&task, Vec::new())
                {
                    self.workspace_error = Some(format!("任务指令尚未保存：{error:#}"));
                    if self.active_task.as_ref() == Some(&id)
                        && let Some(job) = &self.job
                    {
                        job.cancel();
                    }
                }
                self.start_next_task(cx);
            }
            Err(error) => self.workspace_error = Some(format!("任务状态尚未保存：{error:#}")),
        }
        cx.notify();
    }

    pub fn select_task(&mut self, id: &str, cx: &mut Context<Self>) {
        if let Some(workspace) = &mut self.workspace {
            if let Err(error) = workspace.transaction(|state| {
                state.selected_task = Some(id.to_owned());
                if let Some(task) = state.task_mut(id) {
                    task.unread = false;
                }
                Ok(())
            }) {
                self.workspace_error = Some(format!("任务选择尚未保存：{error:#}"));
            }
        }
        cx.notify();
    }

    pub fn resend_uncertain(&mut self, id: String, cx: &mut Context<Self>) {
        if self.active_task.as_deref() == Some(&id) && self.job.is_some() {
            self.message = Some("正在保存当前结果，请等候当前任务停止后再选择重新发送。".into());
            cx.notify();
            return;
        }
        let Some(task) = self
            .workspace
            .as_ref()
            .and_then(|w| w.state.task(&id))
            .cloned()
        else {
            return;
        };
        if let Some(followup) = &task.handled_by {
            self.select_task(followup, cx);
            return;
        }
        let requests: Vec<_> = task
            .blocked
            .iter()
            .filter(|b| b.reason == "uncertain")
            .filter_map(|b| b.request_id.clone())
            .collect();
        if requests.is_empty() {
            return;
        }
        if task.artifact.is_some() {
            let components = task
                .blocked
                .iter()
                .filter(|b| b.reason == "uncertain")
                .filter_map(|b| b.purpose.as_deref())
                .map(|purpose| {
                    if purpose.contains("summary") {
                        "summary".to_owned()
                    } else {
                        "proofreading".to_owned()
                    }
                })
                .collect();
            self.reprocess_task(id, components, requests, cx);
            return;
        }
        if let Some(workspace) = &mut self.workspace {
            let result = workspace.transaction(|state| state.authorize_uncertain(&id, &requests));
            if let Err(error) = result {
                self.workspace_error = Some(format!("重新发送的选择尚未保存：{error:#}"));
                return;
            }
        }
        self.start_next_task(cx);
        cx.notify();
    }

    pub fn reprocess_task(
        &mut self,
        id: String,
        mut components: Vec<String>,
        resend: Vec<String>,
        cx: &mut Context<Self>,
    ) {
        if self.active_task.as_deref() == Some(&id) && self.job.is_some() {
            self.message = Some("正在保存当前结果，请等候当前任务停止后再补做。".into());
            cx.notify();
            return;
        }
        let Some(workspace) = &mut self.workspace else {
            return;
        };
        components.sort();
        components.dedup();
        let result = workspace.transaction(|state| state.reprocess(&id, components, resend));
        match result {
            Ok(id) => {
                self.select_task(&id, cx);
                self.start_next_task(cx);
            }
            Err(error) => self.workspace_error = Some(format!("补做任务尚未建立：{error:#}")),
        }
        cx.notify();
    }

    pub fn adjust_task(&mut self, id: String, window: &mut Window, cx: &mut Context<Self>) {
        if self.active_task.as_deref() == Some(&id) && self.job.is_some() {
            self.message = Some("当前任务还在处理，请先暂停并等候当前步骤结束。".into());
            cx.notify();
            return;
        }
        if !self.save_current_draft(cx) {
            return;
        }
        let Some(workspace) = &mut self.workspace else {
            return;
        };
        let result = workspace.transaction(|state| {
            let task = state.task(&id).context("任务不存在")?.clone();
            ensure!(
                task.handled_by.is_none(),
                "此任务已有后续处理，请打开对应任务继续"
            );
            ensure!(
                !matches!(
                    task.state,
                    TaskState::Running | TaskState::Pausing | TaskState::Queued
                ),
                "当前任务还在处理，请先暂停并等候当前步骤结束"
            );
            let mut draft = workspace::Draft::new(
                task.plan.source.online,
                task.plan.library_id.clone(),
                task.plan.options.clone(),
            );
            draft.input = task.plan.source.input.clone();
            draft.source = Some(task.plan.source);
            draft.title = task.plan.title;
            draft.custom_title = true;
            draft.folder = task.plan.folder;
            draft.subtitle = task.plan.subtitle;
            draft.asr_service = task.plan.asr_service;
            draft.ai_service = task.plan.ai_service;
            draft.retry_of = Some(id);
            draft.overrides = [
                workspace::Override::Provider,
                workspace::Override::TextSource,
                workspace::Override::Proofread,
                workspace::Override::Summary,
                workspace::Override::Vision,
                workspace::Override::KeepVideo,
                workspace::Override::Formats,
            ]
            .into_iter()
            .collect();
            draft.base_config = Some(task.plan.config);
            state.current_draft = draft.id.clone();
            state.drafts.push(draft);
            Ok(())
        });
        match result {
            Ok(()) => {
                self.invalidate_source();
                self.restore_draft(window, cx);
                self.page = Page::New;
            }
            Err(error) => self.workspace_error = Some(format!("调整草稿尚未建立：{error:#}")),
        }
        cx.notify();
    }

    pub fn record_task_event(&mut self, event: &Event) {
        let Some(id) = &self.active_task else {
            return;
        };
        let Some(task) = self.workspace.as_mut().and_then(|w| w.state.task_mut(id)) else {
            return;
        };
        task.updated = workspace::now();
        match event {
            Event::Log { message } => {
                task.logs.push(message.clone());
                if task.logs.len() > 200 {
                    task.logs.drain(..task.logs.len() - 200);
                }
            }
            Event::Stage { stage, status } => {
                if stage.starts_with("scenes/") {
                    // Old workers combined scanning and extraction under this
                    // key. Do not retain its completed counter on continuation.
                    task.stages.remove("scenes");
                    if stage == "scenes/scan" && status == "start" {
                        task.stages.remove("scenes/extract");
                    }
                }
                let value = task.stages.entry(stage.clone()).or_default();
                if status == "start" {
                    value.begin();
                } else {
                    value.status = status.clone();
                }
            }
            Event::Progress {
                stage,
                current,
                total,
                message,
            } => {
                let value = task.stages.entry(stage.clone()).or_default();
                value.current = *current;
                value.total = *total;
                value.detail = message.clone();
            }
            Event::Error { message } => task.error = Some(message.clone()),
            Event::Blocked {
                reason,
                request_id,
                purpose,
                description,
                message,
            } => {
                if reason == "uncertain" {
                    task.state = TaskState::Uncertain;
                }
                let blocked = workspace::BlockedRequest {
                    reason: reason.clone(),
                    request_id: request_id.clone(),
                    purpose: purpose.clone(),
                    description: description.clone().unwrap_or_default(),
                    message: message.clone(),
                };
                if !task.blocked.contains(&blocked) {
                    task.blocked.push(blocked);
                }
                task.error = Some(message.clone());
            }
            _ => {}
        }
    }

    pub fn finish_task(
        &mut self,
        id: &str,
        success: bool,
        cancelled: bool,
        cx: &mut Context<Self>,
    ) {
        let done = self.pending_done.take();
        let manifest = done.as_ref().and_then(|d| {
            course2md::artifact::read_manifest(&d.out_dir.join("manifest.json")).ok()
        });
        let Some(workspace) = &mut self.workspace else {
            return;
        };
        let visible =
            self.page == Page::Task && workspace.state.selected_task.as_deref() == Some(id);
        let result = workspace.transaction(|state| {
            let task = state.task_mut(id).context("任务记录不存在")?;
            task.updated = workspace::now();
            task.unread = !visible;
            task.outcomes = done.as_ref().and_then(|done| done.outcomes.clone());
            workspace::reconcile_receipts(task)?;
            if let (Some(done), Some(manifest)) = (&done, &manifest) {
                task.artifact = Some(done.out_dir.clone());
                let partial = done.partial.unwrap_or(manifest.partial);
                task.state = if task.blocked.iter().any(|b| b.reason == "uncertain") {
                    TaskState::Uncertain
                } else if partial {
                    TaskState::Partial
                } else {
                    TaskState::Complete
                };
                if !partial && task.state != TaskState::Uncertain {
                    task.error = None;
                }
            } else {
                task.state = match task.intent {
                    Intent::Cancel => TaskState::Cancelled,
                    Intent::Pause | Intent::Quit => TaskState::Paused,
                    Intent::Run if task.blocked.iter().any(|b| b.reason == "uncertain") => {
                        TaskState::Uncertain
                    }
                    Intent::Run => TaskState::NeedsAttention,
                };
                if task.error.is_none() && !cancelled && task.intent == Intent::Run {
                    task.error = Some(
                        if success {
                            "处理已结束，但尚未发布可读笔记。任务材料和进度已保留。"
                        } else {
                            "生成中断，已保存的进度仍保留。打开技术详情可以查看失败步骤。"
                        }
                        .into(),
                    );
                }
            }
            Ok(task.clone())
        });
        match result {
            Ok(task) => {
                if let Some(done) = &done {
                    self.completed = Some(done.clone());
                    if let Some(root) = self
                        .workspace
                        .as_ref()
                        .and_then(|w| w.state.library(&task.plan.library_id))
                        .map(|l| l.root.clone())
                    {
                        let _ = source::save_cover(&task.plan.source, &done.out_dir).map_err(|e| {
                            self.message = Some(format!("笔记已保存，封面暂未保存：{e:#}"))
                        });
                        if let Err(error) = organize::Library::edit(&root, |library| {
                            library.assign(
                                &root,
                                &Course::from_completed(done).storage_dir(),
                                task.plan.folder,
                            )
                        }) {
                            self.message =
                                Some(format!("笔记已保存，文件夹归属暂未更新：{error:#}"));
                        }
                    }
                    self.refresh_library(cx);
                }
                self.task_status = task.state.label().into();
            }
            Err(error) => {
                self.workspace_error =
                    Some(format!("完成状态尚未保存：{error:#}。已发布的笔记仍保留。"))
            }
        }
        cx.notify();
    }

    pub fn queue_page(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let Some(workspace) = &self.workspace else {
            return v_flex()
                .gap_3()
                .child("任务记录暂时无法读取，课程库中的笔记仍可打开。")
                .into_any_element();
        };
        if workspace.state.tasks.is_empty() && self.library_materials.is_empty() {
            return v_flex()
                .gap_3()
                .py_12()
                .items_center()
                .child(
                    accessible_text("task-empty-title", "还没有生成任务")
                        .role(Role::Heading)
                        .text_lg(),
                )
                .child(accessible_text(
                    "task-empty-description",
                    "选择视频后，处理进度和生成结果会保留在这里。",
                ))
                .child(
                    control("task-new")
                        .label("生成笔记")
                        .on_click(cx.listener(|this, _, window, cx| this.begin_add(window, cx))),
                )
                .into_any_element();
        }
        let selected = workspace.state.selected_task.clone();
        let tasks = workspace.state.tasks.clone();
        let mut content = v_flex().gap_4();
        let mut historical_content = None;
        if !self.library_materials.is_empty() {
            let mut historical = v_flex()
                .gap_3()
                .p_4()
                .bg(rgb(SURFACE))
                .child(accessible_text("historical-tasks", "历史任务材料").role(Role::Heading))
                .child(accessible_text(
                    "historical-tasks-description",
                    "这些目录尚无可读正文。原文件与处理材料已保留，没有自动重新识别或发送。",
                ));
            for (index, path) in self.library_materials.iter().enumerate() {
                let target = path.clone();
                historical = historical.child(
                    v_flex()
                        .gap_1()
                        .child(
                            accessible_text(("history-path", index), path.display().to_string())
                                .text_sm(),
                        )
                        .child(
                            control(("history-open", index))
                                .self_start()
                                .label("查看保留材料")
                                .on_click(move |_, _, cx| cx.reveal_path(&target)),
                        ),
                );
            }
            historical_content = Some(historical);
        }
        for task in tasks.iter().rev() {
            let id = task.id.clone();
            let is_selected = selected.as_ref() == Some(&id);
            let heading = h_flex()
                .w_full()
                .gap_3()
                .flex_wrap()
                .child(
                    control(SharedString::from(format!("select-{id}")))
                        .ghost()
                        .justify_start()
                        .label(task.plan.title.clone())
                        .on_click(cx.listener({
                            let id = id.clone();
                            move |this, _, _, cx| this.select_task(&id, cx)
                        })),
                )
                .child(
                    accessible_text(
                        SharedString::from(format!("state-{id}")),
                        task_status_label(task),
                    )
                    .role(Role::Status)
                    .text_sm()
                    .text_color(rgb(MUTED)),
                );
            let mut card = v_flex()
                .gap_3()
                .p_4()
                .w_full()
                .bg(rgb(SURFACE))
                .border_1()
                .border_color(rgb(if is_selected { BLUE } else { LINE }))
                .rounded_md()
                .child(heading);
            if is_selected {
                let destination = self
                    .workspace
                    .as_ref()
                    .and_then(|w| w.state.library(&task.plan.library_id))
                    .map(|lib| lib.name.clone())
                    .unwrap_or_else(|| "保存位置暂时不可用".into());
                card = card.child(
                    accessible_text(
                        SharedString::from(format!("plan-{id}")),
                        format!(
                            "保存到 {destination} · {}",
                            if task.plan.source.selected_subtitle.is_some() {
                                "使用已确认字幕"
                            } else {
                                "识别视频声音"
                            }
                        ),
                    )
                    .text_sm()
                    .text_color(rgb(MUTED)),
                );
                for (text_id, line) in self.stage_detail_lines(task) {
                    card = card.child(
                        h_flex()
                            .gap_3()
                            .child(accessible_text(text_id, line).text_color(rgb(MUTED))),
                    );
                }
                let uncertain: Vec<_> = task
                    .blocked
                    .iter()
                    .filter(|b| b.reason == "uncertain")
                    .collect();
                if let Some(error) = &task.error
                    && uncertain.is_empty()
                {
                    card = card.child(
                        accessible_text(SharedString::from(format!("error-{id}")), error.clone())
                            .role(Role::Alert)
                            .text_color(rgb(0xa32626)),
                    );
                }
                if let Some(block) = self.uncertain_block(task, cx) {
                    card = card.child(block);
                }
                let mut actions = h_flex().gap_2().flex_wrap();
                if let Some(library) = self
                    .workspace
                    .as_ref()
                    .and_then(|workspace| workspace.state.library(&task.plan.library_id))
                    && library.root.is_dir()
                    && !library.root.join(".course2md-library-id").exists()
                {
                    let library_id = library.id.clone();
                    actions = actions.child(
                        control(SharedString::from(format!("reassociate-{id}")))
                            .label("重新关联此保存位置")
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.begin_library_reassociation(library_id.clone(), window, cx)
                            })),
                    );
                }
                if let Some(followup) = &task.handled_by {
                    let next = followup.clone();
                    actions = actions.child(
                        control(SharedString::from(format!("followup-{id}")))
                            .label("查看后续处理任务")
                            .on_click(
                                cx.listener(move |this, _, _, cx| this.select_task(&next, cx)),
                            ),
                    );
                }
                if matches!(task.state, TaskState::Running | TaskState::Queued)
                    || (self.active_task.as_deref() == Some(&id) && self.job.is_some())
                {
                    actions = actions
                        .child(
                            control(SharedString::from(format!("pause-{id}")))
                                .label("暂停")
                                .on_click(cx.listener({
                                    let id = id.clone();
                                    move |this, _, _, cx| {
                                        this.set_task_intent(id.clone(), Intent::Pause, cx)
                                    }
                                })),
                        )
                        .child(
                            control(SharedString::from(format!("cancel-{id}")))
                                .label("取消任务")
                                .on_click(cx.listener({
                                    let id = id.clone();
                                    move |this, _, _, cx| {
                                        this.set_task_intent(id.clone(), Intent::Cancel, cx)
                                    }
                                })),
                        );
                }
                if task.handled_by.is_none()
                    && matches!(task.state, TaskState::Paused | TaskState::NeedsAttention)
                    && uncertain.is_empty()
                {
                    actions = actions.child(
                        control(SharedString::from(format!("resume-{id}")))
                            .label("继续任务")
                            .on_click(cx.listener({
                                let id = id.clone();
                                move |this, _, _, cx| {
                                    this.set_task_intent(id.clone(), Intent::Run, cx)
                                }
                            })),
                    );
                }
                if task.handled_by.is_none()
                    && !matches!(
                        task.state,
                        TaskState::Running | TaskState::Pausing | TaskState::Queued
                    )
                {
                    actions = actions.child(
                        control(SharedString::from(format!("adjust-{id}")))
                            .label(if task.state == TaskState::Complete {
                                "调整并生成新版"
                            } else {
                                "调整后重试"
                            })
                            .on_click(cx.listener({
                                let id = id.clone();
                                move |this, _, window, cx| this.adjust_task(id.clone(), window, cx)
                            })),
                    );
                }
                if let Some(path) = &task.artifact {
                    for (component, label, outcome) in task_component_outcomes(task, path) {
                        if !matches!(
                            outcome.status,
                            course2md::artifact::Status::Failed
                                | course2md::artifact::Status::Partial
                        ) {
                            continue;
                        }
                        let unknown_component = uncertain.iter().any(|request| {
                            let purpose = request.purpose.as_deref().unwrap_or_default();
                            (component == "proofreading" && purpose.contains("proof"))
                                || (component == "summary" && purpose.contains("summary"))
                        });
                        if unknown_component {
                            continue;
                        }
                        let reason =
                            activity::component_failure_message(&label, outcome.message.as_deref());
                        card = card.child(
                            accessible_text(
                                SharedString::from(format!("outcome-{id}-{component}")),
                                reason,
                            )
                            .text_sm(),
                        );
                        if uncertain.is_empty() && task.handled_by.is_none() {
                            let task_id = id.clone();
                            actions = actions.child(
                                control(SharedString::from(format!("retry-{id}-{component}")))
                                    .label(format!("仅补{label}"))
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.reprocess_task(
                                            task_id.clone(),
                                            vec![component.clone()],
                                            Vec::new(),
                                            cx,
                                        )
                                    })),
                            );
                        }
                    }
                    let path = path.clone();
                    actions = actions.child(
                        control(SharedString::from(format!("open-{id}")))
                            .primary()
                            .label("阅读笔记")
                            .on_click(cx.listener(move |this, _, _, cx| {
                                let done = Completed {
                                    out_dir: path.clone(),
                                    title: task_title(&path),
                                    slides: 0,
                                    segments: 0,
                                    ..Default::default()
                                };
                                this.open_course(Course::from_completed(&done), cx);
                            })),
                    );
                }
                card = card.child(actions);
                if !task.logs.is_empty() || task.error.is_some() || !uncertain.is_empty() {
                    card = card.child(
                        control(SharedString::from(format!("logs-{id}")))
                            .ghost()
                            .self_start()
                            .label(if self.show_logs {
                                "收起技术详情"
                            } else {
                                "技术详情"
                            })
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.show_logs = !this.show_logs;
                                cx.notify();
                            })),
                    );
                    if self.show_logs {
                        card = card.child(
                            accessible_text(
                                SharedString::from(format!("log-text-{id}")),
                                task.logs
                                    .iter()
                                    .chain(task.error.iter())
                                    .chain(uncertain.iter().map(|request| &request.message))
                                    .cloned()
                                    .collect::<Vec<_>>()
                                    .join("\n"),
                            )
                            .text_sm(),
                        );
                    }
                }
            }
            content = content.child(card);
        }
        if let Some(historical) = historical_content {
            content = content.child(historical);
        }
        content.into_any_element()
    }

    /// Stage facts shared by the queue page and the workbench task card.
    fn stage_detail_lines(&self, task: &TaskRecord) -> Vec<(SharedString, String)> {
        let mut stages: Vec<_> = task.stages.iter().collect();
        stages.sort_by_key(|(name, _)| task_stage_order(name));
        stages
            .into_iter()
            .map(|(stage, value)| {
                let detail = if value.status == "done" {
                    "已完成".into()
                } else if stage.starts_with("scenes/") {
                    activity::quantity(stage, value.current, value.total)
                } else if value.total > 0 {
                    format!("{} / {}", value.current, value.total)
                } else if self.active_task.as_deref() != Some(&task.id) || self.job.is_none() {
                    "尚未完成，进度已保留".into()
                } else {
                    value.detail.clone().unwrap_or_else(|| "正在处理".into())
                };
                (
                    SharedString::from(format!("stage-{}-{stage}", task.id)),
                    format!("{}：{detail}", activity::title(stage)),
                )
            })
            .collect()
    }

    /// Uncertain-outcome block with the explicit resend action; shared by the
    /// queue page and the workbench attention card.
    fn uncertain_block(&self, task: &TaskRecord, cx: &mut Context<Self>) -> Option<Div> {
        let id = task.id.clone();
        let uncertain: Vec<_> = task
            .blocked
            .iter()
            .filter(|b| b.reason == "uncertain")
            .collect();
        if uncertain.is_empty() || task.handled_by.is_some() {
            return None;
        }
        let active = self.active_task.as_deref() == Some(&id) && self.job.is_some();
        Some(
            v_flex()
                .gap_2()
                .p_3()
                .rounded(RADIUS_CARD)
                .bg(rgb(WARNING_BG))
                .children(uncertain.iter().enumerate().map(|(index, blocked)| {
                    accessible_text(
                        SharedString::from(format!("unknown-scope-{id}-{index}")),
                        request_scope(blocked),
                    )
                    .text_sm()
                }))
                .child(accessible_text(
                    SharedString::from(format!("unknown-effect-{id}")),
                    "服务可能已处理这些内容，再次提交可能产生额外费用。其他进度已保留。",
                ))
                .when(active, |view| {
                    view.child(accessible_text(
                        SharedString::from(format!("unknown-saving-{id}")),
                        "正在保存当前结果，完成后可以选择重新发送。",
                    ))
                })
                .child(
                    primary_pill(SharedString::from(format!("resend-{id}")))
                        .self_start()
                        .label("重新发送以上内容")
                        .disabled(active)
                        .on_click(cx.listener({
                            let id = id.clone();
                            move |this, _, _, cx| this.resend_uncertain(id.clone(), cx)
                        })),
                ),
        )
    }

    fn work_progress_bar(fraction: f32) -> Div {
        div()
            .h(px(6.))
            .w_full()
            .rounded_full()
            .bg(rgb(PROGRESS_TRACK))
            .child(
                div()
                    .h_full()
                    .w(relative(fraction))
                    .rounded_full()
                    .bg(rgb(PROGRESS_FILL)),
            )
    }

    /// Workbench box card for a queued/running task: live worker numbers, real
    /// denominators only; unknown totals show the estimating label.
    pub fn box_task_running(&mut self, task: &TaskRecord, cx: &mut Context<Self>) -> Div {
        let id = task.id.clone();
        let active = self.active_task.as_deref() == Some(&id) && self.job.is_some();
        let queued = task.state == TaskState::Queued;
        let mut card = crate::import_ui::box_section("本次任务")
            .gap_3()
            .child(
                h_flex()
                    .gap_2()
                    .items_start()
                    .child(
                        badge(if queued {
                            BadgeKind::Neutral
                        } else {
                            BadgeKind::Progress
                        })
                        .child(task.state.label()),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .whitespace_normal()
                            .text_ellipsis()
                            .line_clamp(2)
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(task.plan.title.clone()),
                    ),
            );
        let mut meta = format!(
            "{} 开始",
            crate::reader_navigation::timestamp_utc(task.created * 1000)
        );
        if (task.plan.options.llm || task.plan.options.summarize)
            && let Some(version) = task
                .plan
                .ai_service
                .as_ref()
                .and_then(|id| self.preferences.version(id))
        {
            meta.push_str(&format!(
                " · AI 结果发送到「{}」（{}）",
                version.config.name,
                version.config.host()
            ));
        }
        card = card.child(div().text_sm().text_color(rgb(GRAY)).child(meta));
        let mut work = v_flex().gap_3();
        if active {
            if self.progress.is_empty() {
                work = work.child(
                    div()
                        .text_sm()
                        .text_color(rgb(GRAY))
                        .child("正在启动任务…"),
                );
            }
            let mut stages: Vec<_> = self.progress.iter().collect();
            stages.sort_by_key(|(stage, _)| activity::stage_order(stage));
            for (stage, item) in stages {
                if item.done {
                    work = work.child(
                        h_flex()
                            .gap_2()
                            .items_baseline()
                            .child(div().text_color(rgb(SUCCESS)).child("✓"))
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(GRAY))
                                    .child(activity::title(stage)),
                            ),
                    );
                    continue;
                }
                let mut row = v_flex().gap_2();
                row = row.child(
                    h_flex()
                        .gap_3()
                        .items_baseline()
                        .child(
                            div()
                                .font_weight(FontWeight::SEMIBOLD)
                                .flex_shrink_0()
                                .child(activity::title(stage)),
                        )
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .text_sm()
                                .text_color(rgb(GRAY))
                                .child(item.detail(stage, true)),
                        ),
                );
                if let Some(fraction) = item.fraction() {
                    row = row.child(Self::work_progress_bar(fraction));
                }
                work = work.child(row);
            }
        } else {
            let fact = if queued {
                let waiting = self
                    .workspace
                    .as_ref()
                    .and_then(|w| w.state.tasks.iter().position(|other| other.id == id))
                    .map(|position| {
                        self.workspace
                            .as_ref()
                            .map(|w| {
                                w.state.tasks[..position]
                                    .iter()
                                    .filter(|other| !other.state.finished())
                                    .count()
                            })
                            .unwrap_or(0)
                    })
                    .unwrap_or(0);
                if waiting > 0 {
                    format!("等待处理 · 前面有 {waiting} 个任务")
                } else {
                    "等待处理".into()
                }
            } else {
                task.state.label().into()
            };
            work = work.child(div().text_sm().text_color(rgb(GRAY)).child(fact));
        }
        card = card.child(work);
        let mut actions = h_flex().gap_2().flex_wrap();
        if matches!(task.state, TaskState::Running | TaskState::Queued | TaskState::Pausing)
            || active
        {
            actions = actions
                .child(
                    outline_pill(SharedString::from(format!("box-pause-{id}")))
                        .icon(icons::pause())
                        .label("暂停生成")
                        .on_click(cx.listener({
                            let id = id.clone();
                            move |this, _, _, cx| this.set_task_intent(id.clone(), Intent::Pause, cx)
                        })),
                )
                .child(
                    quiet(SharedString::from(format!("box-cancel-{id}")))
                        .label("取消任务")
                        .on_click(cx.listener({
                            let id = id.clone();
                            move |this, _, _, cx| {
                                this.set_task_intent(id.clone(), Intent::Cancel, cx)
                            }
                        })),
                );
        }
        card = card.child(actions).child(
            div()
                .text_size(TEXT_AUX)
                .text_color(rgb(GRAY))
                .child("关闭窗口后任务会继续；退出应用会暂停任务。"),
        );
        let more = self
            .workspace
            .as_ref()
            .map(|w| {
                w.state
                    .tasks
                    .iter()
                    .filter(|other| other.id != id && !other.state.finished())
                    .count()
            })
            .unwrap_or(0);
        if active && more > 0 {
            card = card.child(
                div()
                    .text_size(TEXT_AUX)
                    .text_color(rgb(GRAY))
                    .child(format!("队列中还有 {more} 个任务等待处理。")),
            );
        }
        card
    }

    /// Workbench box card for a task that needs the user: paused, failed or
    /// uncertain outcomes, each with its real recovery action.
    pub fn box_task_attention(&mut self, task: &TaskRecord, cx: &mut Context<Self>) -> Div {
        let id = task.id.clone();
        let kind = match task.state {
            TaskState::Paused | TaskState::Pausing => BadgeKind::Neutral,
            _ => BadgeKind::Warning,
        };
        let uncertain = task
            .blocked
            .iter()
            .any(|request| request.reason == "uncertain")
            && task.handled_by.is_none();
        // 部分完成：正文已发布，逐项标明未完成的附加处理并可只补做该项。
        let partial_failures: Vec<(String, String, String)> = if task.state == TaskState::Partial {
            task.artifact
                .as_ref()
                .map(|path| {
                    task_component_outcomes(task, path)
                        .into_iter()
                        .map(|(component, label, outcome)| {
                            let reason = activity::component_failure_message(
                                &label,
                                outcome.message.as_deref(),
                            );
                            (component, label, reason)
                        })
                        .collect()
                })
                .unwrap_or_default()
        } else {
            Vec::new()
        };
        let mut card = crate::import_ui::box_section("本次任务")
            .gap_3()
            .child(
                h_flex()
                    .gap_2()
                    .items_start()
                    .child(badge(kind).child(task.state.label()))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .whitespace_normal()
                            .text_ellipsis()
                            .line_clamp(2)
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(task.plan.title.clone()),
                    ),
            );
        if let Some(error) = &task.error
            && !uncertain
        {
            card = card.child(
                accessible_text(SharedString::from(format!("box-error-{id}")), error.clone())
                    .role(Role::Alert)
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(rgb(DANGER)),
            );
        }
        for (component, _, reason) in &partial_failures {
            card = card.child(
                accessible_text(
                    SharedString::from(format!("box-partial-{id}-{component}")),
                    reason.clone(),
                )
                .text_sm(),
            );
        }
        for (text_id, line) in self.stage_detail_lines(task) {
            card = card.child(
                div()
                    .text_sm()
                    .text_color(rgb(GRAY))
                    .child(accessible_text(text_id, line)),
            );
        }
        if let Some(block) = self.uncertain_block(task, cx) {
            card = card.child(block);
        }
        let mut actions = h_flex().gap_2().flex_wrap();
        if !partial_failures.is_empty() {
            for (index, (component, label, _)) in partial_failures.iter().enumerate() {
                let component = component.clone();
                let button = if index == 0 {
                    primary_pill(SharedString::from(format!("box-retry-{id}-{component}")))
                        .self_start()
                } else {
                    outline_pill(SharedString::from(format!("box-retry-{id}-{component}")))
                };
                actions = actions.child(
                    button
                        .label(format!("仅补{label}"))
                        .on_click(cx.listener({
                            let id = id.clone();
                            move |this, _, _, cx| {
                                this.reprocess_task(
                                    id.clone(),
                                    vec![component.clone()],
                                    Vec::new(),
                                    cx,
                                )
                            }
                        })),
                );
            }
            if let Some(path) = &task.artifact {
                let path = path.clone();
                actions = actions.child(
                    outline_pill(SharedString::from(format!("box-read-{id}")))
                        .label("阅读笔记")
                        .on_click(cx.listener(move |this, _, _, cx| {
                            let done = Completed {
                                out_dir: path.clone(),
                                title: task_title(&path),
                                ..Default::default()
                            };
                            this.open_course(Course::from_completed(&done), cx);
                        })),
                );
            }
        } else if !uncertain {
            actions = actions.child(
                primary_pill(SharedString::from(format!("box-resume-{id}")))
                    .self_start()
                    .label(if task.state == TaskState::Paused {
                        "继续生成"
                    } else {
                        "继续任务"
                    })
                    .on_click(cx.listener({
                        let id = id.clone();
                        move |this, _, _, cx| this.set_task_intent(id.clone(), Intent::Run, cx)
                    })),
            );
        }
        if partial_failures.is_empty() {
            if let Some(library) = self
                .workspace
                .as_ref()
                .and_then(|workspace| workspace.state.library(&task.plan.library_id))
                && library.root.is_dir()
                && !library.root.join(".course2md-library-id").exists()
            {
                let library_id = library.id.clone();
                actions = actions.child(
                    outline_pill(SharedString::from(format!("box-reassociate-{id}")))
                        .label("重新关联此保存位置")
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.begin_library_reassociation(library_id.clone(), window, cx)
                        })),
                );
            }
            actions = actions
                .child(
                    outline_pill(SharedString::from(format!("box-adjust-{id}")))
                        .label("调整后重试")
                        .on_click(cx.listener({
                            let id = id.clone();
                            move |this, _, window, cx| this.adjust_task(id.clone(), window, cx)
                        })),
                )
                .child(
                    quiet(SharedString::from(format!("box-cancel-{id}")))
                        .label("取消任务")
                        .on_click(cx.listener({
                            let id = id.clone();
                            move |this, _, _, cx| this.set_task_intent(id.clone(), Intent::Cancel, cx)
                        })),
                );
        }
        card = card.child(actions);
        if !task.logs.is_empty() || task.error.is_some() || uncertain || !partial_failures.is_empty() {
            card = card
                .child(
                    quiet(SharedString::from(format!("box-logs-{id}")))
                        .self_start()
                        .label(if self.show_logs {
                            "收起技术详情"
                        } else {
                            "技术详情"
                        })
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.show_logs = !this.show_logs;
                            cx.notify();
                        })),
                );
            if self.show_logs {
                card = card.child(
                    accessible_text(
                        SharedString::from(format!("box-log-text-{id}")),
                        task.logs
                            .iter()
                            .chain(task.error.iter())
                            .cloned()
                            .collect::<Vec<_>>()
                            .join("\n"),
                    )
                    .text_sm()
                    .whitespace_normal(),
                );
            }
        }
        card
    }

}

fn task_title(path: &std::path::Path) -> String {
    course2md::artifact::read_manifest(&path.join("manifest.json"))
        .map(|m| m.title)
        .unwrap_or_else(|_| {
            path.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned()
        })
}

impl ConversionOptions {
    pub fn apply_to(&self, config: &mut course2md::settings::ConfigFile) {
        use course2md::config::{AsrProvider, OutputFormat, TranscriptSource};
        config.defaults.provider = match self.provider {
            1 => Some(AsrProvider::Coreml),
            2 => Some(AsrProvider::Gpu),
            3 => Some(AsrProvider::Cpu),
            4 => Some(AsrProvider::Npu),
            5 => Some(AsrProvider::Api),
            _ => None,
        };
        config.defaults.transcript_source = Some(match self.source_mode {
            1 => TranscriptSource::Subtitle,
            2 => TranscriptSource::Asr,
            _ => TranscriptSource::Auto,
        });
        config.defaults.keep_video = Some(self.keep_video);
        config.defaults.resume = Some(true);
        config.defaults.formats = Some(
            [OutputFormat::Md, OutputFormat::Html, OutputFormat::Json]
                .into_iter()
                .zip(self.formats)
                .filter_map(|(f, selected)| selected.then_some(f))
                .collect(),
        );
        config.llm.enabled = self.llm;
        config.llm.summarize = self.summarize;
        config.llm.vision = self.llm && self.vision;
        config.llm.disable_hint = true;
    }
}

fn task_status_label(task: &TaskRecord) -> String {
    if task.handled_by.is_some() {
        return "已有后续处理任务".into();
    }
    if task
        .blocked
        .iter()
        .any(|request| request.reason == "uncertain")
    {
        let purpose: std::collections::BTreeSet<_> = task
            .blocked
            .iter()
            .filter(|request| request.reason == "uncertain")
            .filter_map(|request| request.purpose.as_deref())
            .collect();
        let description = if !purpose.is_empty()
            && purpose.iter().all(|purpose| purpose.contains("proof"))
        {
            "AI 校对结果未确认"
        } else if !purpose.is_empty() && purpose.iter().all(|purpose| purpose.contains("summary")) {
            "摘要结果未确认"
        } else if purpose
            .iter()
            .any(|purpose| purpose.contains("transcription"))
        {
            "语音识别结果未确认"
        } else {
            "部分处理结果未确认"
        };
        return if task.artifact.is_some() {
            format!("笔记可读，{description}")
        } else {
            description.into()
        };
    }
    if matches!(&task.plan.operation, course2md::execution::Operation::Reprocess { components, .. } if components.iter().all(|part| part == "exports"))
        && task.state == TaskState::Complete
    {
        "文件已导出".into()
    } else {
        task.state.label().into()
    }
}

fn request_scope(request: &workspace::BlockedRequest) -> String {
    if !request.description.trim().is_empty() {
        request.description.clone()
    } else {
        let purpose = request.purpose.as_deref().unwrap_or_default();
        let name = if purpose.contains("proof") {
            "AI 校对"
        } else if purpose.contains("summary") {
            "生成摘要"
        } else if purpose.contains("transcription") {
            "语音识别"
        } else {
            "外部处理"
        };
        format!("{name}：此历史请求未保存具体内容范围，可在技术详情中查看请求记录。")
    }
}

fn task_stage_order(stage: &str) -> usize {
    activity::stage_order(stage)
}

fn validate_plan_config(source: &str, config: &course2md::settings::ConfigFile) -> Result<()> {
    let resolved = course2md::options::resolve(source.to_owned(), &Default::default(), config)?;
    resolved.validate()?;
    if resolved.transcript_source != course2md::config::TranscriptSource::Subtitle {
        // Credentials belong to the service vault, not this static model/provider check.
        resolved.validate_asr_with_auth(false, false)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{update_draft_source_title, validate_plan_config};

    fn source(input: &str, title: &str) -> crate::source::Source {
        crate::source::Source {
            input: input.into(),
            title: title.into(),
            identity: format!("online:{input}"),
            online: true,
            ..Default::default()
        }
    }

    #[test]
    fn changing_source_keeps_automatic_title_provenance_and_frozen_task() {
        use crate::workspace::{TaskPlan, Workspace};
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("workspace.json");
        let root = dir.path().join("library");
        let mut workspace =
            Workspace::open_at(file.clone(), root.clone(), Default::default()).unwrap();
        let a = source("https://example.test/video-a", "课程 A");
        let b = source("https://example.test/video-b", "课程 B");
        let id = workspace
            .transaction(|state| {
                update_draft_source_title(
                    state.draft_mut().unwrap(),
                    a.input.clone(),
                    a.title.clone(),
                    Some(a.clone()),
                );
                state.enqueue(
                    TaskPlan {
                        operation: Default::default(),
                        source: a.clone(),
                        source_id: a.identity.clone(),
                        title: a.title.clone(),
                        library_id: state.default_library.clone(),
                        folder: None,
                        options: Default::default(),
                        subtitle: None,
                        config: Default::default(),
                        asr_service: None,
                        ai_service: None,
                    },
                    None,
                )
            })
            .unwrap()
            .0;
        // The visible title input still contains A until B's metadata arrives.
        // More than one autosave, and a restart in between, must not make it custom.
        for _ in 0..3 {
            workspace
                .transaction(|state| {
                    update_draft_source_title(
                        state.draft_mut().unwrap(),
                        b.input.clone(),
                        a.title.clone(),
                        None,
                    );
                    Ok(())
                })
                .unwrap();
        }
        workspace = Workspace::open_at(file.clone(), root.clone(), Default::default()).unwrap();
        assert!(!workspace.state.draft().unwrap().custom_title);
        assert!(workspace.state.draft().unwrap().source.is_none());
        workspace
            .transaction(|state| {
                update_draft_source_title(
                    state.draft_mut().unwrap(),
                    b.input.clone(),
                    b.title.clone(),
                    Some(b.clone()),
                );
                Ok(())
            })
            .unwrap();
        let restored = Workspace::open_at(file, root, Default::default()).unwrap();
        let draft = restored.state.draft().unwrap();
        assert_eq!(draft.title, "课程 B");
        assert!(!draft.custom_title);
        assert_eq!(draft.source.as_ref().unwrap(), &b);
        let task = restored.state.task(&id).unwrap();
        assert_eq!(task.plan.source, a);
        assert_eq!(task.plan.title, "课程 A");
    }

    #[test]
    fn manual_title_survives_a_source_change_and_late_metadata() {
        let mut draft = crate::workspace::Draft::new(true, "library".into(), Default::default());
        let a = source("https://example.test/a", "课程 A");
        let b = source("https://example.test/b", "课程 B");
        update_draft_source_title(&mut draft, a.input.clone(), a.title.clone(), Some(a));
        // Source and title may both change before the autosave deadline.
        update_draft_source_title(&mut draft, b.input.clone(), "我的复习笔记".into(), None);
        assert!(draft.custom_title);
        let revision = draft.revision;
        assert!(draft.accept_source(revision, b));
        assert_eq!(draft.title, "我的复习笔记");
        assert!(draft.custom_title);
    }

    #[test]
    fn submit_validates_the_selected_model_only_when_speech_recognition_is_needed() {
        use course2md::config::{AsrProvider, TranscriptSource};
        let mut config = course2md::settings::ConfigFile::default();
        config.defaults.provider = Some(AsrProvider::Cpu);
        config.defaults.asr_model = Some("whisper".into());
        config.defaults.transcript_source = Some(TranscriptSource::Asr);
        assert!(validate_plan_config("video.mp4", &config).is_err());
        config.defaults.transcript_source = Some(TranscriptSource::Subtitle);
        validate_plan_config("video.mp4", &config).unwrap();
        config.defaults.provider = Some(AsrProvider::Coreml);
        config.defaults.asr_model = Some("qwen3-0.6b".into());
        config.defaults.transcript_source = Some(TranscriptSource::Asr);
        validate_plan_config("video.mp4", &config).unwrap();
    }
}
pub(crate) fn task_component_outcomes(
    task: &TaskRecord,
    path: &std::path::Path,
) -> Vec<(String, String, course2md::artifact::Outcome)> {
    let value = task.outcomes.clone().or_else(|| {
        course2md::artifact::read_manifest(&path.join("manifest.json"))
            .ok()
            .and_then(|m| serde_json::to_value(m.outcomes).ok())
    });
    let Some(value) = value else {
        return Vec::new();
    };
    let mut results = Vec::new();
    for (key, label) in [
        ("screenshots", "截图"),
        ("proofreading", "校对"),
        ("summary", "摘要"),
    ] {
        if let Some(outcome) = value
            .get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
        {
            results.push((key.into(), label.into(), outcome));
        }
    }
    if let Some(exports) = value.get("exports").and_then(|v| v.as_object()) {
        let failures: Vec<course2md::artifact::Outcome> = exports
            .values()
            .filter_map(|v| serde_json::from_value(v.clone()).ok())
            .filter(|o: &course2md::artifact::Outcome| {
                matches!(
                    o.status,
                    course2md::artifact::Status::Failed | course2md::artifact::Status::Partial
                )
            })
            .collect();
        if !failures.is_empty() {
            results.push((
                "exports".into(),
                "导出".into(),
                course2md::artifact::Outcome::failed(
                    failures
                        .iter()
                        .filter_map(|o| o.message.clone())
                        .collect::<Vec<_>>()
                        .join("；"),
                ),
            ));
        }
    }
    results
}
