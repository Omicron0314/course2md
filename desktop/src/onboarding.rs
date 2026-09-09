//! A resumable first-use guide. Configuration, contract tests and model jobs
//! use the same persistence and execution paths as Settings.
use crate::credentials::Secret;
use crate::preferences::{
    Authentication, BindingScope, GenerationPreferences, PreferenceGroup, ServiceConfiguration,
    ServiceDraft, ServicePurpose, ServiceTestEvidence, ServiceVersion, TestOutcome,
};
use crate::service_test::{self, TestKind};
use crate::settings_ui::{
    field_label, settings_detail_row, settings_field_row, settings_row, settings_value,
};
use crate::theme::*;
use crate::*;
use course2md::{
    config::{AsrProvider, model_dir_from},
    models::status::CacheState,
};
use gpui_component::{
    menu::{DropdownMenu, PopupMenuItem},
    switch::Switch,
};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Step {
    Engine,
    Ai,
    Model,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum InputField {
    Address,
    Model,
    Key,
}

struct ServiceSetup {
    inputs: BTreeMap<InputField, Entity<InputState>>,
    draft: ServiceDraft,
    original: Option<ServiceVersion>,
    pending_version: Option<ServiceVersion>,
    errors: Vec<preferences::FieldError>,
    evidence: Vec<(TestKind, ServiceTestEvidence)>,
    running: Option<Arc<AtomicBool>>,
    test_serial: u64,
    details_open: bool,
    show_key: bool,
    _subscriptions: Vec<Subscription>,
}

#[derive(Clone, PartialEq, Eq)]
struct ModelRequest {
    provider: AsrProvider,
    model: String,
    root: PathBuf,
}

struct ModelPreparation {
    request: ModelRequest,
    result: Option<(String, bool)>,
    cancelled: bool,
}

impl ServiceSetup {
    fn new(purpose: ServicePurpose, window: &mut Window, cx: &mut Context<Desktop>) -> Self {
        let inputs: BTreeMap<_, _> = [InputField::Address, InputField::Model, InputField::Key]
            .into_iter()
            .map(|field| {
                let placeholder = match field {
                    InputField::Address => "https://api.example.com/v1",
                    InputField::Model => "服务商提供的模型 ID",
                    InputField::Key => "",
                };
                (
                    field,
                    cx.new(|cx| {
                        InputState::new(window, cx)
                            .placeholder(placeholder)
                            .masked(field == InputField::Key)
                    }),
                )
            })
            .collect();
        let subscriptions = inputs
            .values()
            .map(|input| {
                cx.subscribe_in(input, window, move |this, _, event, _, cx| {
                    if matches!(event, InputEvent::Change) && this.onboarding.active {
                        let service = this.onboarding.service_mut(purpose);
                        service.errors.clear();
                        service.evidence.clear();
                        this.onboarding.notice = None;
                        cx.notify();
                    }
                })
            })
            .collect();
        Self {
            inputs,
            draft: ServiceDraft::new(purpose),
            original: None,
            pending_version: None,
            errors: Vec::new(),
            evidence: Vec::new(),
            running: None,
            test_serial: 0,
            details_open: false,
            show_key: false,
            _subscriptions: subscriptions,
        }
    }
    fn cancel(&self) {
        if let Some(cancel) = &self.running {
            cancel.store(true, Ordering::Release);
        }
    }
    fn value(&self, field: InputField, cx: &App) -> String {
        self.inputs[&field].read(cx).value().to_string()
    }
    fn current_draft(&self, cx: &App) -> ServiceDraft {
        let mut draft = self.draft.clone();
        draft.address = self.value(InputField::Address, cx);
        draft.model = self.value(InputField::Model, cx);
        draft
    }
    fn unchanged(&self, cx: &App) -> bool {
        self.value(InputField::Key, cx).is_empty()
            && self.original.as_ref().is_some_and(|version| {
                self.current_draft(cx)
                    .configuration()
                    .ok()
                    .is_some_and(|config| {
                        config.fingerprint("configuration")
                            == version.config.fingerprint("configuration")
                    })
            })
    }
}

pub(crate) struct State {
    pub(crate) active: bool,
    step: Step,
    session: u64,
    provider: Option<AsrProvider>,
    model: String,
    ai_proofread: bool,
    ai_summary: bool,
    speech: ServiceSetup,
    ai: ServiceSetup,
    notice: Option<(String, bool)>,
    finish_failed: bool,
    model_details_open: bool,
    model_preparation: Option<ModelPreparation>,
    model_status_request: Option<ModelRequest>,
    model_return_page: Option<Page>,
    scroll: ScrollHandle,
}

impl State {
    pub(crate) fn new(window: &mut Window, cx: &mut Context<Desktop>) -> Self {
        Self {
            active: false,
            step: Step::Engine,
            session: 0,
            provider: None,
            model: "qwen3-1.7b".into(),
            ai_proofread: true,
            ai_summary: false,
            speech: ServiceSetup::new(ServicePurpose::Speech, window, cx),
            ai: ServiceSetup::new(ServicePurpose::Ai, window, cx),
            notice: None,
            finish_failed: false,
            model_details_open: false,
            model_preparation: None,
            model_status_request: None,
            model_return_page: None,
            scroll: ScrollHandle::new(),
        }
    }
    fn service(&self, purpose: ServicePurpose) -> &ServiceSetup {
        match purpose {
            ServicePurpose::Speech => &self.speech,
            ServicePurpose::Ai => &self.ai,
        }
    }
    fn service_mut(&mut self, purpose: ServicePurpose) -> &mut ServiceSetup {
        match purpose {
            ServicePurpose::Speech => &mut self.speech,
            ServicePurpose::Ai => &mut self.ai,
        }
    }
}

fn provider_label(provider: Option<AsrProvider>) -> &'static str {
    match provider {
        None => "自动选择（推荐）",
        Some(AsrProvider::Coreml) => "Apple 原生",
        Some(AsrProvider::Gpu) => "GPU",
        Some(AsrProvider::Cpu) => "CPU",
        Some(AsrProvider::Npu) => "Intel NPU",
        Some(AsrProvider::Api) => "语音服务",
    }
}

fn engine_preferences(
    current: &GenerationPreferences,
    provider: Option<AsrProvider>,
    model: &str,
) -> GenerationPreferences {
    let mut next = current.clone();
    next.options.provider = provider;
    if provider != Some(AsrProvider::Api) {
        next.options.asr_model = Some(model.to_owned());
        next.local_model_draft = None;
    }
    next
}

fn requested_tests(
    purpose: ServicePurpose,
    proofread: bool,
    summary: bool,
    vision: bool,
) -> Vec<TestKind> {
    if purpose == ServicePurpose::Speech {
        return vec![TestKind::Speech];
    }
    let mut tests = enabled_ai_tests(proofread, summary, vision);
    if tests.is_empty() {
        tests.push(TestKind::Proofread);
    }
    tests
}

fn enabled_ai_tests(proofread: bool, summary: bool, vision: bool) -> Vec<TestKind> {
    let mut tests = Vec::new();
    if proofread {
        tests.push(TestKind::Proofread);
    }
    if summary {
        tests.push(TestKind::Summary);
    }
    if proofread && vision {
        tests.push(TestKind::Vision);
    }
    tests
}

fn newly_enabled_ai_tests(
    current: &GenerationPreferences,
    proofread: bool,
    summary: bool,
) -> Vec<TestKind> {
    let existing = enabled_ai_tests(current.ai_proofread, current.ai_summary, current.vision);
    enabled_ai_tests(proofread, summary, current.vision)
        .into_iter()
        .filter(|kind| !existing.contains(kind))
        .collect()
}

fn model_action_label(provider: AsrProvider, state: Option<&CacheState>) -> &'static str {
    match state {
        Some(CacheState::Missing | CacheState::Partial) => "下载并准备",
        Some(CacheState::Cached) if matches!(provider, AsrProvider::Cpu | AsrProvider::Gpu) => {
            "检查模型文件"
        }
        Some(CacheState::Loaded) => "重新验证加载",
        _ => "验证模型加载",
    }
}

fn evidence_covers(
    config: &ServiceConfiguration,
    required: &[TestKind],
    evidence: &[(TestKind, ServiceTestEvidence)],
) -> bool {
    required.iter().all(|kind| {
        evidence.iter().any(|(tested, evidence)| {
            tested == kind
                && evidence.outcome == TestOutcome::Passed
                && evidence.contract == kind.contract()
                && evidence.fingerprint == config.fingerprint(kind.contract())
        })
    })
}

fn help(id: impl Into<ElementId>, value: impl Into<SharedString>) -> Stateful<Div> {
    settings_value(id, value).text_color(color(MUTED))
}

impl Desktop {
    pub(crate) fn start_onboarding(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.setup_model_job_running() {
            self.open_setup_model_status(window, cx);
            return;
        }
        if self.page == Page::New && !self.save_current_draft(cx) {
            return;
        }
        self.onboarding.speech.cancel();
        self.onboarding.ai.cancel();
        for purpose in [ServicePurpose::Speech, ServicePurpose::Ai] {
            let id = self.onboarding.service(purpose).draft.id.clone();
            let _ = self.preferences.discard_service_draft(&id);
        }
        self.onboarding.active = false;
        self.onboarding.session += 1;
        let generation = self.preferences.generation();
        self.onboarding.provider = generation.options.provider;
        self.onboarding.model = generation
            .options
            .asr_model
            .clone()
            .unwrap_or("qwen3-1.7b".into());
        self.onboarding.ai_proofread = generation.ai_proofread;
        self.onboarding.ai_summary = generation.ai_summary;
        let refs = self.preferences.default_refs();
        self.hydrate_setup_service(ServicePurpose::Speech, refs.asr.as_deref(), window, cx);
        self.hydrate_setup_service(ServicePurpose::Ai, refs.llm.as_deref(), window, cx);
        if self.onboarding.ai.original.is_none() {
            self.onboarding.ai_proofread = true;
        }
        self.onboarding.model_details_open = false;
        self.onboarding.finish_failed = false;
        self.onboarding.model_status_request = None;
        self.onboarding.model_return_page = None;
        self.onboarding.active = true;
        self.setup_step(Step::Engine, cx);
        self.root_focus.focus(window, cx);
    }

    fn hydrate_setup_service(
        &mut self,
        purpose: ServicePurpose,
        reference: Option<&str>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let original = reference
            .and_then(|id| self.preferences.version(id))
            .cloned();
        let draft = original
            .as_ref()
            .map(ServiceDraft::from_version)
            .unwrap_or_else(|| ServiceDraft::new(purpose));
        let service = self.onboarding.service_mut(purpose);
        service.cancel();
        service.running = None;
        service.draft = draft.clone();
        service.original = original;
        service.pending_version = None;
        service.errors.clear();
        service.evidence.clear();
        service.details_open = false;
        service.show_key = false;
        for (field, value) in [
            (InputField::Address, draft.address),
            (InputField::Model, draft.model),
            (InputField::Key, String::new()),
        ] {
            service.inputs[&field].update(cx, |input, cx| input.set_value(value, window, cx));
        }
        service.inputs[&InputField::Key].update(cx, |input, cx| input.set_masked(true, window, cx));
    }

    fn setup_step(&mut self, step: Step, cx: &mut Context<Self>) {
        if step == Step::Engine && self.setup_model_job_running() {
            return;
        }
        if step != Step::Model {
            self.onboarding.model_status_request = None;
            self.onboarding.model_return_page = None;
        }
        self.onboarding.step = step;
        self.onboarding.notice = None;
        self.onboarding.scroll.set_offset(point(px(0.), px(0.)));
        cx.notify();
    }

    fn setup_tests(&self, purpose: ServicePurpose) -> Vec<TestKind> {
        requested_tests(
            purpose,
            self.onboarding.ai_proofread,
            self.onboarding.ai_summary,
            self.preferences.generation().vision,
        )
    }

    fn setup_required_tests(&self, purpose: ServicePurpose, cx: &App) -> Vec<TestKind> {
        if !self.onboarding.service(purpose).unchanged(cx) {
            return self.setup_tests(purpose);
        }
        match purpose {
            ServicePurpose::Speech => Vec::new(),
            ServicePurpose::Ai => newly_enabled_ai_tests(
                self.preferences.generation(),
                self.onboarding.ai_proofread,
                self.onboarding.ai_summary,
            ),
        }
    }

    fn setup_service_passed(&self, purpose: ServicePurpose, cx: &App) -> bool {
        let service = self.onboarding.service(purpose);
        service.value(InputField::Key, cx).is_empty()
            && service
                .current_draft(cx)
                .configuration()
                .ok()
                .is_some_and(|config| {
                    evidence_covers(
                        &config,
                        &self.setup_required_tests(purpose, cx),
                        &service.evidence,
                    )
                })
    }

    fn stage_setup_service(
        &mut self,
        purpose: ServicePurpose,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<ServiceConfiguration> {
        let service = self.onboarding.service(purpose);
        let draft = service.current_draft(cx);
        let key = service.value(InputField::Key, cx);
        match self.preferences.save_service_draft(
            draft,
            (!key.trim().is_empty()).then(|| Secret::new(key.clone())),
        ) {
            Ok(draft) => {
                let service = self.onboarding.service_mut(purpose);
                service.draft = draft;
                if !key.is_empty() {
                    service.inputs[&InputField::Key]
                        .update(cx, |input, cx| input.set_value("", window, cx));
                }
                service.errors = service.draft.validate();
                if let Some(error) = service.errors.first() {
                    let field = match error.field {
                        "address" => InputField::Address,
                        "model" => InputField::Model,
                        _ => InputField::Key,
                    };
                    service.inputs[&field].update(cx, |input, cx| input.focus(window, cx));
                    cx.notify();
                    return None;
                }
                service.draft.configuration().ok()
            }
            Err(error) => {
                self.onboarding.notice = Some((
                    preferences::save_failure_message(PreferenceGroup::Services, &error),
                    true,
                ));
                cx.notify();
                None
            }
        }
    }

    fn start_setup_test(
        &mut self,
        purpose: ServicePurpose,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.onboarding.service(purpose).running.is_some() {
            return;
        }
        let Some(config) = self.stage_setup_service(purpose, window, cx) else {
            return;
        };
        let required = self.setup_required_tests(purpose, cx);
        let required = if required.is_empty() {
            self.setup_tests(purpose)
        } else {
            required
        };
        let cancel = Arc::new(AtomicBool::new(false));
        let session = self.onboarding.session;
        let service = self.onboarding.service_mut(purpose);
        service.test_serial += 1;
        let serial = service.test_serial;
        service.running = Some(cancel.clone());
        service.evidence.clear();
        self.onboarding.notice = None;
        let vault = self.preferences.vault();
        let task = cx.background_executor().spawn(async move {
            let mut results = Vec::new();
            for kind in required {
                let evidence =
                    service_test::test_service(config.clone(), kind, vault.clone(), cancel.clone())
                        .await;
                let passed = evidence.outcome == TestOutcome::Passed;
                results.push((kind, evidence));
                if !passed || cancel.load(Ordering::Acquire) {
                    break;
                }
            }
            results
        });
        cx.spawn(async move |this, cx| {
            let results = task.await;
            let _ = this.update(cx, |this, cx| {
                let mut persisted = true;
                for (_, evidence) in &results {
                    persisted &= this.preferences.record_test(evidence.clone()).is_ok();
                }
                if this.onboarding.session != session
                    || this.onboarding.service(purpose).test_serial != serial
                {
                    return;
                }
                let service = this.onboarding.service(purpose);
                let unchanged = service.value(InputField::Key, cx).is_empty()
                    && service
                        .current_draft(cx)
                        .configuration()
                        .ok()
                        .is_some_and(|config| {
                            results.iter().all(|(_, evidence)| {
                                evidence.fingerprint == config.fingerprint(&evidence.contract)
                            })
                        });
                let service = this.onboarding.service_mut(purpose);
                service.running = None;
                if unchanged {
                    service.evidence = results;
                    if !persisted {
                        this.onboarding.notice = Some((
                            "检查结果已收到，但尚未保存；不会自动重新发送检查。".into(),
                            true,
                        ));
                    }
                } else {
                    this.onboarding.notice =
                        Some(("配置已改变，请检查当前填写的配置。".into(), false));
                }
                cx.notify();
            });
        })
        .detach();
        cx.notify();
    }

    fn continue_setup_service(
        &mut self,
        purpose: ServicePurpose,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let unchanged = self.onboarding.service(purpose).unchanged(cx);
        let pending = self.onboarding.service(purpose).pending_version.clone();
        if pending.is_none() && !self.setup_service_passed(purpose, cx) {
            self.start_setup_test(purpose, window, cx);
            return;
        }
        let version = if let Some(version) = pending {
            version
        } else if unchanged {
            let Some(version) = self.onboarding.service(purpose).original.clone() else {
                return;
            };
            if let Err(error) = self.preferences.check_dispatch(&version.id) {
                self.onboarding.notice = Some((format!("当前服务暂不可用：{error:#}"), true));
                cx.notify();
                return;
            }
            version
        } else {
            if self.stage_setup_service(purpose, window, cx).is_none() {
                return;
            }
            let id = self.onboarding.service(purpose).draft.id.clone();
            match self
                .preferences
                .publish_service(&id, BindingScope::Defaults)
            {
                Ok(version) => {
                    self.onboarding.service_mut(purpose).pending_version = Some(version.clone());
                    version
                }
                Err(error) => {
                    self.onboarding.notice = Some((
                        preferences::save_failure_message(PreferenceGroup::Services, &error),
                        true,
                    ));
                    cx.notify();
                    return;
                }
            }
        };
        let mut next = self.preferences.generation().clone();
        match purpose {
            ServicePurpose::Speech => next.options.provider = Some(AsrProvider::Api),
            ServicePurpose::Ai => {
                next.ai_proofread = self.onboarding.ai_proofread;
                next.ai_summary = self.onboarding.ai_summary;
            }
        }
        if !self.commit_generation(next, cx) {
            self.onboarding.notice = Some((
                "服务配置已保留，默认选项尚未保存。可以重试，已保存的服务不会重复创建。".into(),
                true,
            ));
            return;
        }
        self.refresh_dispatch_controls(cx);
        self.hydrate_setup_service(purpose, Some(&version.id), window, cx);
        self.setup_step(
            if purpose == ServicePurpose::Speech {
                Step::Ai
            } else {
                Step::Model
            },
            cx,
        );
    }

    fn save_setup_engine(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.onboarding.provider == Some(AsrProvider::Api) {
            self.continue_setup_service(ServicePurpose::Speech, window, cx);
            return;
        }
        let next = engine_preferences(
            self.preferences.generation(),
            self.onboarding.provider,
            &self.onboarding.model,
        );
        if self.commit_generation(next, cx) {
            self.setup_step(Step::Ai, cx);
        } else {
            self.onboarding.notice =
                Some(("默认引擎尚未保存，当前选择保留。请重试保存。".into(), true));
        }
    }

    fn skip_setup_ai(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.onboarding.ai.cancel();
        // Keep this private edit valid when the user returns within the guide.
        // Only finishing the guide discards unpublished service drafts.
        self.setup_step(Step::Model, cx);
    }

    fn setup_model_request(&self) -> ModelRequest {
        self.onboarding
            .model_status_request
            .clone()
            .unwrap_or_else(|| ModelRequest {
                provider: self
                    .onboarding
                    .provider
                    .unwrap_or_else(|| self.recommended_local_provider()),
                model: self.onboarding.model.clone(),
                root: model_dir_from(self.preferences.generation().options.model_dir.as_deref()),
            })
    }

    fn setup_model_job_running(&self) -> bool {
        self.onboarding
            .model_preparation
            .as_ref()
            .is_some_and(|preparation| {
                let request = &preparation.request;
                self.setup_model_snapshot(request.provider, Some(&request.model), &request.root)
                    .preparing
            })
    }

    fn retain_setup_model_result(&mut self) {
        let Some(preparation) = &self.onboarding.model_preparation else {
            return;
        };
        let request = &preparation.request;
        let snapshot =
            self.setup_model_snapshot(request.provider, Some(&request.model), &request.root);
        if let Some(result) = snapshot.notice {
            let preparation = self.onboarding.model_preparation.as_mut().unwrap();
            preparation.result = Some(result);
            preparation.cancelled = snapshot.cancelled;
        }
    }

    fn open_setup_model_status(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.onboarding.active && self.page == Page::New && !self.save_current_draft(cx) {
            return;
        }
        let Some(preparation) = &self.onboarding.model_preparation else {
            return;
        };
        if !self.onboarding.active {
            self.onboarding.model_return_page = Some(self.page);
        }
        self.onboarding.model_status_request = Some(preparation.request.clone());
        self.onboarding.active = true;
        self.setup_step(Step::Model, cx);
        self.root_focus.focus(window, cx);
    }

    fn finish_onboarding(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let mut application = self.preferences.application().clone();
        application.desktop.setup_completed = true;
        if let Err(error) = self.preferences.save_application(application) {
            self.onboarding.finish_failed = true;
            self.onboarding.notice = Some((
                preferences::save_failure_message(PreferenceGroup::Application, &error),
                true,
            ));
            cx.notify();
            return;
        }
        self.leave_onboarding(window, cx);
    }

    fn leave_onboarding(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        for purpose in [ServicePurpose::Speech, ServicePurpose::Ai] {
            self.onboarding.service(purpose).cancel();
            let id = self.onboarding.service(purpose).draft.id.clone();
            let _ = self.preferences.discard_service_draft(&id);
            self.onboarding.service(purpose).inputs[&InputField::Key]
                .update(cx, |input, cx| input.set_value("", window, cx));
        }
        self.onboarding.active = false;
        self.onboarding.finish_failed = false;
        self.refresh_preference_defaults(cx);
        let return_page = self
            .onboarding
            .model_return_page
            .take()
            .unwrap_or(Page::New);
        self.navigate(return_page, cx);
        self.root_focus.focus(window, cx);
        cx.notify();
    }

    pub(crate) fn onboarding_page(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        self.retain_setup_model_result();
        let step = self.onboarding.step;
        let (number, title, description) = match step {
            Step::Engine => (
                1,
                "设置默认识别方式",
                "后续生成笔记会使用这些默认选项。有字幕时优先使用字幕。",
            ),
            Step::Ai => (
                2,
                "配置 AI 服务",
                "AI 可以校对文字和生成摘要。没有 AI 服务，也能生成笔记。",
            ),
            Step::Model => (
                3,
                "准备识别模型",
                "现在准备模型，或先使用视频字幕，稍后再下载。",
            ),
        };
        let content = match step {
            Step::Engine => self.setup_engine_content(window, cx),
            Step::Ai => self.setup_service_content(ServicePurpose::Ai, window, cx),
            Step::Model => self.setup_model_content(window, cx),
        };
        let compact = f32::from(window.viewport_size().height) < f32::from(window.rem_size()) * 32.;
        let heading = v_flex()
            .flex_shrink_0()
            .gap_2()
            .when(!compact, |view| {
                view.child(
                    help("setup-step", format!("使用引导 · {number} / 3")).text_size(TEXT_AUX),
                )
            })
            .child(
                settings_value("setup-title", title)
                    .role(Role::Heading)
                    .text_size(TEXT_DISPLAY)
                    .font_weight(FontWeight::SEMIBOLD),
            )
            .when(!compact, |view| {
                view.child(help("setup-description", description))
            });
        let scrolling_content = v_flex()
            .w_full()
            .min_w_0()
            .gap_4()
            .child(heading)
            .child(content.pb_4())
            .when_some(self.onboarding.notice.clone(), |view, (message, error)| {
                view.child(
                    settings_value("setup-notice", message)
                        .role(Role::Status)
                        .text_color(color(if error { DANGER } else { MUTED })),
                )
            });
        v_flex()
            .flex_1()
            .min_h_0()
            .w_full()
            .max_w(rems(44.))
            .mx_auto()
            .px_6()
            .py_4()
            .gap_4()
            .child(
                v_flex()
                    .id("setup-scroll")
                    .flex_1()
                    .min_h_0()
                    .w_full()
                    .min_w_0()
                    .overflow_y_scroll()
                    .track_scroll(&self.onboarding.scroll)
                    .child(scrolling_content),
            )
            .child(self.setup_footer(compact, cx))
            .into_any_element()
    }

    fn setup_engine_content(&self, window: &mut Window, cx: &mut Context<Self>) -> Div {
        let selected = self.onboarding.provider;
        let weak = cx.weak_entity();
        let picker = control("setup-engine")
            .w_full()
            .label(provider_label(selected))
            .child(icons::chevron_down().size_4().flex_shrink_0())
            .dropdown_menu(move |menu, _, _| {
                [
                    None,
                    Some(AsrProvider::Coreml),
                    Some(AsrProvider::Gpu),
                    Some(AsrProvider::Cpu),
                    Some(AsrProvider::Npu),
                    Some(AsrProvider::Api),
                ]
                .into_iter()
                .fold(menu, |menu, provider| {
                    let weak = weak.clone();
                    menu.item(
                        PopupMenuItem::new(provider_label(provider))
                            .checked(provider == selected)
                            .on_click(move |_, _, cx| {
                                let _ = weak.update(cx, |this, cx| {
                                    this.onboarding.provider = provider;
                                    this.onboarding.notice = None;
                                    cx.notify();
                                });
                            }),
                    )
                })
            });
        let mut body = v_flex().w_full().min_w_0().gap_4().child(settings_row(
            "setup-engine-label",
            "识别方式",
            "",
            picker,
        ));
        if selected == Some(AsrProvider::Api) {
            return body
                .child(help(
                    "setup-speech-help",
                    "音频会发送到你配置的语音服务；无需下载本机模型。",
                ))
                .child(self.setup_service_content(ServicePurpose::Speech, window, cx));
        }
        body = body.child(if let Some(environment) = &self.environment {
            let provider = selected.unwrap_or_else(|| self.recommended_local_provider());
            let available = match provider {
                AsrProvider::Coreml => environment.apple,
                AsrProvider::Gpu => environment.llama && environment.gpu.is_some(),
                AsrProvider::Cpu => environment.llama,
                AsrProvider::Npu => environment.npu,
                AsrProvider::Api => true,
            } && environment.engine;
            help(
                "setup-detected-engine",
                format!(
                    "{}{}{}",
                    if selected.is_none() {
                        "本机会选择 "
                    } else {
                        "已选择 "
                    },
                    provider_label(Some(provider)),
                    if available {
                        "。运行环境可用。"
                    } else {
                        "。运行环境仍需准备，可在模型步骤查看原因。"
                    }
                ),
            )
            .into_any_element()
        } else {
            h_flex()
                .gap_2()
                .items_center()
                .child(motion::spinner("setup-engine-detecting", cx))
                .child(help(
                    "setup-engine-detecting-label",
                    "正在检测本机可用引擎…",
                ))
                .into_any_element()
        });
        let provider = selected.unwrap_or_else(|| self.recommended_local_provider());
        let mut models = vec![("qwen3-1.7b".to_owned(), "Qwen3 1.7B".to_owned())];
        if matches!(provider, AsrProvider::Coreml | AsrProvider::Npu) {
            models.extend([
                ("qwen3-0.6b".into(), "Qwen3 0.6B".into()),
                ("whisper".into(), "Whisper".into()),
            ]);
        }
        if provider == AsrProvider::Npu {
            models.extend([
                ("whisper-tiny".into(), "Whisper Tiny".into()),
                ("whisper-base".into(), "Whisper Base".into()),
                ("whisper-small".into(), "Whisper Small".into()),
            ]);
        }
        let model = self.onboarding.model.clone();
        let label = models
            .iter()
            .find(|(id, _)| id == &model)
            .map(|(_, label)| label.clone())
            .unwrap_or_else(|| format!("当前模型：{model}"));
        let weak = cx.weak_entity();
        body.child(settings_row(
            "setup-model-label",
            "识别模型",
            "",
            control("setup-model-choice")
                .w_full()
                .label(label)
                .child(icons::chevron_down().size_4().flex_shrink_0())
                .dropdown_menu(move |menu, _, _| {
                    models.iter().fold(menu, |menu, (id, label)| {
                        let id = id.clone();
                        let weak = weak.clone();
                        menu.item(
                            PopupMenuItem::new(label.clone())
                                .checked(id == model)
                                .on_click(move |_, _, cx| {
                                    let _ = weak.update(cx, |this, cx| {
                                        this.onboarding.model = id.clone();
                                        this.onboarding.notice = None;
                                        cx.notify();
                                    });
                                }),
                        )
                    })
                }),
        ))
        .child(help(
            "setup-model-choice-help",
            "模型在最后一步准备，也可以稍后下载。",
        ))
    }

    fn setup_input_row(
        &self,
        purpose: ServicePurpose,
        field: InputField,
        label: &'static str,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let service = self.onboarding.service(purpose);
        let id = format!("setup-{}-{field:?}", purpose as usize);
        let input = text_input(&service.inputs[&field])
            .w_full()
            .aria_label(label)
            .disabled(service.running.is_some() || service.pending_version.is_some());
        let key = match field {
            InputField::Address => "address",
            InputField::Model => "model",
            InputField::Key => "api_key",
        };
        let mut field_column = v_flex().w_full().min_w_0().gap_2();
        if field == InputField::Key {
            field_column = field_column.child(
                h_flex()
                    .w_full()
                    .min_w_0()
                    .gap_2()
                    .items_center()
                    .child(div().flex_1().min_w_0().child(input))
                    .child(
                        quiet(format!("setup-key-visibility-{}", purpose as usize))
                            .icon(if service.show_key {
                                icons::eye_off()
                            } else {
                                icons::eye()
                            })
                            .label(if service.show_key { "隐藏" } else { "显示" })
                            .tooltip(if service.show_key {
                                "隐藏密钥"
                            } else {
                                "显示密钥"
                            })
                            .on_click(cx.listener(move |this, _, window, cx| {
                                let service = this.onboarding.service_mut(purpose);
                                service.show_key = !service.show_key;
                                let masked = !service.show_key;
                                service.inputs[&InputField::Key]
                                    .update(cx, |input, cx| input.set_masked(masked, window, cx));
                                cx.notify();
                            })),
                    ),
            );
            if service.draft.credential.is_some() {
                field_column = field_column.child(
                    help(
                        format!("setup-kept-key-{}", purpose as usize),
                        "已保存密钥。留空保留，输入新值才会替换。",
                    )
                    .text_size(TEXT_AUX),
                );
            }
        } else {
            field_column = field_column.child(input);
        }
        field_column = field_column.children(
            service
                .errors
                .iter()
                .filter(|error| error.field == key)
                .enumerate()
                .map(|(index, error)| {
                    settings_value(
                        SharedString::from(format!("{id}-error-{index}")),
                        error.message.clone(),
                    )
                    .text_color(color(DANGER))
                }),
        );
        let row = settings_field_row(SharedString::from(id.clone()), label, "", field_column);
        crate::focus_scroll::RevealFocus::new(
            SharedString::from(format!("{id}-reveal")),
            row,
            self.onboarding.scroll.clone(),
        )
        .into_any_element()
    }

    fn setup_service_content(
        &self,
        purpose: ServicePurpose,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Div {
        let service = self.onboarding.service(purpose);
        let busy = service.running.is_some() || service.pending_version.is_some();
        let mut body = v_flex().w_full().min_w_0().gap_4();
        if purpose == ServicePurpose::Speech {
            body = body.child(settings_row(
                "setup-speech-protocol-label",
                "接口类型",
                "",
                SingleChoiceGroup::new("setup-speech-protocol", "语音接口类型")
                    .full_width()
                    .options([("transcriptions", "语音转录"), ("chat", "音频聊天")])
                    .selected(
                        if service.draft.protocol == preferences::ServiceProtocol::SpeechChat {
                            "chat"
                        } else {
                            "transcriptions"
                        },
                    )
                    .disabled(busy)
                    .on_change(cx.listener(|this, value: &SharedString, _, cx| {
                        this.onboarding.speech.draft.protocol = if value.as_ref() == "chat" {
                            preferences::ServiceProtocol::SpeechChat
                        } else {
                            preferences::ServiceProtocol::SpeechTranscriptions
                        };
                        this.onboarding.speech.evidence.clear();
                        cx.notify();
                    })),
            ));
        }
        body = body
            .child(self.setup_input_row(purpose, InputField::Address, "服务地址", cx))
            .child(self.setup_input_row(purpose, InputField::Model, "模型 ID", cx))
            .child(settings_row(
                format!("setup-auth-label-{}", purpose as usize),
                "认证方式",
                "",
                SingleChoiceGroup::new(format!("setup-auth-{}", purpose as usize), "服务认证")
                    .full_width()
                    .options([("key", "API Key"), ("none", "无需认证")])
                    .selected(if service.draft.authentication == Authentication::ApiKey {
                        "key"
                    } else {
                        "none"
                    })
                    .disabled(busy)
                    .on_change(cx.listener(move |this, value: &SharedString, _, cx| {
                        let service = this.onboarding.service_mut(purpose);
                        service.draft.authentication = if value.as_ref() == "none" {
                            Authentication::None
                        } else {
                            Authentication::ApiKey
                        };
                        service.evidence.clear();
                        service.errors.clear();
                        cx.notify();
                    })),
            ));
        if service.draft.authentication == Authentication::ApiKey {
            body = body.child(self.setup_input_row(purpose, InputField::Key, "API Key", cx));
        }
        if purpose == ServicePurpose::Ai {
            for (id, label, checked) in [
                ("proofread", "自动校对文字", self.onboarding.ai_proofread),
                ("summary", "生成课程摘要", self.onboarding.ai_summary),
            ] {
                body = body.child(settings_row(
                    format!("setup-ai-{id}-label"),
                    label,
                    "",
                    crate::focus_scroll::FocusRing::new(
                        format!("setup-ai-{id}-focus"),
                        Switch::new(format!("setup-ai-{id}"))
                            .checked(checked)
                            .disabled(busy)
                            .on_click(cx.listener(move |this, enabled, _, cx| {
                                if id == "proofread" {
                                    this.onboarding.ai_proofread = *enabled;
                                } else {
                                    this.onboarding.ai_summary = *enabled;
                                }
                                cx.notify();
                            })),
                    ),
                ));
            }
        }
        let required = self.setup_required_tests(purpose, cx);
        let optional_recheck = required.is_empty();
        let required = if optional_recheck {
            self.setup_tests(purpose)
        } else {
            required
        };
        body = body
            .child(
                help(
                    format!("setup-test-notice-{}", purpose as usize),
                    service_test::TEST_NOTICE,
                )
                .text_size(TEXT_AUX),
            )
            .child(
                help(
                    format!("setup-test-scope-{}", purpose as usize),
                    format!(
                        "{}：{}",
                        if optional_recheck {
                            "可重新检查"
                        } else {
                            "检查内容"
                        },
                        required
                            .iter()
                            .map(|kind| kind.label())
                            .collect::<Vec<_>>()
                            .join("、")
                    ),
                )
                .text_size(TEXT_AUX),
            );
        if let Some(cancel) = &service.running {
            let stopping = cancel.load(Ordering::Acquire);
            body = body
                .child(
                    h_flex()
                        .items_center()
                        .gap_2()
                        .child(motion::spinner(
                            format!("setup-service-check-{}", purpose as usize),
                            cx,
                        ))
                        .child(settings_value(
                            format!("setup-service-check-label-{}", purpose as usize),
                            if stopping {
                                "正在等待这次检查结束…"
                            } else {
                                "正在检查配置…"
                            },
                        )),
                )
                .child(
                    outline_pill(format!("setup-cancel-check-{}", purpose as usize))
                        .self_start()
                        .icon(icons::close())
                        .label("停止检查")
                        .disabled(stopping)
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.onboarding.service(purpose).cancel();
                            this.onboarding.notice = Some((
                                "已请求停止。已发送的请求可能仍被服务处理，不会自动重试。".into(),
                                false,
                            ));
                            cx.notify();
                        })),
                );
        } else if self.setup_service_passed(purpose, cx) {
            body = body.child(
                quiet(format!("setup-recheck-service-{}", purpose as usize))
                    .self_start()
                    .icon(icons::refresh())
                    .label(if service.evidence.is_empty() {
                        "检查当前配置"
                    } else {
                        "重新检查"
                    })
                    .disabled(service.pending_version.is_some())
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.start_setup_test(purpose, window, cx)
                    })),
            );
        }
        for (index, (kind, evidence)) in service.evidence.iter().enumerate() {
            body = body.child(settings_field_row(
                format!("setup-test-result-{}-{index}", purpose as usize),
                kind.label(),
                "",
                settings_value(
                    format!("setup-test-message-{}-{index}", purpose as usize),
                    evidence.message.clone(),
                )
                .text_color(color(if evidence.outcome == TestOutcome::Passed {
                    INK
                } else {
                    DANGER
                })),
            ));
        }
        if !service.evidence.is_empty() {
            let details = service
                .evidence
                .iter()
                .flat_map(|(_, evidence)| evidence.details.iter().cloned())
                .collect::<Vec<_>>()
                .join("\n");
            body = body
                .child(
                    quiet(format!("setup-test-details-{}", purpose as usize))
                        .self_start()
                        .icon(icons::info())
                        .label(if service.details_open {
                            "收起检查详情"
                        } else {
                            "检查详情"
                        })
                        .on_click(cx.listener(move |this, _, _, cx| {
                            let service = this.onboarding.service_mut(purpose);
                            service.details_open = !service.details_open;
                            cx.notify();
                        })),
                )
                .child(motion::disclosure(
                    format!("setup-test-detail-content-{}", purpose as usize),
                    service.details_open,
                    v_flex().w_full().min_w_0().child(
                        help(
                            format!("setup-test-details-text-{}", purpose as usize),
                            details,
                        )
                        .text_size(TEXT_AUX),
                    ),
                    window,
                    cx,
                ));
        }
        body
    }

    fn setup_model_content(&mut self, window: &mut Window, cx: &mut Context<Self>) -> Div {
        let request = self.setup_model_request();
        let provider = request.provider;
        if provider == AsrProvider::Api {
            let version = self
                .preferences
                .default_refs()
                .asr
                .and_then(|id| self.preferences.version(&id))
                .cloned();
            return v_flex()
                .w_full()
                .min_w_0()
                .gap_4()
                .child(
                    settings_value("setup-cloud-model-title", "语音服务不需要本机模型")
                        .font_weight(FontWeight::MEDIUM),
                )
                .child(help(
                    "setup-cloud-model-description",
                    version
                        .map(|version| {
                            format!(
                                "识别时使用 {} · {}。",
                                version.config.name, version.config.model
                            )
                        })
                        .unwrap_or_else(|| "当前没有默认语音服务，请返回第一步完成配置。".into()),
                ))
                .child(help(
                    "setup-later-preferences",
                    "你可以随时在设置中更换引擎、服务或重新开始用户引导。",
                ));
        }
        let preparation_result = self
            .onboarding
            .model_preparation
            .as_ref()
            .filter(|preparation| preparation.request == request)
            .and_then(|preparation| preparation.result.clone());
        let previously_cancelled = self
            .onboarding
            .model_preparation
            .as_ref()
            .is_some_and(|preparation| preparation.request == request && preparation.cancelled);
        let model = request.model;
        let root = request.root;
        self.ensure_model_diagnostic(provider, Some(&model), &root, cx);
        let snapshot = self.setup_model_snapshot(provider, Some(&model), &root);
        let cancelled = if snapshot.notice.is_some() {
            snapshot.cancelled
        } else {
            previously_cancelled
        };
        let status = snapshot
            .result
            .as_ref()
            .and_then(|result| result.as_ref().ok());
        let mut body = v_flex().w_full().min_w_0().gap_4().child(
            settings_value(
                "setup-model-identity",
                format!("{} · {model}", provider_label(Some(provider))),
            )
            .font_weight(FontWeight::MEDIUM),
        );
        let (label, hint) = if snapshot.preparing {
            ("正在准备模型", "下载、文件检查和加载验证会按实际进度更新。")
        } else if cancelled {
            ("模型准备已暂停", "下载文件已保留，可以继续准备。")
        } else if snapshot.checking {
            ("正在检查模型", "正在读取本机已有文件，不会自动开始下载。")
        } else if snapshot.device_issue.is_some() {
            (
                "运行环境需要准备",
                "当前引擎还不能使用。可以重新检查，或返回选择其他引擎。",
            )
        } else {
            match status.map(|status| &status.state) {
                Some(CacheState::Loaded) => ("模型已验证加载", "可以开始使用。"),
                Some(CacheState::Cached) => (
                    "模型文件已下载",
                    "尚未验证加载，可以先检查文件或在首次识别时加载。",
                ),
                Some(CacheState::Missing) => {
                    ("模型尚未下载", "下载完成前，可以先使用视频已有字幕。")
                }
                Some(CacheState::Partial) => ("模型尚未下载完整", "继续准备会复用已经下载的文件。"),
                Some(CacheState::Unsupported) => {
                    ("引擎不支持当前模型", "请返回第一步选择其他模型。")
                }
                None => ("模型检查未完成", "可以重新检查；详细原因保留在下方。"),
            }
        };
        body = body
            .child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .when(
                        (snapshot.checking && !cancelled) || snapshot.preparing,
                        |row| row.child(motion::spinner("setup-model-spinner", cx)),
                    )
                    .child(
                        settings_value("setup-model-state", label).font_weight(FontWeight::MEDIUM),
                    ),
            )
            .child(help("setup-model-state-hint", hint));
        if snapshot.preparing {
            body = body.child(self.setup_model_progress(window, cx)).child(
                outline_pill("setup-pause-model")
                    .self_start()
                    .icon(icons::pause())
                    .label(if self.cancelling {
                        "正在暂停…"
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
        } else {
            let allowed = self.environment.is_some()
                && snapshot.device_issue.is_none()
                && status.is_some_and(|status| status.can_prepare);
            let needs_download = status.is_some_and(|status| {
                matches!(status.state, CacheState::Missing | CacheState::Partial)
            });
            if allowed {
                let button = if needs_download {
                    primary_pill("setup-prepare-model")
                } else {
                    outline_pill("setup-prepare-model")
                };
                let target = root.clone();
                body = body.child(
                    button
                        .self_start()
                        .icon(icons::download())
                        .label(if cancelled {
                            "继续准备"
                        } else {
                            model_action_label(provider, status.map(|status| &status.state))
                        })
                        .disabled(self.job.is_some() || snapshot.checking)
                        .on_click(cx.listener(move |this, _, _, cx| {
                            if this.job.is_some() {
                                return;
                            }
                            let request = ModelRequest {
                                provider,
                                model: model.clone(),
                                root: target.clone(),
                            };
                            this.onboarding.model_status_request = Some(request.clone());
                            this.onboarding.model_preparation = Some(ModelPreparation {
                                request,
                                result: None,
                                cancelled: false,
                            });
                            this.prepare_setup_model(provider, Some(&model), &target, cx);
                        })),
                );
            }
            body = body.child(
                quiet("setup-recheck-model")
                    .self_start()
                    .icon(icons::refresh())
                    .label("重新检查")
                    .disabled(snapshot.checking || self.job.is_some())
                    .on_click(cx.listener(|this, _, _, cx| this.refresh_environment(cx))),
            );
            if self.job.is_some() {
                body = body.child(help(
                    "setup-model-job-busy",
                    "当前处理完成后可以准备模型；也可先继续使用应用。",
                ));
            }
        }
        let mut details = v_flex().w_full().min_w_0().gap_3();
        if let Some((message, error)) = snapshot.notice.or(preparation_result) {
            if error && !cancelled && !snapshot.preparing {
                body = body.child(
                    settings_value(
                        "setup-model-result",
                        "准备未完成，已下载文件保留。可重试，具体原因见模型详情。",
                    )
                    .text_color(color(DANGER))
                    .role(Role::Status),
                );
            }
            details = details.child(help("setup-model-preparation-detail", message));
        }
        if let Some(issue) = snapshot.device_issue {
            details = details.child(help("setup-model-device-detail", issue));
        }
        if let Some(Err(error)) = &snapshot.result {
            details = details.child(help("setup-model-error-detail", error.clone()));
        }
        if let Some(status) = status {
            if status.bytes > 0 {
                details = details.child(settings_detail_row(
                    "setup-model-size-label",
                    "已下载",
                    settings_value(
                        "setup-model-size",
                        format!("{:.1} MB", status.bytes as f64 / 1024. / 1024.),
                    ),
                ));
            }
            for (index, part) in status.parts.iter().enumerate() {
                details = details
                    .child(field_label(("setup-model-part", index), part.name.clone()))
                    .child(
                        help(("setup-model-path", index), part.path.display().to_string())
                            .text_size(TEXT_AUX),
                    );
            }
            if let Some(error) = &status.last_error {
                details = details.child(help(
                    "setup-model-last-error",
                    format!("上次准备记录：{error}"),
                ));
            }
        }
        body.child(
            quiet("setup-model-details")
                .self_start()
                .icon(icons::info())
                .label(if self.onboarding.model_details_open {
                    "收起模型详情"
                } else {
                    "模型详情"
                })
                .on_click(cx.listener(|this, _, _, cx| {
                    this.onboarding.model_details_open = !this.onboarding.model_details_open;
                    cx.notify();
                })),
        )
        .child(motion::disclosure(
            "setup-model-detail-content",
            self.onboarding.model_details_open,
            details,
            window,
            cx,
        ))
        .child(
            help(
                "setup-download-scope",
                "准备模型可能下载文件，课程内容不会上传。",
            )
            .text_size(TEXT_AUX),
        )
    }

    fn setup_model_progress(&self, window: &mut Window, cx: &mut Context<Self>) -> Div {
        let mut view = v_flex().w_full().min_w_0().gap_3();
        for (index, (stage, progress)) in self
            .progress
            .iter()
            .filter(|(stage, progress)| {
                (stage.starts_with("model") || stage.contains("download"))
                    && progress.has_samples()
                    && !progress.done
            })
            .enumerate()
        {
            view = view
                .child(help(
                    ("setup-model-progress", index),
                    format!(
                        "{} · {}",
                        activity::title(stage),
                        progress.detail(stage, true)
                    ),
                ))
                .when_some(progress.fraction(), |view, fraction| {
                    view.child(motion::progress(
                        ("setup-model-progress-bar", index),
                        fraction,
                        window,
                        cx,
                    ))
                });
        }
        view
    }

    fn setup_service_action_label(&self, purpose: ServicePurpose, cx: &App) -> &'static str {
        let service = self.onboarding.service(purpose);
        if let Some(cancel) = &service.running {
            if cancel.load(Ordering::Acquire) {
                "正在停止检查…"
            } else {
                "正在检查…"
            }
        } else if service.pending_version.is_some() {
            "重试保存"
        } else if service.unchanged(cx) && self.setup_required_tests(purpose, cx).is_empty() {
            "保留并继续"
        } else if self.setup_service_passed(purpose, cx) {
            "保存并继续"
        } else if service.evidence.is_empty() {
            "检查配置"
        } else {
            "重新检查"
        }
    }

    fn setup_footer(&self, compact: bool, cx: &mut Context<Self>) -> Div {
        if self.onboarding.finish_failed {
            return v_flex()
                .w_full()
                .flex_shrink_0()
                .gap_2()
                .child(
                    h_flex()
                        .w_full()
                        .min_w_0()
                        .flex_wrap()
                        .gap_2()
                        .child(
                            quiet("setup-enter-temporarily")
                                .label("暂时进入应用")
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.leave_onboarding(window, cx)
                                })),
                        )
                        .child(div().flex_1())
                        .child(
                            primary_pill("setup-retry-finish")
                                .label("重试保存")
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.finish_onboarding(window, cx)
                                })),
                        ),
                )
                .child(
                    help(
                        "setup-finish-failure-hint",
                        "暂时进入不会把引导标记为完成，也不会保存未完成的服务或引擎选择。",
                    )
                    .text_size(TEXT_AUX),
                );
        }
        let step = self.onboarding.step;
        let model_review = self.onboarding.model_return_page.is_some();
        let model_running = self.setup_model_job_running();
        let emphasize_model_preparation = if step == Step::Model
            && !model_running
            && self.job.is_none()
        {
            let request = self.setup_model_request();
            let snapshot =
                self.setup_model_snapshot(request.provider, Some(&request.model), &request.root);
            request.provider != AsrProvider::Api
                && self.environment.is_some()
                && !snapshot.checking
                && snapshot.device_issue.is_none()
                && snapshot.result.as_ref().is_some_and(|result| {
                    result.as_ref().is_ok_and(|status| {
                        status.can_prepare
                            && matches!(status.state, CacheState::Missing | CacheState::Partial)
                    })
                })
        } else {
            false
        };
        let busy = match step {
            Step::Engine if self.onboarding.provider == Some(AsrProvider::Api) => {
                self.onboarding.speech.running.is_some()
            }
            Step::Ai => self.onboarding.ai.running.is_some(),
            _ => false,
        };
        let label = match step {
            Step::Engine if self.onboarding.provider == Some(AsrProvider::Api) => {
                self.setup_service_action_label(ServicePurpose::Speech, cx)
            }
            Step::Engine => "保存并继续",
            Step::Ai => self.setup_service_action_label(ServicePurpose::Ai, cx),
            Step::Model if model_review => "返回应用",
            Step::Model if model_running => "继续使用",
            Step::Model if emphasize_model_preparation => "稍后准备",
            Step::Model => "开始使用",
        };
        let mut row = h_flex()
            .w_full()
            .min_w_0()
            .gap_2()
            .items_center()
            .flex_wrap();
        if step != Step::Ai && !model_review {
            row = row.child(
                quiet("setup-later")
                    .label(if step == Step::Model {
                        "返回引擎选择"
                    } else {
                        "稍后设置"
                    })
                    .disabled(step == Step::Model && model_running)
                    .on_click(cx.listener(move |this, _, window, cx| {
                        if step == Step::Model {
                            this.setup_step(Step::Engine, cx);
                        } else {
                            this.finish_onboarding(window, cx);
                        }
                    })),
            );
        }
        if step == Step::Ai {
            row = row.child(
                quiet("setup-back")
                    .icon(icons::arrow_left())
                    .label("上一步")
                    .on_click(cx.listener(|this, _, _, cx| this.setup_step(Step::Engine, cx))),
            );
        }
        row = row.child(div().flex_1());
        if step == Step::Ai {
            row = row.child(
                outline_pill("setup-skip-ai")
                    .label("稍后配置 AI")
                    .on_click(cx.listener(|this, _, window, cx| this.skip_setup_ai(window, cx))),
            );
        }
        let next = if emphasize_model_preparation {
            outline_pill("setup-next")
        } else {
            primary_pill("setup-next")
        };
        row = row.child(
            next.label(label)
                .icon(icons::arrow_forward())
                .disabled(busy)
                .on_click(cx.listener(move |this, _, window, cx| match step {
                    Step::Engine => this.save_setup_engine(window, cx),
                    Step::Ai => this.continue_setup_service(ServicePurpose::Ai, window, cx),
                    Step::Model if model_review => this.leave_onboarding(window, cx),
                    Step::Model => this.finish_onboarding(window, cx),
                })),
        );
        let hint = if model_running {
            "继续使用后，模型仍会在后台准备；可随时查看进度。".to_owned()
        } else if compact {
            let number = match step {
                Step::Engine => 1,
                Step::Ai => 2,
                Step::Model => 3,
            };
            format!("使用引导 · {number} / 3 · 可在设置中重新开始")
        } else {
            "以后可在设置中重新开始引导；已保存的配置会保留。".to_owned()
        };
        v_flex()
            .w_full()
            .flex_shrink_0()
            .gap_2()
            .child(row)
            .child(help("setup-return-hint", hint).text_size(TEXT_AUX))
    }

    pub(crate) fn onboarding_background_notice(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<Div> {
        self.retain_setup_model_result();
        if self.onboarding.active {
            return None;
        }
        let preparation = self.onboarding.model_preparation.as_ref()?;
        let running = self.setup_model_job_running();
        let result_error = preparation.result.as_ref().is_some_and(|(_, error)| *error);
        let label = if running {
            if self.cancelling {
                "正在暂停模型准备"
            } else {
                "模型正在后台准备"
            }
        } else if preparation.cancelled {
            "模型准备已暂停，下载文件已保留"
        } else if result_error {
            "模型准备未完成，已下载文件保留"
        } else if preparation.result.is_some() {
            "模型准备已结束，可以查看检查结果"
        } else {
            "模型准备已结束，请查看结果"
        };
        Some(
            v_flex()
                .w_full()
                .min_w_0()
                .px_6()
                .py_3()
                .gap_2()
                .bg(color(INSET))
                .child(
                    h_flex()
                        .w_full()
                        .min_w_0()
                        .gap_2()
                        .items_center()
                        .flex_wrap()
                        .child(
                            h_flex()
                                .flex_1()
                                .flex_basis(rems(240. / 14.))
                                .min_w_0()
                                .gap_2()
                                .items_center()
                                .when(running, |row| {
                                    row.child(div().flex_shrink_0().child(motion::spinner(
                                        "setup-background-model-spinner",
                                        cx,
                                    )))
                                })
                                .child(
                                    settings_value(
                                        "setup-background-model-status",
                                        format!("{} · {label}", preparation.request.model),
                                    )
                                    .w_auto()
                                    .flex_1()
                                    .text_color(color(
                                        if result_error && !preparation.cancelled && !running {
                                            DANGER
                                        } else {
                                            INK
                                        },
                                    )),
                                ),
                        )
                        .child(
                            quiet("setup-background-model-open")
                                .label(if running {
                                    "查看进度"
                                } else {
                                    "查看结果"
                                })
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.open_setup_model_status(window, cx)
                                })),
                        )
                        .when(!running, |row| {
                            row.child(
                                quiet("setup-background-model-dismiss")
                                    .label("关闭提示")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        if this.setup_model_job_running() {
                                            return;
                                        }
                                        this.onboarding.model_preparation = None;
                                        cx.notify();
                                    })),
                            )
                        }),
                )
                .when(running, |view| {
                    view.child(self.setup_model_progress(window, cx))
                }),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{engine_preferences, evidence_covers, newly_enabled_ai_tests, requested_tests};
    use crate::preferences::{
        Authentication, GenerationPreferences, ServiceConfiguration, ServiceProtocol,
        ServicePurpose, ServiceTestEvidence, TestOutcome,
    };
    use crate::service_test::TestKind;
    use course2md::config::AsrProvider;

    #[test]
    fn existing_ai_configuration_checks_only_new_capabilities_and_never_disabled_ones() {
        let mut existing = GenerationPreferences::default();
        existing.ai_proofread = true;
        existing.ai_summary = false;
        existing.vision = true;

        assert!(newly_enabled_ai_tests(&existing, true, false).is_empty());
        assert!(newly_enabled_ai_tests(&existing, false, false).is_empty());
        assert_eq!(
            newly_enabled_ai_tests(&existing, true, true),
            vec![TestKind::Summary]
        );
        assert_eq!(
            newly_enabled_ai_tests(&existing, false, true),
            vec![TestKind::Summary]
        );

        existing.ai_proofread = false;
        existing.ai_summary = true;
        assert_eq!(
            newly_enabled_ai_tests(&existing, true, true),
            vec![TestKind::Proofread, TestKind::Vision]
        );
        assert!(newly_enabled_ai_tests(&existing, false, false).is_empty());

        existing.vision = false;
        assert_eq!(
            newly_enabled_ai_tests(&existing, true, true),
            vec![TestKind::Proofread]
        );
    }

    #[test]
    fn engine_choice_changes_only_engine_fields_and_preserves_other_preferences() {
        let mut current = GenerationPreferences::default();
        current.ai_proofread = true;
        current.ai_summary = true;
        current.ai_concurrency = 5;
        current.prompt = Some("Keep technical terms".into());
        current.options.keep_video = Some(true);
        current.options.model_dir = Some("/existing/cache".into());
        let next = engine_preferences(&current, Some(AsrProvider::Gpu), "qwen3-1.7b");
        let mut expected = current.clone();
        expected.options.provider = Some(AsrProvider::Gpu);
        expected.options.asr_model = Some("qwen3-1.7b".into());
        assert_eq!(next, expected);
        let cloud = engine_preferences(&current, Some(AsrProvider::Api), "ignored");
        assert_eq!(cloud.options.asr_model, current.options.asr_model);
        assert_eq!(cloud.options.model_dir, current.options.model_dir);
    }

    #[test]
    fn checks_cover_selected_capabilities_and_never_treat_other_contracts_as_passes() {
        let config = ServiceConfiguration {
            name: "Test".into(),
            protocol: ServiceProtocol::AiChat,
            endpoint: "https://example.test/v1/chat/completions".into(),
            model: "model-a".into(),
            authentication: Authentication::None,
            credential: None,
            credential_source: None,
        };
        let passed = |kind: TestKind| {
            (
                kind,
                ServiceTestEvidence {
                    fingerprint: config.fingerprint(kind.contract()),
                    contract: kind.contract().into(),
                    tested_at: 0,
                    outcome: TestOutcome::Passed,
                    message: "样例通过".into(),
                    details: Vec::new(),
                },
            )
        };
        let required = requested_tests(ServicePurpose::Ai, true, true, false);
        assert!(!evidence_covers(
            &config,
            &required,
            &[passed(TestKind::Proofread)]
        ));
        let evidence = vec![passed(TestKind::Proofread), passed(TestKind::Summary)];
        assert!(evidence_covers(&config, &required, &evidence));
        let mut changed = config.clone();
        changed.model = "model-b".into();
        assert!(!evidence_covers(&changed, &required, &evidence));
        let mut cancelled = evidence.clone();
        cancelled[0].1.outcome = TestOutcome::OutcomeUnknown;
        assert!(!evidence_covers(&config, &required, &cancelled));
        assert_eq!(
            requested_tests(ServicePurpose::Speech, true, true, true),
            vec![TestKind::Speech]
        );
        assert_eq!(
            requested_tests(ServicePurpose::Ai, false, true, true),
            vec![TestKind::Summary]
        );
        assert_eq!(
            requested_tests(ServicePurpose::Ai, true, false, true),
            vec![TestKind::Proofread, TestKind::Vision]
        );
    }
}
