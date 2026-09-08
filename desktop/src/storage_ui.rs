//! Controlled library migration and explicit cleanup of verified old-location backups.
use super::*;
use crate::{
    storage::{self, Journal, Phase, Progress},
    theme::*,
};
use anyhow::{Context as _, Result, ensure};
use gpui_component::button::*;
use std::{
    path::Path,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

#[derive(Default)]
pub struct State {
    pub busy: bool,
    cancel: Option<Arc<AtomicBool>>,
    generation: u64,
    progress: Progress,
    error: Option<String>,
    pending: Vec<(PathBuf, Journal)>,
    resume_task: Option<String>,
    cleanup: bool,
    association: bool,
}

impl State {
    fn title(&self) -> &'static str {
        if self.association {
            "重新关联保存位置"
        } else if self.cleanup {
            "清理旧位置备份"
        } else {
            "移动课程库"
        }
    }
}

struct StorageDialog {
    desktop: Entity<Desktop>,
    _observation: Subscription,
}
impl Render for StorageDialog {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.desktop.update(cx, |desktop, cx| {
            div()
                .id("storage-dialog-content")
                .role(Role::Dialog)
                .aria_label(desktop.storage_ui.title())
                .child(desktop.storage_operation_view(window, cx))
        })
    }
}

fn relocate_source(source: &mut source::Source, old: &Path, new: &Path) {
    if !source.online {
        let mut path = PathBuf::from(&source.input);
        storage::relocate_path(&mut path, old, new);
        source.input = path.display().to_string();
    }
    if let Some(path) = &mut source.cover {
        storage::relocate_path(path, old, new);
    }
    if let Some(subtitle) = &mut source.selected_subtitle {
        storage::relocate_path(&mut subtitle.path, old, new);
    }
    fn track(track: &mut course2md::subtitle::SubtitleTrack, old: &Path, new: &Path) {
        if let course2md::subtitle::SubtitleOrigin::File { path } = &mut track.origin {
            storage::relocate_path(path, old, new);
            // Track identity refers to the original confirmed choice. Its content
            // is stored in the task; relocating a file does not reselect a track.
        }
    }
    if let course2md::subtitle::SubtitleEvidence::Found { tracks, .. } = &mut source.subtitles {
        for item in tracks {
            track(item, old, new);
        }
    }
    if let Some(item) = &mut source.subtitle_request {
        track(item, old, new);
    }
}

fn relocate_config(config: &mut course2md::settings::ConfigFile, old: &Path, new: &Path) {
    if let Some(path) = &mut config.defaults.out {
        storage::relocate_path(path, old, new);
    }
    if let Some(path) = &mut config.defaults.model_dir {
        storage::relocate_path(path, old, new);
    }
}

fn validate_registered_destination(
    state: &workspace::State,
    library_id: &str,
    destination: &Path,
) -> Result<()> {
    let destination = std::fs::canonicalize(destination).context("目标位置暂时无法访问")?;
    ensure!(
        !state
            .libraries
            .iter()
            .filter(|library| library.id != library_id)
            .any(|library| {
                let root =
                    std::fs::canonicalize(&library.root).unwrap_or_else(|_| library.root.clone());
                root.starts_with(&destination) || destination.starts_with(root)
            }),
        "目标与另一个已登记课程库重叠，请选择独立的空文件夹"
    );
    Ok(())
}

pub fn needs_reassociation(location: &workspace::LibraryLocation) -> bool {
    location.root.is_dir()
        && std::fs::symlink_metadata(location.root.join(".course2md-library-id"))
            .is_err_and(|error| error.kind() == std::io::ErrorKind::NotFound)
}

/// Apply only after the destination copy is verified. Keeping the library ID
/// makes folder IDs, reading positions and all service bindings stable.
fn publish_location(
    state: &mut workspace::State,
    journal: &Journal,
    journal_path: &Path,
) -> Result<()> {
    ensure!(
        journal.phase == Phase::Verified,
        "目标副本尚未核验，不能切换保存位置"
    );
    let location = state
        .libraries
        .iter()
        .find(|library| library.id == journal.library_id)
        .context("原课程库已不在已登记位置中")?;
    ensure!(
        std::fs::canonicalize(&location.root)? == journal.source,
        "课程库的位置在迁移期间已经变化，尚未切换"
    );
    validate_registered_destination(state, &journal.library_id, &journal.destination)?;
    let roots = [journal.source.clone(), location.root.clone()];
    for old in &roots {
        for path in state.reader_sources.values_mut() {
            storage::relocate_path(path, old, &journal.destination);
        }
        for draft in &mut state.drafts {
            if let Some(source) = &mut draft.source {
                let before = source.input.clone();
                relocate_source(source, old, &journal.destination);
                if !source.online && before != source.input {
                    draft.input = source.input.clone();
                }
            } else if !draft.online {
                let mut path = PathBuf::from(&draft.input);
                storage::relocate_path(&mut path, old, &journal.destination);
                draft.input = path.display().to_string();
            }
            if let Some(path) = &mut draft.subtitle {
                storage::relocate_path(path, old, &journal.destination);
            }
            if let Some(config) = &mut draft.base_config {
                relocate_config(config, old, &journal.destination);
            }
        }
        for task in &mut state.tasks {
            storage::relocate_path(&mut task.work_dir, old, &journal.destination);
            if let Some(path) = &mut task.artifact {
                storage::relocate_path(path, old, &journal.destination);
            }
            if let Some(path) = &mut task.plan.subtitle {
                storage::relocate_path(path, old, &journal.destination);
            }
            relocate_source(&mut task.plan.source, old, &journal.destination);
            relocate_config(&mut task.plan.config, old, &journal.destination);
            if let course2md::execution::Operation::Reprocess {
                base_version_dir,
                prior_work_dir,
                ..
            } = &mut task.plan.operation
            {
                storage::relocate_path(base_version_dir, old, &journal.destination);
                if let Some(path) = prior_work_dir {
                    storage::relocate_path(path, old, &journal.destination);
                }
            }
        }
    }
    let location = state
        .libraries
        .iter_mut()
        .find(|library| library.id == journal.library_id)
        .unwrap();
    for old in roots {
        if !location.previous_roots.contains(&old) {
            location.previous_roots.push(old);
        }
    }
    location.root = journal.destination.clone();
    for backup in &mut state.storage_backups {
        if backup.library_id == journal.library_id {
            backup.current_root = journal.destination.clone();
        }
    }
    state.storage_backups.push(storage::BackupRecord {
        id: journal.id.clone(),
        library_id: journal.library_id.clone(),
        path: journal.source.clone(),
        current_root: journal.destination.clone(),
        created: journal.created,
        verified: true,
        journal_path: journal_path.to_owned(),
    });
    Ok(())
}

impl Desktop {
    pub fn begin_library_reassociation(
        &mut self,
        library_id: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.storage_ui.busy {
            return;
        }
        let Some(location) = self
            .workspace
            .as_ref()
            .and_then(|workspace| workspace.state.library(&library_id))
            .cloned()
        else {
            return;
        };
        let courses = self
            .courses
            .iter()
            .filter(|course| {
                self.course_location(course)
                    .is_some_and(|library| library.id == library_id)
            })
            .cloned()
            .collect::<Vec<_>>();
        self.storage_ui.busy = true;
        self.storage_ui.association = true;
        self.storage_ui.cleanup = false;
        self.storage_ui.error = None;
        self.storage_ui.progress = Progress {
            message: "正在重新读取这个位置中的笔记…".into(),
            ..Default::default()
        };
        let cancel = Arc::new(AtomicBool::new(false));
        self.storage_ui.cancel = Some(cancel.clone());
        self.open_storage_dialog(window, cx);
        let root = location.root.clone();
        let worker_cancel = cancel.clone();
        let worker = cx.background_executor().spawn(async move {
            let stamp = storage::directory_stamp(&root)?;
            let mut names = Vec::new();
            let mut unreadable = 0;
            for course in courses {
                ensure!(
                    !worker_cancel.load(Ordering::Relaxed),
                    "已取消重新关联，原文件保持完整。"
                );
                let title = course.title.clone();
                if crate::notes::read_preview(course).is_ok() {
                    names.push(title);
                } else {
                    unreadable += 1;
                }
            }
            ensure!(
                storage::directory_stamp(&root)? == stamp,
                "保存位置在读取期间已经变化，请重新检查。"
            );
            ensure!(
                !worker_cancel.load(Ordering::Relaxed),
                "已取消重新关联，原文件保持完整。"
            );
            Ok::<_, anyhow::Error>((stamp, names, unreadable))
        });
        cx.spawn_in(window, async move |this, cx| {
            let result = worker.await;
            let Ok((stamp, names, unreadable)) = result else {
                let message = result.unwrap_err().to_string();
                let _ = this.update_in(cx, |this, _, cx| {
                    this.storage_ui.busy = false;
                    this.storage_ui.cancel = None;
                    this.storage_ui.progress = Progress::default();
                    this.storage_ui.error = Some(message);
                    this.start_next_task(cx);
                    cx.notify();
                });
                return;
            };
            let mut detail = format!("保存位置：{}\n\n", location.root.display());
            if names.is_empty() {
                detail.push_str("目前未读到已列入课程库的笔记。现有文件会全部保留。\n");
            } else {
                detail.push_str(&format!(
                    "目前可读取 {} 份笔记：\n{}",
                    names.len(),
                    names
                        .iter()
                        .take(5)
                        .map(|name| format!("• {name}"))
                        .collect::<Vec<_>>()
                        .join("\n")
                ));
                if names.len() > 5 {
                    detail.push_str(&format!("\n以及另外 {} 份笔记。", names.len() - 5));
                }
                detail.push('\n');
            }
            if unreadable > 0 {
                detail.push_str(&format!(
                    "另有 {unreadable} 份已列出的笔记暂时无法读取，文件会保留。\n"
                ));
            }
            detail.push_str("\n关联后继续使用这个文件夹，现有笔记、原视频、草稿和任务都会保留。");
            let answer = this.update_in(cx, |this, window, cx| {
                this.storage_ui.cancel = None;
                window.close_dialog(cx);
                window.prompt(
                    PromptLevel::Info,
                    "重新关联此保存位置？",
                    Some(&detail),
                    &["重新关联", "取消"],
                    cx,
                )
            });
            let Ok(answer) = answer else {
                return;
            };
            let accepted = answer.await.ok() == Some(0);
            let _ = this.update_in(cx, |this, window, cx| {
                this.storage_ui.busy = false;
                this.storage_ui.progress = Progress::default();
                this.storage_ui.cancel = None;
                if !accepted {
                    this.start_next_task(cx);
                    cx.notify();
                    return;
                }
                let result = (|| -> Result<()> {
                    let workspace = this
                        .workspace
                        .as_mut()
                        .context("课程库登记记录暂时不可用")?;
                    let current = workspace
                        .state
                        .library(&library_id)
                        .context("课程库已不在登记位置中")?;
                    ensure!(
                        current.root == location.root
                            && storage::directory_stamp(&current.root)? == stamp,
                        "保存位置在确认期间已经变化，尚未重新关联。请重新检查这个位置。"
                    );
                    workspace.reassociate_library(&library_id)
                })();
                match result {
                    Ok(()) => {
                        this.storage_ui.error = None;
                        this.message = Some(format!(
                            "已重新关联「{}」。已有笔记保持完整，可以继续原任务。",
                            location.name
                        ));
                        this.refresh_library(cx);
                    }
                    Err(error) => {
                        this.storage_ui.error = Some(format!("尚未重新关联：{error:#}"));
                        this.open_storage_dialog(window, cx);
                    }
                }
                this.start_next_task(cx);
                cx.notify();
            });
        })
        .detach();
        cx.notify();
    }

    pub fn restore_storage_state(&mut self, cx: &mut Context<Self>) {
        self.storage_ui.pending =
            storage::pending_journals(&self.preferences.root().join("storage"));
        // If the registry transaction committed before the app closed, the
        // journal is merely behind; never offer to replay or undo that copy.
        self.storage_ui.pending.retain(|(path, journal)| {
            let committed = self.workspace.as_ref().is_some_and(|workspace| {
                workspace
                    .state
                    .library(&journal.library_id)
                    .is_some_and(|library| library.root == journal.destination)
            });
            if committed {
                let mut journal = journal.clone();
                journal.phase = Phase::Committed;
                let _ = storage::save_journal(path, &journal);
                false
            } else {
                true
            }
        });
        let pending_settings: Vec<_> = self
            .workspace
            .as_ref()
            .into_iter()
            .flat_map(|workspace| &workspace.state.storage_backups)
            .filter_map(|backup| {
                std::fs::read(&backup.journal_path)
                    .ok()
                    .and_then(|bytes| serde_json::from_slice::<Journal>(&bytes).ok())
                    .filter(|journal| {
                        !journal.settings_relocated
                            && journal.id == backup.id
                            && journal.source == backup.path
                            && journal.library_id == backup.library_id
                    })
                    .map(|journal| (backup.journal_path.clone(), journal))
            })
            .collect();
        for (path, mut journal) in pending_settings {
            match self.relocate_settings_paths(&journal.source, &journal.destination, cx) {
                Ok(()) => {
                    journal.settings_relocated = true;
                    if let Err(error) = storage::save_journal(&path, &journal) {
                        self.storage_ui.error =
                            Some(format!("模型位置已更新，迁移记录尚未保存：{error:#}"));
                    }
                }
                Err(error) => {
                    self.storage_ui.error = Some(format!(
                        "课程库已移动，模型位置尚未保存。旧位置备份已保留：{error:#}"
                    ))
                }
            }
        }
        if let Some(workspace) = &mut self.workspace {
            let completed: Vec<_> = workspace
                .state
                .storage_backups
                .iter()
                .filter(|backup| {
                    std::fs::read(&backup.journal_path)
                        .ok()
                        .and_then(|bytes| serde_json::from_slice::<Journal>(&bytes).ok())
                        .is_some_and(|journal| {
                            journal.cleanup_complete
                                && journal.id == backup.id
                                && journal.source == backup.path
                                && journal.library_id == backup.library_id
                        })
                })
                .map(|backup| backup.id.clone())
                .collect();
            if !completed.is_empty() {
                if let Err(error) = workspace.transaction(|state| {
                    state
                        .storage_backups
                        .retain(|backup| !completed.contains(&backup.id));
                    Ok(())
                }) {
                    self.storage_ui.error =
                        Some(format!("备份已清理，界面记录尚未保存：{error:#}"));
                }
            }
        }
        cx.notify();
    }

    pub fn poll_storage(&mut self, cx: &mut Context<Self>) {
        if !self.storage_ui.busy && self.job.is_none() {
            if let Some(id) = self.storage_ui.resume_task.take() {
                let can_resume = self
                    .workspace
                    .as_ref()
                    .and_then(|workspace| workspace.state.task(&id))
                    .is_some_and(|task| {
                        task.state == workspace::TaskState::Paused
                            && task.intent == workspace::Intent::Pause
                    });
                if can_resume {
                    self.set_task_intent(id, workspace::Intent::Run, cx);
                }
            }
        }
    }

    pub fn cancel_storage_for_close(&mut self) {
        if let Some(cancel) = &self.storage_ui.cancel {
            cancel.store(true, Ordering::Relaxed);
        }
    }

    pub fn begin_library_move(
        &mut self,
        library_id: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.storage_ui.busy {
            return;
        }
        self.storage_ui.association = false;
        let Some(location) = self
            .workspace
            .as_ref()
            .and_then(|workspace| workspace.state.library(&library_id))
            .cloned()
        else {
            return;
        };
        let prompt = cx.prompt_for_paths(PathPromptOptions {
            files: false,
            directories: true,
            multiple: false,
            prompt: Some("选择空文件夹作为课程库的新位置".into()),
        });
        cx.spawn_in(window, async move |this, cx| {
            let answer = prompt.await;
            let _ = this.update_in(cx, |this, window, cx| {
                match answer {
                    Ok(Ok(Some(paths))) => {
                        if let Some(destination) = paths.into_iter().next() {
                            match storage::validate_destination(&location.root, &destination) {
                                Ok((source, destination)) => this.start_library_move(
                                    location.id.clone(),
                                    source,
                                    destination,
                                    None,
                                    window,
                                    cx,
                                ),
                                Err(error) => {
                                    this.storage_ui.error = Some(format!("{error:#}"));
                                    this.storage_ui.cleanup = false;
                                    this.storage_ui.progress = Progress::default();
                                    this.open_storage_dialog(window, cx);
                                }
                            }
                        }
                    }
                    Ok(Ok(None)) => {}
                    Ok(Err(error)) => {
                        this.storage_ui.error = Some(format!("无法选择保存位置：{error:#}"))
                    }
                    Err(error) => this.storage_ui.error = Some(error.to_string()),
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn start_library_move(
        &mut self,
        library_id: String,
        source: PathBuf,
        destination: PathBuf,
        resume: Option<PathBuf>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.storage_ui.busy || !self.save_current_draft(cx) {
            return;
        }
        self.storage_ui.association = false;
        let valid = self
            .workspace
            .as_ref()
            .context("课程库登记记录暂时不可用")
            .and_then(|workspace| {
                validate_registered_destination(&workspace.state, &library_id, &destination)
            });
        if let Err(error) = valid {
            self.storage_ui.error = Some(format!("{error:#}"));
            self.storage_ui.progress = Progress::default();
            self.storage_ui.cleanup = false;
            self.open_storage_dialog(window, cx);
            return;
        }
        self.storage_ui.busy = true;
        self.storage_ui.cleanup = false;
        self.storage_ui.generation = self.storage_ui.generation.wrapping_add(1);
        let generation = self.storage_ui.generation;
        self.storage_ui.error = None;
        self.storage_ui.progress = Progress {
            message: "正在保存当前任务进度，完成后开始复制…".into(),
            completed: 0,
            total: 0,
        };
        let cancel = Arc::new(AtomicBool::new(false));
        self.storage_ui.cancel = Some(cancel.clone());
        if let Some(id) = self.active_task.clone() {
            let running = self
                .workspace
                .as_ref()
                .and_then(|workspace| workspace.state.task(&id))
                .is_some_and(|task| task.intent == workspace::Intent::Run);
            if running {
                self.storage_ui.resume_task = Some(id.clone());
                self.set_task_intent(id, workspace::Intent::Pause, cx);
            }
        } else if let Some(job) = &self.job {
            job.cancel();
        }
        self.open_storage_dialog(window, cx);
        let journal_dir = self.preferences.root().join("storage");
        cx.spawn_in(window, async move |this, cx| {
            let waiting = Instant::now();
            loop {
                if cancel.load(Ordering::Relaxed) || waiting.elapsed() > Duration::from_secs(30) {
                    let message = if cancel.load(Ordering::Relaxed) {
                        "已取消迁移，原保存位置未改变。"
                    } else {
                        "当前任务还在保存进度，迁移尚未开始。任务停止后可重新移动课程库。"
                    };
                    let _ = this.update_in(cx, |this, _, cx| {
                        this.finish_storage_error(message.into(), cx)
                    });
                    return;
                }
                let ready = this
                    .update_in(cx, |this, _, _| this.job.is_none())
                    .unwrap_or(false);
                if ready {
                    break;
                }
                smol::Timer::after(Duration::from_millis(100)).await;
            }
            let progress = Arc::new(Mutex::new(Progress::default()));
            let worker_progress = progress.clone();
            let worker_cancel = cancel.clone();
            let mut worker = cx.background_executor().spawn(async move {
                let report = |value| {
                    if let Ok(mut progress) = worker_progress.lock() {
                        *progress = value;
                    }
                };
                match resume {
                    Some(path) => storage::resume_move(&path, &worker_cancel, &report),
                    None => storage::prepare_move(
                        library_id,
                        &source,
                        &destination,
                        &journal_dir,
                        &worker_cancel,
                        &report,
                    ),
                }
            });
            let result = loop {
                if let Some(result) = smol::future::poll_once(&mut worker).await {
                    break result;
                }
                let snapshot = progress.lock().ok().map(|value| value.clone());
                if let Some(snapshot) = snapshot {
                    let _ = this.update_in(cx, |this, _, cx| {
                        this.storage_ui.progress = snapshot;
                        cx.notify();
                    });
                }
                smol::Timer::after(Duration::from_millis(100)).await;
            };
            let final_progress = progress
                .lock()
                .ok()
                .map(|value| value.clone())
                .unwrap_or_default();
            let _ = this.update_in(cx, |this, window, cx| {
                if this.storage_ui.generation != generation {
                    return;
                }
                this.storage_ui.progress = final_progress;
                match result {
                    Ok(mut prepared) => {
                        if this.closing || cancel.load(Ordering::Relaxed) {
                            this.finish_storage_error(
                                "迁移副本已核验，保存位置尚未切换，原文件未删除。".into(),
                                cx,
                            );
                            return;
                        }
                        let committed = this
                            .workspace
                            .as_mut()
                            .context("课程库登记记录暂时不可用")
                            .and_then(|workspace| {
                                workspace.transaction(|state| {
                                    publish_location(state, &prepared.journal, &prepared.path)
                                })
                            });
                        if let Err(error) = committed {
                            this.finish_storage_error(
                                format!("副本已核验，保存位置尚未切换：{error:#}"),
                                cx,
                            );
                            return;
                        }
                        if this.library_root == prepared.journal.source {
                            this.library_root = prepared.journal.destination.clone();
                        }
                        prepared.journal.phase = Phase::Committed;
                        this.storage_ui.error = None;
                        match this.relocate_settings_paths(
                            &prepared.journal.source,
                            &prepared.journal.destination,
                            cx,
                        ) {
                            Ok(()) => prepared.journal.settings_relocated = true,
                            Err(error) => {
                                this.storage_ui.error = Some(format!(
                                    "课程库已移动，模型位置尚未保存。旧位置备份已保留：{error:#}"
                                ))
                            }
                        }
                        if let Err(error) = storage::save_journal(&prepared.path, &prepared.journal)
                        {
                            this.message =
                                Some(format!("课程库已移动，迁移记录稍后重试保存：{error:#}"));
                        }
                        this.storage_ui.busy = false;
                        this.storage_ui.cancel = None;
                        this.storage_ui.progress = Progress {
                            message: format!(
                                "课程库已移动到 {}。旧位置保留为已核验备份。",
                                prepared.journal.destination.display()
                            ),
                            completed: 1,
                            total: 1,
                        };
                        this.restore_storage_state(cx);
                        this.restore_draft(window, cx);
                        this.refresh_library(cx);
                        this.poll_storage(cx);
                        this.start_next_task(cx);
                    }
                    Err(error) => this.finish_storage_error(
                        format!("{error:#}。保存位置尚未切换，已复制的内容保留在目标位置。"),
                        cx,
                    ),
                }
                cx.notify();
            });
        })
        .detach();
        cx.notify();
    }

    fn finish_storage_error(&mut self, message: String, cx: &mut Context<Self>) {
        self.storage_ui.busy = false;
        self.storage_ui.cancel = None;
        self.storage_ui.error = Some(message);
        self.restore_storage_state(cx);
        self.poll_storage(cx);
        self.start_next_task(cx);
        cx.notify();
    }

    fn open_storage_dialog(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let desktop = cx.entity();
        let content = cx.new(|cx| StorageDialog {
            _observation: cx.observe(&desktop, |_, _, cx| cx.notify()),
            desktop,
        });
        let title = self.storage_ui.title();
        window.open_dialog(cx, move |dialog, _, _| {
            dialog
                .title(title)
                .w(px(540.))
                .overlay_closable(false)
                .close_button(false)
                .keyboard(false)
                .child(content.clone())
        });
    }

    fn storage_operation_view(&self, window: &mut Window, cx: &mut Context<Self>) -> Div {
        let cancelling = self
            .storage_ui
            .cancel
            .as_ref()
            .is_some_and(|cancel| cancel.load(Ordering::Relaxed));
        let mut view = v_flex().gap_4().child(
            h_flex()
                .gap_2()
                .items_center()
                .when(self.storage_ui.busy, |row| {
                    row.child(crate::motion::spinner("storage-operation-busy", cx))
                })
                .when(!self.storage_ui.busy, |row| {
                    row.child(
                        if self.storage_ui.error.is_some() {
                            icons::warning()
                        } else {
                            icons::check_circle()
                        }
                        .size_6()
                        .text_color(color(
                            if self.storage_ui.error.is_some() {
                                WARNING
                            } else {
                                SUCCESS
                            },
                        )),
                    )
                })
                .child(
                    accessible_text(
                        "storage-operation-heading",
                        if cancelling {
                            "正在结束操作…"
                        } else if self.storage_ui.busy {
                            "正在处理文件…"
                        } else if self.storage_ui.error.is_some() {
                            "操作未完成"
                        } else {
                            "操作已完成"
                        },
                    )
                    .font_weight(FontWeight::SEMIBOLD),
                ),
        );
        if !self.storage_ui.progress.message.is_empty() {
            view = view.child(accessible_text(
                "storage-operation-state",
                self.storage_ui.progress.message.clone(),
            ));
        }
        if let Some(error) = &self.storage_ui.error {
            view = view.child(
                accessible_text("storage-operation-error", error.clone()).text_color(color(DANGER)),
            );
        }
        if self.storage_ui.busy {
            let progress = &self.storage_ui.progress;
            if progress.total > 0 {
                view = view.child(crate::motion::progress(
                    "storage-progress",
                    progress.completed as f32 / progress.total as f32,
                    window,
                    cx,
                ));
            }
            view = view.child(
                control("cancel-library-move")
                    .icon(icons::close())
                    .disabled(cancelling)
                    .loading(cancelling)
                    .label(if cancelling {
                        "正在结束…"
                    } else if self.storage_ui.association {
                        "取消重新关联"
                    } else if self.storage_ui.cleanup {
                        "取消清理"
                    } else {
                        "取消迁移"
                    })
                    .on_click(cx.listener(|this, _, _, cx| {
                        if let Some(cancel) = &this.storage_ui.cancel {
                            cancel.store(true, Ordering::Relaxed);
                        }
                        this.storage_ui.progress.message = if this.storage_ui.association {
                            "正在结束读取，现有文件会保留…"
                        } else if this.storage_ui.cleanup {
                            "正在结束清理操作…"
                        } else {
                            "正在结束迁移，原课程库会保留…"
                        }
                        .into();
                        cx.notify();
                    })),
            );
        } else {
            view = view.child(
                control("close-storage-operation")
                    .icon(icons::close())
                    .label("关闭")
                    .on_click(|_, window, cx| window.close_dialog(cx)),
            );
        }
        view
    }

    pub fn storage_status_panel(&self, cx: &mut Context<Self>) -> Div {
        let mut view = v_flex().gap_3();
        if let Some(workspace) = &self.workspace {
            let access = storage::library_access(
                workspace
                    .state
                    .libraries
                    .iter()
                    .map(|library| library.root.clone()),
            );
            let message = match access.coverage() {
                storage::LibraryCoverage::Unavailable => Some(
                    "已登记的保存位置暂时都无法访问，笔记内容尚未读取；草稿和任务记录仍保留。"
                        .into(),
                ),
                storage::LibraryCoverage::Partial => Some(format!(
                    "目前 {} 个保存位置可访问，{} 个暂时无法访问。课程库只显示和搜索可访问位置中的笔记。",
                    access.available.len(),
                    access.unavailable.len()
                )),
                storage::LibraryCoverage::Complete => None,
            };
            if let Some(message) = message {
                view = view.child(
                    accessible_text("storage-access-coverage", message)
                        .text_sm()
                        .text_color(color(MUTED)),
                );
            }
        }
        if let Some(error) = &self.storage_ui.error {
            view = view.child(
                accessible_text("storage-status-error", error.clone())
                    .text_sm()
                    .text_color(color(DANGER)),
            );
        }
        if let Some(workspace) = &self.workspace {
            for (index, location) in workspace.state.libraries.iter().enumerate() {
                if needs_reassociation(location) {
                    let id = location.id.clone();
                    view = view.child(v_flex().gap_2()
                        .child(accessible_text(("storage-association-needed", index), format!("「{}」的关联记录缺失。重新关联后可继续使用这个位置，已有文件会保留。", location.name)))
                        .child(accessible_text(("storage-association-path", index), location.root.display().to_string()).text_sm().text_color(color(MUTED)))
                        .child(control(("reassociate-storage-location", index)).icon(icons::storage()).label("重新关联此保存位置").self_start().disabled(self.storage_ui.busy)
                            .on_click(cx.listener(move |this, _, window, cx| this.begin_library_reassociation(id.clone(), window, cx)))));
                } else if location.root.is_dir()
                    && let Err(error) = workspace::check_library(location)
                {
                    view = view.child(
                        accessible_text(
                            ("storage-association-error", index),
                            format!("「{}」暂时无法确认：{error:#}", location.name),
                        )
                        .role(Role::Alert)
                        .text_sm()
                        .text_color(color(DANGER)),
                    );
                }
            }
        }
        for (index, (path, journal)) in self.storage_ui.pending.iter().enumerate() {
            let path = path.clone();
            let abandoned_path = path.clone();
            let journal = journal.clone();
            let destination = journal.destination.clone();
            view = view.child(
                v_flex()
                    .gap_2()
                    .child(accessible_text(
                        ("storage-pending-description", index),
                        format!(
                            "尚未完成迁移：{} → {}",
                            journal.source.display(),
                            journal.destination.display()
                        ),
                    ))
                    .child(
                        control(("resume-library-move", index))
                            .icon(icons::play_arrow())
                            .label("继续迁移并切换位置")
                            .disabled(self.storage_ui.busy)
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.start_library_move(
                                    journal.library_id.clone(),
                                    journal.source.clone(),
                                    journal.destination.clone(),
                                    Some(path.clone()),
                                    window,
                                    cx,
                                )
                            })),
                    )
                    .child(
                        control(("open-move-copy", index))
                            .ghost()
                            .icon(icons::folder_open())
                            .label("打开目标副本")
                            .on_click(move |_, _, cx| cx.open_with_system(&destination)),
                    )
                    .child(
                        control(("abandon-library-move", index))
                            .ghost()
                            .icon(icons::close())
                            .label("结束这次迁移，保留副本")
                            .disabled(self.storage_ui.busy)
                            .on_click(cx.listener(move |this, _, _, cx| {
                                let result = std::fs::read(&abandoned_path)
                                    .map_err(anyhow::Error::from)
                                    .and_then(|bytes| {
                                        serde_json::from_slice::<Journal>(&bytes)
                                            .map_err(anyhow::Error::from)
                                    })
                                    .and_then(|mut journal| {
                                        journal.phase = Phase::Abandoned;
                                        storage::save_journal(&abandoned_path, &journal)
                                    });
                                if let Err(error) = result {
                                    this.storage_ui.error =
                                        Some(format!("尚未保存迁移选择：{error:#}"));
                                }
                                this.restore_storage_state(cx);
                            })),
                    ),
            );
        }
        if let Some(workspace) = &self.workspace {
            for (index, backup) in workspace.state.storage_backups.iter().enumerate() {
                let id = backup.id.clone();
                let path = backup.path.clone();
                view = view.child(
                    v_flex()
                        .gap_2()
                        .child(accessible_text(
                            ("storage-backup-description", index),
                            format!("旧位置备份：{}", backup.path.display()),
                        ))
                        .child(
                            h_flex()
                                .gap_2()
                                .child(
                                    control(("open-library-backup", index))
                                        .icon(icons::folder_open())
                                        .label("打开备份位置")
                                        .on_click(move |_, _, cx| cx.open_with_system(&path)),
                                )
                                .child(
                                    quiet(("cleanup-library-backup", index))
                                        .icon(icons::delete())
                                        .label("清理旧位置备份…")
                                        .disabled(self.storage_ui.busy)
                                        .on_click(cx.listener(move |this, _, window, cx| {
                                            this.begin_backup_cleanup(id.clone(), window, cx)
                                        })),
                                ),
                        ),
                );
            }
        }
        view
    }

    fn begin_backup_cleanup(&mut self, id: String, window: &mut Window, cx: &mut Context<Self>) {
        if self.storage_ui.busy {
            return;
        }
        self.storage_ui.association = false;
        let Some(backup) = self
            .workspace
            .as_ref()
            .and_then(|workspace| {
                workspace
                    .state
                    .storage_backups
                    .iter()
                    .find(|backup| backup.id == id)
            })
            .cloned()
        else {
            return;
        };
        if self.preferences.references_storage_path(&backup.path) {
            self.storage_ui.cleanup = true;
            self.storage_ui.progress = Progress::default();
            self.storage_ui.error = Some("识别模型仍引用这个旧位置，备份已保留。请先在“生成笔记”设置中完成模型位置的保存，再清理备份。".into());
            self.open_storage_dialog(window, cx);
            return;
        }
        let answer = window.prompt(
            PromptLevel::Warning,
            "清理旧位置备份？",
            Some(&format!(
                "将删除 {} 中的旧位置备份。当前课程库仍保留在 {}。",
                backup.path.display(),
                backup.current_root.display()
            )),
            &["清理备份", "保留备份"],
            cx,
        );
        cx.spawn_in(window, async move |this, cx| {
            if answer.await.ok() != Some(0) {
                return;
            }
            let cancel = Arc::new(AtomicBool::new(false));
            let worker_cancel = cancel.clone();
            let roots = this
                .update_in(cx, |this, window, cx| {
                    this.storage_ui.busy = true;
                    this.storage_ui.cleanup = true;
                    this.storage_ui.cancel = Some(cancel.clone());
                    this.storage_ui.error = None;
                    this.storage_ui.progress = Progress {
                        message: "正在核验旧位置备份，确认内容未变化后清理…".into(),
                        completed: 0,
                        total: 0,
                    };
                    this.open_storage_dialog(window, cx);
                    cx.notify();
                    this.workspace
                        .as_ref()
                        .map(|workspace| {
                            workspace
                                .state
                                .libraries
                                .iter()
                                .map(|library| library.root.clone())
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default()
                })
                .unwrap_or_default();
            let worker = cx
                .background_executor()
                .spawn(async move { storage::cleanup_backup(&backup, &roots, &worker_cancel) });
            let result = worker.await;
            let _ = this.update_in(cx, |this, _, cx| {
                this.storage_ui.busy = false;
                this.storage_ui.cancel = None;
                this.storage_ui.progress = Progress::default();
                match result {
                    Ok(()) => {
                        if let Some(workspace) = &mut this.workspace {
                            match workspace.transaction(|state| {
                                state.storage_backups.retain(|backup| backup.id != id);
                                Ok(())
                            }) {
                                Ok(()) => {
                                    this.storage_ui.progress.message =
                                        "旧位置备份已清理，当前课程库保持完整。".into();
                                }
                                Err(error) => {
                                    this.storage_ui.error =
                                        Some(format!("备份已清理，界面记录尚未保存：{error:#}"))
                                }
                            }
                        }
                    }
                    Err(_) if cancel.load(Ordering::Relaxed) => {
                        this.storage_ui.error = Some("已取消清理，旧位置备份仍保留。".into())
                    }
                    Err(error) => this.storage_ui.error = Some(format!("{error:#}")),
                }
                this.start_next_task(cx);
                cx.notify();
            });
        })
        .detach();
    }
}

#[cfg(test)]
mod tests {
    use super::{publish_location, validate_registered_destination};
    use crate::{ConversionOptions, source, storage, workspace};
    use std::{path::PathBuf, sync::atomic::AtomicBool};

    #[test]
    fn registry_commit_preserves_ids_and_moves_only_library_owned_paths() {
        let directory = tempfile::tempdir().unwrap();
        let old = directory.path().join("old");
        let new = directory.path().join("new");
        std::fs::create_dir_all(old.join("media")).unwrap();
        std::fs::create_dir_all(&new).unwrap();
        std::fs::write(old.join("media/video.mp4"), "fixture video content").unwrap();
        let old = std::fs::canonicalize(old).unwrap();
        let new = std::fs::canonicalize(new).unwrap();
        let record_path = directory.path().join("workspace.json");
        let mut workspace = workspace::Workspace::open_at(
            record_path.clone(),
            old.clone(),
            ConversionOptions::default(),
        )
        .unwrap();
        let library_id = workspace.state.default_library.clone();
        let mut source = source::Source {
            input: old.join("media/video.mp4").display().to_string(),
            title: "My confirmed title".into(),
            identity: "local:content-stays-the-same".into(),
            online: false,
            cover: Some(old.join("media/cover.png")),
            ..Default::default()
        };
        let draft = workspace.state.draft_mut().unwrap();
        draft.online = false;
        draft.input = source.input.clone();
        draft.source = Some(source.clone());
        draft.title = "My custom title".into();
        draft.custom_title = true;
        draft.folder = Some(17);
        let original_draft_id = draft.id.clone();
        let mut config = course2md::settings::ConfigFile::default();
        config.defaults.model_dir = Some(old.join("models"));
        let plan = workspace::TaskPlan {
            operation: course2md::execution::Operation::Reprocess {
                base_version_dir: old.join("note/versions/old-version"),
                components: vec!["summary".into()],
                prior_work_dir: Some(old.join(".course2md/work/old-task")),
            },
            source: source.clone(),
            source_id: source.identity.clone(),
            title: "Frozen title".into(),
            library_id: library_id.clone(),
            folder: Some(17),
            options: ConversionOptions::default(),
            subtitle: Some(old.join("subtitles/selected.srt")),
            config,
            asr_service: Some("fixed-asr-version".into()),
            ai_service: Some("fixed-ai-version".into()),
        };
        let (task_id, _) = workspace.state.enqueue(plan, None).unwrap();
        let old_task_work = workspace.state.task(&task_id).unwrap().work_dir.clone();
        source.input = directory.path().join("outside.mp4").display().to_string();
        let outside_input = source.input.clone();
        let mut outside_draft =
            workspace::Draft::new(false, library_id.clone(), ConversionOptions::default());
        outside_draft.input = outside_input.clone();
        outside_draft.source = Some(source);
        let outside_draft_id = outside_draft.id.clone();
        workspace.state.drafts.push(outside_draft);
        workspace
            .state
            .reader_sources
            .insert("local:relocated-video".into(), old.join("media/video.mp4"));
        workspace
            .state
            .reader_sources
            .insert("local:outside-video".into(), PathBuf::from(&outside_input));
        workspace.transaction(|_| Ok(())).unwrap();
        let prepared = storage::prepare_move(
            library_id.clone(),
            &old,
            &new,
            &directory.path().join("journals"),
            &AtomicBool::new(false),
            &|_| {},
        )
        .unwrap();
        assert_eq!(workspace.state.library(&library_id).unwrap().root, old);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let permissions = std::fs::metadata(directory.path()).unwrap().permissions();
            std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o555))
                .unwrap();
            let failed = workspace
                .transaction(|state| publish_location(state, &prepared.journal, &prepared.path));
            std::fs::set_permissions(directory.path(), permissions).unwrap();
            assert!(failed.is_err());
            assert_eq!(workspace.state.library(&library_id).unwrap().root, old);
        }
        let before_commit = workspace::Workspace::open_at(
            record_path.clone(),
            old.clone(),
            ConversionOptions::default(),
        )
        .unwrap();
        assert_eq!(before_commit.state.library(&library_id).unwrap().root, old);
        assert_eq!(
            before_commit.state.task(&task_id).unwrap().work_dir,
            old_task_work
        );
        workspace
            .transaction(|state| publish_location(state, &prepared.journal, &prepared.path))
            .unwrap();
        let mut reopened =
            workspace::Workspace::open_at(record_path, old.clone(), ConversionOptions::default())
                .unwrap();
        let state = &reopened.state;
        let lagging: storage::Journal =
            serde_json::from_slice(&std::fs::read(&prepared.path).unwrap()).unwrap();
        assert_eq!(lagging.phase, storage::Phase::Verified);
        assert_eq!(state.library(&library_id).unwrap().root, new);
        assert_eq!(
            state.reader_sources["local:relocated-video"],
            new.join("media/video.mp4")
        );
        assert_eq!(
            state.reader_sources["local:outside-video"],
            PathBuf::from(&outside_input)
        );
        let draft = state
            .drafts
            .iter()
            .find(|draft| draft.id == original_draft_id)
            .unwrap();
        assert_eq!(draft.library_id, library_id);
        assert_eq!(draft.folder, Some(17));
        assert_eq!(draft.title, "My custom title");
        assert_eq!(
            draft.input,
            new.join("media/video.mp4").display().to_string()
        );
        assert_eq!(
            draft.source.as_ref().unwrap().identity,
            "local:content-stays-the-same"
        );
        assert_eq!(
            state
                .drafts
                .iter()
                .find(|draft| draft.id == outside_draft_id)
                .unwrap()
                .input,
            outside_input
        );
        let task = state.task(&task_id).unwrap();
        assert_eq!(task.plan.title, "Frozen title");
        assert_eq!(task.plan.asr_service.as_deref(), Some("fixed-asr-version"));
        assert_eq!(
            task.plan.config.defaults.model_dir,
            Some(new.join("models"))
        );
        assert_eq!(
            task.work_dir,
            new.join(old_task_work.strip_prefix(&old).unwrap())
        );
        assert!(
            matches!(&task.plan.operation, course2md::execution::Operation::Reprocess {base_version_dir, prior_work_dir:Some(work),..}
            if base_version_dir.starts_with(&new) && work.starts_with(&new))
        );
        assert_eq!(state.storage_backups[0].path, old);
        assert!(
            state
                .library(&library_id)
                .unwrap()
                .previous_roots
                .contains(&old)
        );
        assert!(old.join("media/video.mp4").is_file());
        let final_root = directory.path().join("third");
        std::fs::create_dir(&final_root).unwrap();
        let second_move = storage::prepare_move(
            library_id.clone(),
            &new,
            &final_root,
            &directory.path().join("journals"),
            &AtomicBool::new(false),
            &|_| {},
        )
        .unwrap();
        reopened
            .transaction(|state| publish_location(state, &second_move.journal, &second_move.path))
            .unwrap();
        let location = reopened.state.library(&library_id).unwrap();
        assert!(location.previous_roots.contains(&old) && location.previous_roots.contains(&new));
        assert_eq!(
            reopened.state.reader_sources["local:relocated-video"],
            location.root.join("media/video.mp4")
        );
        assert!(
            reopened
                .state
                .storage_backups
                .iter()
                .all(|backup| backup.current_root == location.root)
        );
    }

    #[test]
    fn another_registered_library_is_rejected_even_when_empty() {
        let directory = tempfile::tempdir().unwrap();
        let first = directory.path().join("first");
        let second = directory.path().join("second");
        std::fs::create_dir_all(&first).unwrap();
        std::fs::create_dir_all(&second).unwrap();
        let mut workspace = workspace::Workspace::open_at(
            directory.path().join("workspace.json"),
            first,
            ConversionOptions::default(),
        )
        .unwrap();
        workspace.state.libraries.push(workspace::LibraryLocation {
            id: "other".into(),
            name: "Other library".into(),
            root: second.clone(),
            previous_roots: Vec::new(),
        });
        assert!(
            validate_registered_destination(
                &workspace.state,
                &workspace.state.default_library,
                &second
            )
            .is_err()
        );
        assert!(std::fs::read_dir(second).unwrap().next().is_none());
    }
}
