//! Model evidence is loaded in the background; preparation uses the existing model job.
use crate::theme::*;
use crate::*;
use course2md::{
    config::{AsrProvider, model_dir_from},
    models::status::{CacheState, LocalModelStatus},
};
use gpui_component::{button::*, progress::Progress};
use std::path::Path;

#[derive(Clone)]
struct Request {
    provider: AsrProvider,
    model: String,
    root: PathBuf,
}
impl Request {
    fn new(provider: AsrProvider, model: Option<&str>, root: &Path) -> Self {
        Self {
            provider,
            model: model
                .filter(|model| !model.trim().is_empty())
                .unwrap_or("qwen3-1.7b")
                .into(),
            root: root.into(),
        }
    }
    fn key(&self) -> String {
        format!(
            "{}:{}:{}",
            self.provider.as_str(),
            self.model,
            self.root.display()
        )
    }
}
struct Entry {
    request: Request,
    generation: u64,
    checking: bool,
    result: Option<Result<LocalModelStatus, String>>,
}
#[derive(Default)]
pub(super) struct State {
    entries: BTreeMap<String, Entry>,
    preparing: Option<Request>,
    result: Option<(String, bool)>,
    details: bool,
}
fn bytes(value: u64) -> String {
    if value >= 1024 * 1024 * 1024 {
        format!("{:.2} GB", value as f64 / (1024_f64.powi(3)))
    } else if value >= 1024 * 1024 {
        format!("{:.1} MB", value as f64 / (1024_f64.powi(2)))
    } else {
        format!("{value} 字节")
    }
}
fn provider_name(provider: AsrProvider) -> &'static str {
    match provider {
        AsrProvider::Coreml => "Apple 原生",
        AsrProvider::Gpu => "GPU",
        AsrProvider::Cpu => "CPU",
        AsrProvider::Npu => "Intel NPU",
        AsrProvider::Api => "语音服务",
    }
}
impl Desktop {
    fn default_model_request(&self) -> Request {
        let defaults = &self.preferences.generation().options;
        Request::new(
            defaults
                .provider
                .unwrap_or_else(|| self.recommended_local_provider()),
            defaults.asr_model.as_deref(),
            &model_dir_from(defaults.model_dir.as_deref()),
        )
    }
    pub fn ensure_model_diagnostic(
        &mut self,
        provider: AsrProvider,
        model: Option<&str>,
        root: &Path,
        cx: &mut Context<Self>,
    ) {
        self.check_model_request(Request::new(provider, model, root), false, cx);
    }
    fn check_model_request(&mut self, request: Request, force: bool, cx: &mut Context<Self>) {
        if request.provider == AsrProvider::Api {
            return;
        }
        let key = request.key();
        let state = &mut self.settings_ui.model_diagnostics;
        if !force && state.entries.contains_key(&key) {
            return;
        }
        let generation = state
            .entries
            .get(&key)
            .map_or(1, |entry| entry.generation + 1);
        state.entries.insert(
            key.clone(),
            Entry {
                request: request.clone(),
                generation,
                checking: true,
                result: None,
            },
        );
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_executor()
                .spawn(async move {
                    course2md::models::status::inspect(
                        request.provider,
                        &request.model,
                        &request.root,
                    )
                    .map_err(|error| format!("{error:#}"))
                })
                .await;
            let _ = this.update(cx, |this, cx| {
                if let Some(entry) = this.settings_ui.model_diagnostics.entries.get_mut(&key)
                    && entry.generation == generation
                {
                    entry.checking = false;
                    entry.result = Some(result);
                    cx.notify();
                }
            });
        })
        .detach();
    }
    pub fn refresh_model_diagnostics(&mut self, cx: &mut Context<Self>) {
        let mut requests = self
            .settings_ui
            .model_diagnostics
            .entries
            .values()
            .map(|entry| (entry.request.key(), entry.request.clone()))
            .collect::<BTreeMap<_, _>>();
        let default = self.default_model_request();
        requests.insert(default.key(), default);
        if let Some(request) = self.settings_ui.model_diagnostics.preparing.clone() {
            requests.insert(request.key(), request);
        }
        for request in requests.into_values() {
            self.check_model_request(request, true, cx);
        }
    }
    pub(super) fn ensure_settings_model_diagnostic(&mut self, cx: &mut Context<Self>) {
        self.check_model_request(self.default_model_request(), false, cx);
    }
    pub fn model_preparation_args(&self) -> Vec<String> {
        let request = self
            .settings_ui
            .model_diagnostics
            .preparing
            .clone()
            .unwrap_or_else(|| self.default_model_request());
        vec![
            "models".into(),
            "prepare".into(),
            "--provider".into(),
            request.provider.as_str().into(),
            "--model".into(),
            request.model,
            "--dir".into(),
            request.root.display().to_string(),
            "--json".into(),
        ]
    }
    fn begin_model_preparation(&mut self, request: Request, cx: &mut Context<Self>) {
        if self.job.is_some() {
            self.message = Some("当前处理结束后可以准备模型。已下载的文件会保留。".into());
            cx.notify();
            return;
        }
        self.settings_ui.model_diagnostics.preparing = Some(request);
        self.settings_ui.model_diagnostics.result = None;
        self.settings_ui.model_diagnostics.details = false;
        self.start(Kind::Models, cx);
    }
    pub fn model_preparation_finished(
        &mut self,
        success: bool,
        cancelled: bool,
        cx: &mut Context<Self>,
    ) {
        let request = self
            .settings_ui
            .model_diagnostics
            .preparing
            .clone()
            .unwrap_or_else(|| self.default_model_request());
        self.settings_ui.model_diagnostics.result = Some((
            if success {
                format!(
                    "{} · {} 的准备已完成，正在重新检查缓存。",
                    provider_name(request.provider),
                    request.model
                )
            } else if cancelled {
                format!(
                    "{} 的准备已停止。已下载完成的文件保留，可以继续准备。",
                    request.model
                )
            } else {
                format!(
                    "{} 的准备未完成：{}。已下载完成的文件保留。",
                    request.model,
                    self.task_error
                        .as_deref()
                        .unwrap_or("转换程序没有返回成功结果")
                )
            },
            !success,
        ));
        self.refresh_model_diagnostics(cx);
        cx.notify();
    }
    fn model_device_issue(&self, provider: AsrProvider) -> Option<String> {
        let environment = self.environment.as_ref()?;
        if !environment.engine {
            return Some("转换程序无法运行，请先恢复应用内的转换程序。".into());
        }
        match provider {
            AsrProvider::Coreml if !environment.apple && !cfg!(all(target_os = "macos", target_arch = "aarch64")) => Some("已选择 Apple 原生，但当前系统无法使用这套识别方式。选择仍保留，请选择本机可用的识别方式。".into()),
            AsrProvider::Coreml if !environment.apple => Some("已选择 Apple 原生，但本机没有检测到完整的 Apple 识别运行时。选择仍保留；安装完整应用可以恢复组件，也可以明确选择其他可用方式。".into()),
            AsrProvider::Gpu if !environment.llama => Some("已选择 GPU，但未检测到 llama-server 识别运行时。选择仍保留；安装运行时后重新检查。".into()),
            AsrProvider::Gpu if environment.gpu.is_none() => Some("已选择 GPU，但识别运行时没有报告可用的 GPU。选择仍保留；可以明确改用 CPU。".into()),
            AsrProvider::Cpu if !environment.llama => Some("已选择 CPU，但未检测到 llama-server 识别运行时。安装完成后可以使用已缓存的模型。".into()),
            AsrProvider::Npu if !environment.npu_device => Some("已选择 Intel NPU，但本机未检测到可用的 Intel NPU 设备。选择仍保留，请选择本机可用的识别方式。".into()),
            AsrProvider::Npu if !environment.npu_runtime => Some("已选择 Intel NPU，已检测到设备，但缺少 uv 或 Python 启动器。安装完成后重新检查；选择仍保留。".into()),
            _ => None,
        }
    }
    fn model_runtime_repair(&self, provider: AsrProvider, key: &str) -> Div {
        let mut view = v_flex().gap_2();
        let Some(environment) = &self.environment else {
            return view;
        };
        if !environment.engine
            || (provider == AsrProvider::Coreml
                && !environment.apple
                && cfg!(all(target_os = "macos", target_arch = "aarch64")))
        {
            return view
                .child(
                    accessible_text(
                        SharedString::from(format!("model-reinstall-help-{key}")),
                        "重新安装完整应用会恢复应用内组件，已保存的笔记和设置保留。",
                    )
                    .text_sm(),
                )
                .child(
                    control(SharedString::from(format!("model-reinstall-{key}")))
                        .label("下载完整应用")
                        .self_start()
                        .on_click(|_, _, cx| {
                            cx.open_url("https://github.com/mizorewww/course2md/releases")
                        }),
                );
        }
        if matches!(provider, AsrProvider::Cpu | AsrProvider::Gpu) && !environment.llama {
            view = view.child(
                control(SharedString::from(format!(
                    "model-llama-install-help-{key}"
                )))
                .label("查看 llama.cpp 安装说明")
                .self_start()
                .on_click(|_, _, cx| {
                    cx.open_url("https://github.com/ggml-org/llama.cpp#quick-start")
                }),
            );
            if cfg!(target_os = "macos") {
                view = view
                    .child(accessible_text(
                        SharedString::from(format!("model-llama-install-command-{key}")),
                        "已安装 Homebrew 时，可在终端运行 brew install llama.cpp。完成后重新打开应用并检查本机能力。",
                    ).text_sm())
                    .child(control(SharedString::from(format!("model-copy-llama-install-{key}")))
                        .label("复制识别程序安装命令")
                        .self_start()
                        .on_click(|_, _, cx| cx.write_to_clipboard(ClipboardItem::new_string("brew install llama.cpp".into()))));
            } else {
                view = view.child(accessible_text(
                    SharedString::from(format!("model-llama-path-help-{key}")),
                    "安装与当前系统及设备匹配的 llama-server，并将它所在的目录加入 PATH。完成后重新打开应用并检查本机能力。",
                ).text_sm());
            }
        }
        if provider == AsrProvider::Npu && environment.npu_device && !environment.npu_runtime {
            view = view.child(
                control(SharedString::from(format!(
                    "model-python-install-help-{key}"
                )))
                .label("查看 uv 安装说明")
                .self_start()
                .on_click(|_, _, cx| {
                    cx.open_url("https://docs.astral.sh/uv/getting-started/installation/")
                }),
            );
        }
        view
    }
    pub fn model_readiness_panel(
        &self,
        provider: AsrProvider,
        model: Option<&str>,
        root: &Path,
        cx: &mut Context<Self>,
    ) -> Div {
        let request = Request::new(provider, model, root);
        let key = request.key();
        let entry = self.settings_ui.model_diagnostics.entries.get(&key);
        let device_issue = self.model_device_issue(provider);
        let mut view = v_flex().gap_2().child(
            accessible_text(
                SharedString::from(format!("model-name-{key}")),
                format!("识别模型：{} · {}", provider_name(provider), request.model),
            )
            .font_weight(FontWeight::MEDIUM),
        );
        if let Some(issue) = &device_issue {
            view = view
                .child(
                    accessible_text(
                        SharedString::from(format!("model-device-problem-{key}")),
                        issue.clone(),
                    )
                    .text_sm()
                    .text_color(rgb(0xa32626)),
                )
                .child(self.model_runtime_repair(provider, &key));
        }
        let checking = entry.is_none_or(|entry| entry.checking);
        let status = entry
            .and_then(|entry| entry.result.as_ref())
            .and_then(|result| result.as_ref().ok());
        if checking {
            view = view.child(
                accessible_text(
                    SharedString::from(format!("model-checking-{key}")),
                    "正在检查这套模型的本机缓存…",
                )
                .role(Role::Status)
                .text_sm(),
            );
        }
        if let Some(Err(error)) = entry.and_then(|entry| entry.result.as_ref()) {
            view = view.child(
                accessible_text(
                    SharedString::from(format!("model-check-error-{key}")),
                    format!("模型缓存检查未完成：{error}"),
                )
                .text_sm()
                .text_color(rgb(0xa32626)),
            );
        }
        if let Some(status) = status {
            let description = match status.state {
                CacheState::Missing => {
                    "尚未下载。需要识别时，应用会先准备这套模型；可读取的字幕不需要它。"
                }
                CacheState::Partial => {
                    "模型文件尚未齐全。准备时会复用完整文件，缺少或不完整的文件由下载器重新获取。"
                }
                CacheState::Cached => {
                    "已找到所需模型文件，尚未验证能否加载。生成时仍会检查实际加载结果。"
                }
                CacheState::Loaded => "这套模型已成功加载，缓存文件从上次检查后未改变。",
                CacheState::Unsupported => {
                    "这种识别方式不支持当前模型。原选择保留，请明确选择支持的模型。"
                }
            };
            view = view.child(
                accessible_text(
                    SharedString::from(format!("model-state-{key}")),
                    description,
                )
                .text_sm(),
            );
            if status.bytes > 0 {
                view = view.child(
                    accessible_text(
                        SharedString::from(format!("model-cached-size-{key}")),
                        format!("本机已有缓存：{}", bytes(status.bytes)),
                    )
                    .text_sm()
                    .text_color(color(MUTED)),
                );
            }
            for (index, part) in status.parts.iter().enumerate() {
                let path = part.path.clone();
                view = view
                    .child(
                        accessible_text(
                            SharedString::from(format!("model-cache-path-{key}-{index}")),
                            format!("{}：{}", part.name, part.path.display()),
                        )
                        .text_sm()
                        .text_color(color(MUTED)),
                    )
                    .when(path.is_dir(), |view| {
                        view.child(
                            control(SharedString::from(format!(
                                "open-model-cache-{key}-{index}"
                            )))
                            .ghost()
                            .label("打开缓存位置")
                            .accessibility_label(format!(
                                "打开 {} 的缓存位置：{}",
                                part.name,
                                part.path.display()
                            ))
                            .self_start()
                            .on_click(move |_, _, cx| cx.open_with_system(&path)),
                        )
                    });
                if !part.missing.is_empty() {
                    view = view.child(
                        accessible_text(
                            SharedString::from(format!("model-missing-files-{key}-{index}")),
                            format!("缺少或尚未验证：{}", part.missing.join("、")),
                        )
                        .text_sm(),
                    );
                }
            }
            if let Some(error) = &status.last_error {
                view = view.child(
                    accessible_text(
                        SharedString::from(format!("model-last-error-{key}")),
                        format!("上次准备未完成：{error}"),
                    )
                    .text_sm()
                    .text_color(rgb(0xa32626)),
                );
            }
        }
        let active = self.kind == Kind::Models
            && self.job.is_some()
            && self
                .settings_ui
                .model_diagnostics
                .preparing
                .as_ref()
                .is_some_and(|active| active.key() == key);
        let request_for_check = request.clone();
        let mut actions = h_flex().gap_2().flex_wrap().child(
            control(SharedString::from(format!("recheck-model-{key}")))
                .label("重新检查模型")
                .disabled(checking || active)
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.check_model_request(request_for_check.clone(), true, cx)
                })),
        );
        let prepare_allowed = self.environment.is_some()
            && device_issue.is_none()
            && status.is_some_and(|status| status.can_prepare);
        if active {
            view = view.child(
                accessible_text(
                    "active-model-prepare",
                    format!("正在准备 {}", request.model),
                )
                .role(Role::Status),
            );
            for (index, (stage, progress)) in self
                .progress
                .iter()
                .filter(|(stage, _)| stage.starts_with("model") || stage.contains("download"))
                .enumerate()
            {
                let label = progress.detail(stage, true);
                view = view
                    .child(
                        accessible_text(
                            ("model-download-progress", index),
                            format!("{} · {label}", activity::title(stage)),
                        )
                        .text_sm(),
                    )
                    .when_some(progress.fraction(), |view, fraction| {
                        view.child(
                            Progress::new(("model-download-bar", index))
                                .value(fraction as f32 * 100.),
                        )
                    });
            }
            actions = actions.child(
                control("stop-model-preparation")
                    .label(if self.cancelling {
                        "正在停止…"
                    } else {
                        "暂停准备"
                    })
                    .disabled(self.cancelling)
                    .on_click(cx.listener(|this, _, _, cx| {
                        if let Some(job) = &this.job {
                            job.cancel();
                            this.cancelling = true;
                            cx.notify();
                        }
                    })),
            );
        } else if prepare_allowed {
            let label = match status.map(|status| &status.state) {
                Some(CacheState::Missing) => "下载并准备模型",
                Some(CacheState::Loaded) => "重新验证加载",
                Some(CacheState::Cached)
                    if matches!(provider, AsrProvider::Cpu | AsrProvider::Gpu) =>
                {
                    "检查模型文件"
                }
                Some(CacheState::Cached) => "验证模型加载",
                _ => "继续准备模型",
            };
            actions = actions.child(
                control(SharedString::from(format!("prepare-model-{key}")))
                    .label(label)
                    .accessibility_label(format!(
                        "{label}：{} · {}",
                        provider_name(provider),
                        request.model
                    ))
                    .disabled(self.job.is_some())
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.begin_model_preparation(request.clone(), cx)
                    })),
            );
            if self.job.is_some() {
                view = view.child(
                    accessible_text(
                        SharedString::from(format!("model-waits-for-job-{key}")),
                        "当前处理结束后可准备模型；本次已保存的任务不受影响。",
                    )
                    .text_sm(),
                );
            }
            view = view.child(accessible_text(SharedString::from(format!("model-network-scope-{key}")), "准备可能下载模型文件，不发送课程内容。没有网络时已有完整缓存仍可尝试加载；下载失败会保留具体原因。").text_sm().text_color(color(MUTED)));
        }
        view.child(actions)
    }
    pub(super) fn model_diagnostics_panel(&self, cx: &mut Context<Self>) -> Div {
        let request = self.default_model_request();
        let mut view = v_flex().gap_3().child(
            accessible_text("model-diagnostic-default", "当前默认识别方式与模型")
                .role(Role::Heading),
        );
        if request.provider == AsrProvider::Api {
            view = view.child(
                accessible_text(
                    "default-asr-is-service",
                    "当前默认使用语音服务；本机模型不影响这项选择。",
                )
                .text_sm(),
            );
        } else {
            view = view.child(self.model_readiness_panel(
                request.provider,
                Some(&request.model),
                &request.root,
                cx,
            ));
        }
        if self.kind == Kind::Models
            && self.job.is_some()
            && let Some(active) = &self.settings_ui.model_diagnostics.preparing
            && active.key() != request.key()
        {
            view = view
                .child(
                    accessible_text("other-active-model", "正在准备此前选择的模型")
                        .role(Role::Heading),
                )
                .child(self.model_readiness_panel(
                    active.provider,
                    Some(&active.model),
                    &active.root,
                    cx,
                ));
        }
        if let Some((message, error)) = &self.settings_ui.model_diagnostics.result {
            view = view.child(
                accessible_text("model-prepare-result", message.clone())
                    .role(Role::Status)
                    .text_sm()
                    .text_color(color(if *error { DANGER } else { INK })),
            );
        }
        if let Some(environment) = &self.environment {
            view = view
                .child(
                    accessible_text("model-hardware-heading", "设备与识别程序").role(Role::Heading),
                )
                .child(
                    accessible_text("cpu-device", format!("CPU：{}", std::env::consts::ARCH))
                        .text_sm(),
                )
                .child(
                    accessible_text(
                        "apple-device",
                        if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
                            "Apple 芯片：当前应用原生运行于 Apple 芯片"
                        } else {
                            "Apple 芯片：当前系统不使用 Apple 原生识别"
                        },
                    )
                    .text_sm(),
                )
                .child(
                    accessible_text(
                        "gpu-device",
                        format!(
                            "GPU：{}",
                            environment.gpu.as_deref().unwrap_or(if environment.llama {
                                "识别运行时未报告可用设备"
                            } else {
                                "尚无法检查，缺少 llama-server 运行时"
                            })
                        ),
                    )
                    .text_sm(),
                )
                .child(
                    accessible_text(
                        "npu-device",
                        if environment.npu_device {
                            "Intel NPU：已检测到设备"
                        } else {
                            "Intel NPU：未检测到设备"
                        },
                    )
                    .text_sm(),
                )
                .child(
                    accessible_text(
                        "npu-runtime",
                        if environment.npu_runtime {
                            "NPU 程序：已找到 Python 启动器，首次准备时验证 OpenVINO 及设备编译"
                        } else {
                            "NPU 程序：未找到 uv 或 Python 启动器"
                        },
                    )
                    .text_sm(),
                )
                .child(
                    accessible_text(
                        "apple-runtime",
                        if environment.apple {
                            "Apple 识别程序：已检测到原生运行时与 Metal 资源"
                        } else {
                            "Apple 识别程序：未检测到完整运行时与 Metal 资源"
                        },
                    )
                    .text_sm(),
                );
            let alternatives = [
                (AsrProvider::Coreml, environment.apple),
                (
                    AsrProvider::Gpu,
                    environment.gpu.is_some() && environment.llama,
                ),
                (AsrProvider::Cpu, environment.llama),
                (AsrProvider::Npu, environment.npu),
            ];
            let available = alternatives
                .into_iter()
                .filter(|(_, available)| *available)
                .map(|(provider, _)| provider_name(provider))
                .collect::<Vec<_>>();
            if !available.is_empty() {
                view = view.child(
                    accessible_text(
                        "available-local-model-providers",
                        format!(
                            "本机检测到的识别方式：{}。模型状态分别检查。",
                            available.join("、")
                        ),
                    )
                    .text_sm(),
                );
            }
            if self.model_device_issue(request.provider).is_some() {
                view =
                    view.child(
                        h_flex().gap_2().flex_wrap().children(
                            alternatives
                                .into_iter()
                                .filter(|(provider, available)| {
                                    *available && *provider != request.provider
                                })
                                .map(|(provider, _)| {
                                    control(SharedString::from(format!(
                                        "choose-available-model-{}",
                                        provider.as_str()
                                    )))
                                    .label(format!("默认改用 {}", provider_name(provider)))
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        let mut value = this.generation_edit_base();
                                        value.options.provider = Some(provider);
                                        value.options.asr_model = Some("qwen3-1.7b".into());
                                        if this.commit_generation(value, cx) {
                                            this.refresh_model_diagnostics(cx);
                                        }
                                    }))
                                }),
                        ),
                    );
            }
        }
        if self.settings_ui.model_diagnostics.preparing.is_some()
            && !self.logs.is_empty()
            && self.kind == Kind::Models
        {
            view = view.child(
                control("model-preparation-details")
                    .ghost()
                    .label(if self.settings_ui.model_diagnostics.details {
                        "收起准备详情"
                    } else {
                        "查看准备详情"
                    })
                    .self_start()
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.settings_ui.model_diagnostics.details =
                            !this.settings_ui.model_diagnostics.details;
                        cx.notify();
                    })),
            );
            if self.settings_ui.model_diagnostics.details {
                view = view.child(
                    accessible_text(
                        "model-preparation-log",
                        self.logs.iter().cloned().collect::<Vec<_>>().join("\n"),
                    )
                    .text_sm(),
                );
            }
        }
        view
    }
    pub(super) fn default_model_readiness_panel(&self, cx: &mut Context<Self>) -> Div {
        let request = self.default_model_request();
        self.model_readiness_panel(request.provider, Some(&request.model), &request.root, cx)
    }

    /// One-line readiness conclusion for the generation group main flow; the full
    /// panel stays behind 技术详情.
    pub(super) fn default_model_conclusion(&self) -> (bool, String) {
        let request = self.default_model_request();
        if request.provider == AsrProvider::Api {
            return (true, "当前使用语音服务识别，本机模型不参与。".into());
        }
        if let Some(issue) = self.model_device_issue(request.provider) {
            return (false, issue);
        }
        let entry = self
            .settings_ui
            .model_diagnostics
            .entries
            .get(&request.key());
        if entry.is_none_or(|entry| entry.checking) {
            return (true, format!("正在检查 {} 的本机缓存…", request.model));
        }
        if entry
            .and_then(|entry| entry.result.as_ref())
            .is_some_and(Result::is_err)
        {
            return (
                false,
                "模型缓存检查未完成，可在技术详情中查看原因并重新检查。".into(),
            );
        }
        match entry
            .and_then(|entry| entry.result.as_ref())
            .and_then(|result| result.as_ref().ok())
            .map(|status| &status.state)
        {
            Some(CacheState::Loaded) => (true, format!("✓ 识别模型 {} 已验证加载", request.model)),
            Some(CacheState::Cached) => (
                true,
                format!("识别模型 {} 的缓存完整，尚未验证能否加载。", request.model),
            ),
            Some(CacheState::Missing | CacheState::Partial) => (
                true,
                format!(
                    "○ 识别模型 {} 尚未在本机准备好，需要识别时会自动准备",
                    request.model
                ),
            ),
            Some(CacheState::Unsupported) => (
                false,
                format!("○ 当前识别方式不支持 {}，请改选可用模型", request.model),
            ),
            None => (true, format!("○ 尚未检查 {} 的本机缓存", request.model)),
        }
    }
}
