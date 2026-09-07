//! Four independently saved setting groups and a shared service editor.
use super::*;
#[path = "model_diagnostics.rs"]
mod model_diagnostics;
use crate::credentials::Secret;
use crate::preferences::{
    ApplicationPreferences, Authentication, BindingScope, GenerationPreferences, PreferenceGroup,
    ServiceDraft, ServiceProtocol, ServicePurpose, ServiceVersion,
};
use crate::service_test::{self, TestKind};
use crate::theme::*;
use anyhow::{Context as _, anyhow};
use gpui_component::{
    button::*,
    checkbox::Checkbox,
    input::{InputContentType, Textarea, TextareaState},
    switch::Switch,
};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

actions!(
    course2md_settings,
    [
        NextSettingsTab,
        PreviousSettingsTab,
        FirstSettingsTab,
        LastSettingsTab
    ]
);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum EditField {
    Name,
    Address,
    Model,
    Key,
    LocalModel,
    Languages,
}

struct ServiceEditor {
    draft: ServiceDraft,
    target: Option<String>,
    also_default: bool,
    return_focus: Option<FocusHandle>,
    errors: Vec<preferences::FieldError>,
    status: Option<String>,
    test_kind: TestKind,
    test_running: Option<Arc<AtomicBool>>,
    evidence: Option<preferences::ServiceTestEvidence>,
    pending_binding: Option<ServiceVersion>,
    show_key: bool,
    save_failed: bool,
}

pub struct OrdinaryPreferenceIssue {
    pub group: PreferenceGroup,
    pub message: String,
    pub can_retry: bool,
}

pub(crate) struct State {
    model_diagnostics: model_diagnostics::State,
    tab_focus: [FocusHandle; 4],
    inputs: BTreeMap<EditField, Entity<InputState>>,
    prompt: Entity<TextareaState>,
    editor: Option<ServiceEditor>,
    initialized: bool,
    show_prompt: bool,
    feedback: BTreeMap<PreferenceGroup, (String, bool, Instant)>,
    feedback_details: BTreeMap<PreferenceGroup, String>,
    expanded_feedback: std::collections::BTreeSet<PreferenceGroup>,
    generation_block_messages: std::collections::BTreeSet<String>,
    pending_generation: Option<GenerationPreferences>,
    pending_application: Option<ApplicationPreferences>,
    legacy_status: Option<String>,
    legacy_available: bool,
    _subscriptions: Vec<Subscription>,
}

impl State {
    pub fn new(window: &mut Window, cx: &mut Context<Desktop>) -> Self {
        cx.bind_keys([
            KeyBinding::new("right", NextSettingsTab, Some("SettingsTabs")),
            KeyBinding::new("left", PreviousSettingsTab, Some("SettingsTabs")),
            KeyBinding::new("home", FirstSettingsTab, Some("SettingsTabs")),
            KeyBinding::new("end", LastSettingsTab, Some("SettingsTabs")),
        ]);
        let inputs: BTreeMap<_, _> = [
            EditField::Name,
            EditField::Address,
            EditField::Model,
            EditField::Key,
            EditField::LocalModel,
            EditField::Languages,
        ]
        .into_iter()
        .map(|field| {
            (
                field,
                cx.new(|cx| InputState::new(window, cx).masked(field == EditField::Key)),
            )
        })
        .collect();
        let mut subscriptions = Vec::new();
        for (field, input) in &inputs {
            let field = *field;
            subscriptions.push(cx.subscribe_in(
                input,
                window,
                move |this, _, event, window, cx| {
                    if matches!(event, InputEvent::Change) {
                        this.setting_input_changed(field, cx);
                    }
                    if matches!(event, InputEvent::Blur)
                        && matches!(
                            field,
                            EditField::Name
                                | EditField::Address
                                | EditField::Model
                                | EditField::Key
                        )
                    {
                        this.save_open_service_draft(true, window, cx);
                    }
                },
            ));
        }
        let prompt = cx.new(|cx| TextareaState::new(window, cx).rows(6));
        subscriptions.push(cx.subscribe(&prompt, |this, _, event, cx| {
            if matches!(event, InputEvent::Change) {
                this.save_prompt_draft(cx);
            }
        }));
        Self {
            model_diagnostics: Default::default(),
            tab_focus: std::array::from_fn(|_| cx.focus_handle()),
            inputs,
            prompt,
            editor: None,
            initialized: false,
            show_prompt: false,
            feedback: BTreeMap::new(),
            feedback_details: BTreeMap::new(),
            expanded_feedback: Default::default(),
            generation_block_messages: Default::default(),
            pending_generation: None,
            pending_application: None,
            legacy_status: None,
            legacy_available: course2md::settings::load()
                .map(|config| preferences::has_importable_legacy(&config))
                .unwrap_or(true),
            _subscriptions: subscriptions,
        }
    }
}

struct ServiceDialog {
    desktop: Entity<Desktop>,
    _observation: Subscription,
}
impl Render for ServiceDialog {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.desktop
            .update(cx, |desktop, cx| desktop.service_editor_content(window, cx))
    }
}

fn text(id: impl Into<ElementId>, value: impl Into<SharedString>) -> Stateful<Div> {
    theme::accessible_text(id, value)
}
fn group(id: &'static str, title: &'static str) -> Div {
    v_flex()
        .w_full()
        .gap_4()
        .pt_5()
        .border_t_1()
        .border_color(rgb(LINE))
        .child(
            text(id, title)
                .text_size(rems(16.0 / 14.0))
                .font_weight(FontWeight::SEMIBOLD),
        )
}
pub(super) fn preference(label: &'static str, hint: &'static str, control: Switch) -> Div {
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
                .child(text(
                    SharedString::from(format!("preference-label-{label}")),
                    label,
                ))
                .when(!hint.is_empty(), |view| {
                    view.child(
                        text(SharedString::from(format!("preference-hint-{label}")), hint)
                            .text_sm()
                            .text_color(rgb(MUTED)),
                    )
                }),
        )
        .child(control.accessibility_label(label).p_2())
}

impl Desktop {
    fn generation_edit_base(&self) -> GenerationPreferences {
        self.settings_ui
            .pending_generation
            .as_ref()
            .or_else(|| self.preferences.generation_intent())
            .unwrap_or_else(|| self.preferences.generation())
            .clone()
    }
    fn application_edit_base(&self) -> ApplicationPreferences {
        self.settings_ui
            .pending_application
            .as_ref()
            .or_else(|| self.preferences.application_intent())
            .unwrap_or_else(|| self.preferences.application())
            .clone()
    }
    fn service_draft_changed(&self, draft: &ServiceDraft) -> bool {
        if let Some(old) = draft
            .based_on
            .as_deref()
            .and_then(|id| self.preferences.version(id))
        {
            return match draft.configuration() {
                Err(_) => true,
                Ok(config) => {
                    config.fingerprint("editor") != old.config.fingerprint("editor")
                        || config.name != old.config.name
                }
            };
        }
        !draft.name.is_empty()
            || !draft.address.is_empty()
            || !draft.model.is_empty()
            || draft.credential.is_some()
    }
    fn setting_value(&self, field: EditField, cx: &App) -> String {
        self.settings_ui.inputs[&field].read(cx).value().to_string()
    }
    fn setting_field(&self, field: EditField, label: &'static str, _cx: &App) -> Div {
        let error = self
            .settings_ui
            .editor
            .as_ref()
            .and_then(|editor| {
                editor.errors.iter().find(|error| {
                    error.field
                        == match field {
                            EditField::Address => "address",
                            EditField::Model => "model",
                            EditField::Key => "api_key",
                            _ => "",
                        }
                })
            })
            .map(|error| error.message.clone());
        v_flex()
            .w_full()
            .gap_2()
            .child(
                text(("setting-field-label", field as usize), label)
                    .font_weight(FontWeight::MEDIUM),
            )
            .child(
                Input::new(&self.settings_ui.inputs[&field])
                    .w_full()
                    .when(field == EditField::Key, |input| {
                        input.content_type(InputContentType::Password)
                    })
                    .aria_label(
                        error
                            .as_ref()
                            .map(|error| format!("{label}，{error}"))
                            .unwrap_or_else(|| label.to_owned()),
                    )
                    .readonly(
                        self.settings_ui
                            .editor
                            .as_ref()
                            .is_some_and(|editor| editor.pending_binding.is_some()),
                    )
                    .when(error.is_some(), |input| input.border_color(rgb(0xa32626))),
            )
            .when_some(error, |view, error| {
                view.child(
                    text(("setting-error", field as usize), error)
                        .text_sm()
                        .text_color(rgb(0xa32626)),
                )
            })
    }
    fn hydrate_settings_inputs(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.settings_ui.initialized {
            return;
        }
        self.settings_ui.initialized = true;
        let value = self
            .preferences
            .generation_intent()
            .unwrap_or_else(|| self.preferences.generation())
            .clone();
        if self.preferences.generation_intent().is_some() {
            self.settings_ui.pending_generation = Some(value.clone());
            self.set_settings_feedback(
                PreferenceGroup::Generation,
                "已找回上次未生效的修改，可以重试保存".into(),
                true,
            );
        }
        if let Some(intent) = self.preferences.application_intent().cloned() {
            self.settings_ui.pending_application = Some(intent);
            self.set_settings_feedback(
                PreferenceGroup::Application,
                "已找回上次未生效的修改，可以重试保存".into(),
                true,
            );
        }
        let languages = value
            .subtitle_languages_draft
            .unwrap_or_else(|| value.preferred_subtitle_languages.join(", "));
        let model = value
            .local_model_draft
            .unwrap_or_else(|| value.options.asr_model.unwrap_or_default());
        self.settings_ui.inputs[&EditField::Languages]
            .update(cx, |input, cx| input.set_value(languages, window, cx));
        self.settings_ui.inputs[&EditField::LocalModel]
            .update(cx, |input, cx| input.set_value(model, window, cx));
        self.settings_ui.prompt.update(cx, |input, cx| {
            input.set_value(
                value.prompt_draft.or(value.prompt).unwrap_or_default(),
                window,
                cx,
            )
        });
    }
    pub(crate) fn settings_have_problem(&self) -> bool {
        [
            PreferenceGroup::Generation,
            PreferenceGroup::Application,
            PreferenceGroup::Services,
        ]
        .into_iter()
        .any(|group| self.preferences.is_blocked(group))
            || self.preferences.generation_intent().is_some()
            || self.preferences.application_intent().is_some()
            || self.settings_ui.pending_generation.is_some()
            || self.settings_ui.pending_application.is_some()
            || self
                .settings_ui
                .feedback
                .values()
                .any(|(_, error, _)| *error)
            || self
                .settings_ui
                .editor
                .as_ref()
                .is_some_and(|editor| editor.save_failed)
    }

    fn select_settings_tab(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        self.settings_tab = index;
        self.scrolls[Page::Settings as usize].set_offset(point(px(0.), px(0.)));
        self.settings_ui.tab_focus[index].focus(window, cx);
        cx.notify();
    }

    pub fn settings_page(&mut self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        self.hydrate_settings_inputs(window, cx);
        self.ensure_settings_model_diagnostic(cx);
        if self.settings_tab == 5 {
            self.settings_tab = 1;
        }
        if self.settings_tab > 3 {
            self.settings_tab = 3;
        }
        let mut view = v_flex().w_full().max_w(px(900.)).gap_6().child(
            gpui_base::Tabs::new("settings-tabs")
                .tab_group()
                .key_context("SettingsTabs")
                .on_action(cx.listener(|this, _: &NextSettingsTab, window, cx| {
                    this.select_settings_tab((this.settings_tab + 1) % 4, window, cx)
                }))
                .on_action(cx.listener(|this, _: &PreviousSettingsTab, window, cx| {
                    this.select_settings_tab((this.settings_tab + 3) % 4, window, cx)
                }))
                .on_action(cx.listener(|this, _: &FirstSettingsTab, window, cx| {
                    this.select_settings_tab(0, window, cx)
                }))
                .on_action(cx.listener(|this, _: &LastSettingsTab, window, cx| {
                    this.select_settings_tab(3, window, cx)
                }))
                .flex()
                .gap_2()
                .flex_wrap()
                .children(
                    ["生成笔记", "服务与账号", "存储", "应用"]
                        .into_iter()
                        .enumerate()
                        .map(|(index, label)| {
                            let selected = self.settings_tab == index;
                            gpui_base::Tab::new(("settings-group", index))
                                .accessibility_label(label)
                                .set_position(index + 1, 4)
                                .selected(selected)
                                .track_focus(&self.settings_ui.tab_focus[index])
                                .tab_stop(selected)
                                .min_h(rems(2.6))
                                .h_auto()
                                .px_3()
                                .py_2()
                                .text_size(rems(1.))
                                .rounded_md()
                                .border_2()
                                .border_color(rgb(if selected { BLUE } else { LINE }))
                                .bg(rgb(if selected { TINT } else { SURFACE }))
                                .text_color(rgb(INK))
                                .focus(|style| style.border_color(rgb(INK)).shadow_sm())
                                .child(label)
                                .on_click(cx.listener(move |this, _, window, cx| {
                                    this.select_settings_tab(index, window, cx)
                                }))
                        }),
                ),
        );
        for group in [
            PreferenceGroup::Generation,
            PreferenceGroup::Services,
            PreferenceGroup::Application,
        ] {
            let target = match group {
                PreferenceGroup::Generation => 0,
                PreferenceGroup::Services => 1,
                PreferenceGroup::Application => 3,
            };
            if target != self.settings_tab
                && let Some((message, true)) = self.settings_group_notice(group)
            {
                view = view.child(
                    h_flex()
                        .gap_2()
                        .flex_wrap()
                        .child(
                            text(
                                ("settings-failed-group", target),
                                if message.starts_with(group.label()) {
                                    message
                                } else {
                                    format!("{}：{message}", group.label())
                                },
                            )
                            .flex_1()
                            .min_w_0()
                            .text_sm(),
                        )
                        .child(
                            control(("open-failed-settings-group", target))
                                .label("查看设置")
                                .on_click(cx.listener(move |this, _, window, cx| {
                                    this.select_settings_tab(target, window, cx);
                                })),
                        ),
                );
            }
        }
        view.child(
            div()
                .id("settings-panel")
                .role(Role::TabPanel)
                .aria_label(["生成笔记", "服务与账号", "存储", "应用"][self.settings_tab])
                .child(match self.settings_tab {
                    0 => self.generation_settings_page(cx),
                    1 => self.services_settings_page(cx),
                    2 => self.storage_settings_page(cx),
                    _ => self.application_settings_page(window, cx),
                }),
        )
        .into_any_element()
    }

    fn generation_settings_page(&self, cx: &mut Context<Self>) -> AnyElement {
        let value = self.preferences.generation().clone();
        let provider = value.options.provider.map(|p| p.as_str()).unwrap_or("");
        let mut view = v_flex().gap_5()
            .child(text("generation-default-scope", "这里的默认选项会同步到准备中笔记尚未单独修改的选项。已排队或开始生成的笔记保持原设置。").text_sm().text_color(rgb(MUTED)))
            .child(self.group_feedback(PreferenceGroup::Generation, cx))
            .child(group("language-settings", "文字来源")
                .child(text("subtitle-policy", "优先使用可读取的字幕；需要识别时使用下面的方式。字幕读取失败不会被当成没有字幕。").text_sm().text_color(rgb(MUTED)))
                .child(self.setting_field(EditField::Languages, "字幕优先语言", cx))
                .child(text("subtitle-language-help", "使用语言代码，以逗号分隔，例如 zh-Hans, en；留空按界面语言和来源语言选择。所有已发现的语言仍可选择。").text_sm().text_color(rgb(MUTED)))
                .child(control("apply-languages").label("应用语言偏好").self_start().on_click(cx.listener(|this,_,_,cx|this.apply_language_preferences(cx)))))
            .child(group("asr-default-settings", "需要识别视频声音时")
                .child(SingleChoiceGroup::new("default-asr-device", "默认语音识别方式").options([
                    ("","自动选择本机方式"),("coreml","Apple 原生"),("gpu","GPU"),("cpu","CPU"),("npu","Intel NPU"),("api","使用语音服务")
                ].into_iter().filter(|(id,_)| *id!="coreml" || cfg!(target_os="macos") || provider=="coreml")
                    .filter(|(id,_)| *id!="npu" || !cfg!(target_os="macos") || provider=="npu"))
                        .selected(provider.to_owned())
                        .on_change(cx.listener(move|this,id: &SharedString,_,cx|{
                            let mut next=this.generation_edit_base();
                            next.options.provider=match id.as_ref() {"coreml"=>Some(course2md::config::AsrProvider::Coreml),"gpu"=>Some(course2md::config::AsrProvider::Gpu),"cpu"=>Some(course2md::config::AsrProvider::Cpu),"npu"=>Some(course2md::config::AsrProvider::Npu),"api"=>Some(course2md::config::AsrProvider::Api),_=>None};
                            this.commit_generation(next,cx);
                        })))
                .child(text("asr-fixed-choice-help","固定选择不可用时会保留原因。自动方式只在本机调度，不会改用网络服务。").text_sm().text_color(rgb(MUTED)))
                .when(provider!="api",|view|view.child(self.local_model_picker(cx)).child(self.default_model_readiness_panel(cx)))
                .when(provider=="api",|view|view.child(self.service_picker(ServicePurpose::Speech,false,cx))));
        view=view.child(group("ai-default-settings","AI 校对与摘要")
            .child(preference("AI 校对","修正识别错误和标点，保留原意与原语言。",Switch::new("default-ai-proofread").checked(value.ai_proofread)
                .on_click(cx.listener(|this,enabled,_,cx|{let mut next=this.generation_edit_base();next.ai_proofread=*enabled;this.commit_generation(next,cx);}))))
            .child(preference("生成摘要","提炼课程要点并放在笔记开头，原正文继续保留。",Switch::new("default-ai-summary").checked(value.ai_summary)
                .on_click(cx.listener(|this,enabled,_,cx|{let mut next=this.generation_edit_base();next.ai_summary=*enabled;this.commit_generation(next,cx);}))))
            .when(value.needs_ai(),|view|view.child(self.service_picker(ServicePurpose::Ai,false,cx)))
            .when(value.ai_proofread,|view|view.child(preference("发送截图辅助校对","文字及对应截图会发送到所选 AI 服务。",Switch::new("default-ai-vision").checked(value.vision)
                .on_click(cx.listener(|this,enabled,_,cx|{let mut next=this.generation_edit_base();next.vision=*enabled;this.commit_generation(next,cx);})))))
            .child(control("show-proofread-rules").ghost().label(if value.prompt.is_some(){"查看自定义校对规则"}else{"查看校对规则"}).self_start()
                .on_click(cx.listener(|this,_,_,cx|{this.settings_ui.show_prompt=!this.settings_ui.show_prompt;cx.notify();})))
            .when(self.settings_ui.show_prompt,|view|view
                .child(text("standard-proofread-rule",course2md::llm::DEFAULT_PROMPT).text_sm().text_color(rgb(MUTED)))
                .child(text("prompt-contract","自定义规则影响校对内容；段落 ID 对应和返回结构由软件固定，不会被这些规则覆盖。").text_sm().text_color(rgb(MUTED)))
                .child(Textarea::new(&self.settings_ui.prompt).h(px(160.)).w_full().aria_label("自定义校对规则"))
                .child(h_flex().gap_2().flex_wrap()
                    .child(control("apply-proofread-rules").label("应用校对规则").on_click(cx.listener(|this,_,_,cx|{
                        let prompt=this.settings_ui.prompt.read(cx).value().trim().to_owned();let mut next=this.generation_edit_base();
                        next.prompt=(!prompt.is_empty()).then_some(prompt);next.prompt_draft=None;this.commit_generation(next,cx);
                    })))
                    .child(control("restore-proofread-rules").ghost().label("恢复标准规则").on_click(cx.listener(|this,_,window,cx|{
                        let mut next=this.generation_edit_base();next.prompt=None;next.prompt_draft=None;
                        if this.commit_generation(next,cx){this.settings_ui.prompt.update(cx,|input,cx|input.set_value("",window,cx));}
                    }))))));
        let selected = value.options.formats.clone().unwrap_or_default();
        view.child(group("export-default-settings","同时导出文件")
            .child(text("internal-note-policy","每次都会保存可阅读的笔记，完成后可随时导出。以下选择会同时生成额外文件。").text_sm().text_color(rgb(MUTED)))
            .child(h_flex().gap_2().flex_wrap().children([
                (course2md::config::OutputFormat::Md,"Markdown 包"),(course2md::config::OutputFormat::Html,"网页文件"),(course2md::config::OutputFormat::Json,"JSON 数据")
            ].into_iter().enumerate().map(|(index,(format,label))|Checkbox::new(("default-export",index)).label(label).checked(selected.contains(&format)).min_h(rems(2.6))
                .on_click(cx.listener(move|this,enabled,_,cx|{let mut next=this.generation_edit_base();let formats=next.options.formats.get_or_insert_with(Vec::new);
                    if !enabled{formats.retain(|value|*value!=format);}else if !formats.contains(&format){formats.push(format);}this.commit_generation(next,cx);
                })))))
            .child(text("export-format-purpose","Markdown 包包含图片资源；网页文件内嵌图片；JSON 数据供程序使用。全部不选也会生成内部笔记。").text_sm().text_color(rgb(MUTED)))
            .child(preference("保留视频供离线播放","仅用于在线来源；生成后保留下载的视频，会占用额外空间。",Switch::new("default-keep-video").checked(value.options.keep_video.unwrap_or(false))
                .on_click(cx.listener(|this,enabled,_,cx|{let mut next=this.generation_edit_base();next.options.keep_video=Some(*enabled);this.commit_generation(next,cx);}))))).into_any_element()
    }

    fn local_model_picker(&self, cx: &mut Context<Self>) -> Div {
        use course2md::config::AsrProvider;
        let preferences = self.preferences.generation();
        let provider = preferences.options.provider;
        let selected = preferences
            .options
            .asr_model
            .as_deref()
            .unwrap_or("qwen3-1.7b");
        let mut models = vec![("qwen3-1.7b", "Qwen3-ASR 1.7B")];
        if provider == Some(AsrProvider::Coreml)
            || (provider.is_none() && cfg!(target_os = "macos"))
            || provider == Some(AsrProvider::Npu)
        {
            models.extend([("qwen3-0.6b", "Qwen3-ASR 0.6B"), ("whisper", "Whisper")]);
        }
        if provider == Some(AsrProvider::Npu) {
            models.extend([
                ("whisper-tiny", "Whisper Tiny"),
                ("whisper-base", "Whisper Base"),
                ("whisper-small", "Whisper Small"),
            ]);
        }
        let known = models.iter().any(|(id, _)| *id == selected);
        let mut view = v_flex()
            .gap_2()
            .child(text("local-model-heading", "识别模型"))
            .child(
                SingleChoiceGroup::new("default-local-model", "默认识别模型")
                    .options(models)
                    .selected(selected.to_owned())
                    .on_change(cx.listener(move |this, id: &SharedString, window, cx| {
                        let mut next = this.generation_edit_base();
                        next.options.asr_model = Some(id.to_string());
                        next.local_model_draft = None;
                        if this.commit_generation(next, cx) {
                            this.settings_ui.inputs[&EditField::LocalModel]
                                .update(cx, |input, cx| input.set_value(id.clone(), window, cx));
                        }
                    })),
            );
        if !known && provider != Some(AsrProvider::Npu) {
            view = view.child(text("unavailable-fixed-model", format!("已保留指定模型 {selected}，当前识别方式不支持此模型。请选择上面的实际模型。")).text_sm().text_color(rgb(0xa32626)));
        }
        if matches!(provider, Some(AsrProvider::Cpu | AsrProvider::Gpu)) {
            view = view.child(
                text(
                    "local-gguf-model",
                    "此识别方式使用 Qwen3-ASR-1.7B Q8_0 GGUF。模型缺失时会在生成前准备。",
                )
                .text_sm()
                .text_color(rgb(MUTED)),
            );
        }
        if provider == Some(AsrProvider::Npu) {
            view = view
                .child(self.setting_field(EditField::LocalModel, "NPU 模型 ID 或仓库 ID", cx))
                .child(
                    control("apply-custom-npu-model")
                        .label("应用模型选择")
                        .self_start()
                        .on_click(cx.listener(|this, _, _, cx| {
                            let model = this
                                .setting_value(EditField::LocalModel, cx)
                                .trim()
                                .to_owned();
                            let mut next = this.generation_edit_base();
                            next.options.asr_model = (!model.is_empty()).then_some(model);
                            next.local_model_draft = None;
                            this.commit_generation(next, cx);
                        })),
                );
        }
        if let Some(environment) = &self.environment {
            let ready = match provider {
                Some(AsrProvider::Coreml) => environment.apple && environment.engine,
                Some(AsrProvider::Gpu) => {
                    environment.gpu.is_some() && environment.llama && environment.engine
                }
                Some(AsrProvider::Cpu) => environment.llama && environment.engine,
                Some(AsrProvider::Npu) => environment.npu && environment.engine,
                _ => true,
            };
            if !ready {
                view = view.child(text("selected-device-unavailable", "尚未检测到所选识别方式需要的运行环境。已保留这个选择，可在应用的诊断详情中查看原因。").text_sm().text_color(rgb(0x945000)));
            }
        }
        view
    }

    fn services_settings_page(&self, cx: &mut Context<Self>) -> AnyElement {
        let mut view=v_flex().gap_5().child(self.group_feedback(PreferenceGroup::Services,cx))
            .child(text("service-purpose-help","服务配置保存后才会使用。测试是单独的主动操作，不会在保存或生成前暗中发送测试内容。").text_sm().text_color(rgb(MUTED)))
            .child(h_flex().gap_2().flex_wrap()
                .child(control("add-speech-service").label("添加语音服务").on_click(cx.listener(|this,_,window,cx|this.open_settings_service_editor(ServicePurpose::Speech,None,window,cx))))
                .child(control("add-ai-service").label("添加 AI 服务").on_click(cx.listener(|this,_,window,cx|this.open_settings_service_editor(ServicePurpose::Ai,None,window,cx)))));
        let mut latest = BTreeMap::<String, ServiceVersion>::new();
        for version in self.preferences.versions() {
            if latest
                .get(&version.service_id)
                .is_none_or(|old| old.number < version.number)
            {
                latest.insert(version.service_id.clone(), version.clone());
            }
        }
        for (index, version) in latest.values().enumerate() {
            let stopped = self.preferences.is_service_stopped(&version.service_id);
            let id = version.id.clone();
            let service_id = version.service_id.clone();
            let purpose = version.config.protocol.purpose();
            let defaults = self.preferences.default_refs();
            let default_version = match purpose {
                ServicePurpose::Speech => defaults.asr.as_deref(),
                ServicePurpose::Ai => defaults.llm.as_deref(),
            }
            .and_then(|id| self.preferences.version(id))
            .filter(|default| default.service_id == version.service_id);
            let default_description = default_version
                .map(|default| {
                    if default.id == version.id {
                        " · 默认服务".to_owned()
                    } else {
                        format!(" · 默认仍使用版本 {}", default.number)
                    }
                })
                .unwrap_or_default();
            let kinds: &[TestKind] = if purpose == ServicePurpose::Speech {
                &[TestKind::Speech]
            } else {
                &[TestKind::Proofread, TestKind::Summary, TestKind::Vision]
            };
            let tests: Vec<String> = kinds
                .iter()
                .filter_map(|kind| {
                    self.preferences
                        .test_evidence(&version.config, kind.contract())
                        .map(|evidence| format!("上次{}测试：{}", kind.label(), evidence.message))
                })
                .collect();
            view = view.child(
                v_flex()
                    .gap_2()
                    .py_4()
                    .border_t_1()
                    .border_color(rgb(LINE))
                    .child(
                        text(
                            ("saved-service-title", index),
                            format!(
                                "{}{}{}",
                                version.config.name,
                                default_description,
                                if stopped { " · 已停用" } else { "" }
                            ),
                        )
                        .font_weight(FontWeight::SEMIBOLD),
                    )
                    .child(
                        text(
                            ("saved-service-description", index),
                            format!(
                                "{} · {} · {} · 版本 {}",
                                if purpose == ServicePurpose::Speech {
                                    "语音识别"
                                } else {
                                    "AI 校对与摘要"
                                },
                                version.config.host(),
                                version.config.model,
                                version.number
                            ),
                        )
                        .text_sm()
                        .text_color(rgb(MUTED)),
                    )
                    .child(
                        text(
                            ("saved-service-test-status", index),
                            if tests.is_empty() {
                                "尚未测试。可以直接用于生成，不会自动发送测试请求。".to_owned()
                            } else {
                                tests.join("\n")
                            },
                        )
                        .text_sm()
                        .text_color(rgb(MUTED)),
                    )
                    .when(!stopped, |view| {
                        view.child(
                            h_flex()
                                .gap_2()
                                .flex_wrap()
                                .child(
                                    control(("edit-saved-service", index))
                                        .label("编辑服务")
                                        .on_click(cx.listener(move |this, _, window, cx| {
                                            this.open_settings_service_editor(
                                                purpose,
                                                Some(id.clone()),
                                                window,
                                                cx,
                                            )
                                        })),
                                )
                                .child(
                                    control(("stop-saved-service", index))
                                        .ghost()
                                        .label("停止使用此服务")
                                        .on_click(cx.listener(move |this, _, window, cx| {
                                            this.confirm_stop_service(
                                                service_id.clone(),
                                                window,
                                                cx,
                                            )
                                        })),
                                ),
                        )
                    }),
            );
        }
        for (index, draft) in self.preferences.service_drafts().cloned().enumerate() {
            let id = draft.id.clone();
            let label = if draft.name.is_empty() {
                format!(
                    "{}草稿",
                    if draft.protocol.purpose() == ServicePurpose::Speech {
                        "语音服务"
                    } else {
                        "AI 服务"
                    }
                )
            } else {
                format!("{} · 草稿", draft.name)
            };
            view = view.child(
                h_flex()
                    .gap_3()
                    .items_center()
                    .child(text(("saved-service-draft", index), label).flex_1())
                    .child(
                        control(("continue-service-draft", index))
                            .label("继续填写")
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.open_existing_service_draft(&id, None, window, cx)
                            })),
                    ),
            );
        }
        view.child(
            group("account-settings-heading", "来源账号").child(self.account_settings_page(cx)),
        )
        .into_any_element()
    }
    pub fn task_service_picker(&self, purpose: ServicePurpose, cx: &mut Context<Self>) -> Div {
        self.service_picker(purpose, true, cx)
    }
    pub fn selected_task_service(&self, purpose: ServicePurpose) -> Option<ServiceVersion> {
        let fixed = self
            .workspace
            .as_ref()
            .and_then(|w| w.state.draft())
            .and_then(|draft| match purpose {
                ServicePurpose::Speech => draft.asr_service.clone(),
                ServicePurpose::Ai => draft.ai_service.clone(),
            });
        let defaults = self.preferences.default_refs();
        let id = fixed.or(match purpose {
            ServicePurpose::Speech => defaults.asr,
            ServicePurpose::Ai => defaults.llm,
        })?;
        self.preferences.version(&id).cloned()
    }
    fn service_picker(
        &self,
        purpose: ServicePurpose,
        current_task: bool,
        cx: &mut Context<Self>,
    ) -> Div {
        let refs = self.preferences.default_refs();
        let current = if current_task {
            self.selected_task_service(purpose).map(|v| v.id)
        } else {
            match purpose {
                ServicePurpose::Speech => refs.asr,
                ServicePurpose::Ai => refs.llm,
            }
        };
        let mut latest = BTreeMap::<String, ServiceVersion>::new();
        for version in self.preferences.versions().filter(|v| {
            v.config.protocol.purpose() == purpose
                && !self.preferences.is_service_stopped(&v.service_id)
        }) {
            if latest
                .get(&version.service_id)
                .is_none_or(|old| old.number < version.number)
            {
                latest.insert(version.service_id.clone(), version.clone());
            }
        }
        let mut view = v_flex().gap_2();
        if let Some(version) = current
            .as_deref()
            .and_then(|id| self.preferences.version(id))
        {
            view = view.child(
                text(
                    (
                        "selected-service",
                        purpose as usize * 2 + current_task as usize,
                    ),
                    format!(
                        "{} · {} · {} · 版本 {}{}",
                        version.config.name,
                        version.config.host(),
                        version.config.model,
                        version.number,
                        if self.preferences.is_service_stopped(&version.service_id) {
                            " · 已停用"
                        } else {
                            ""
                        }
                    ),
                )
                .text_sm(),
            );
        }
        let mut choices = latest.into_values().collect::<Vec<_>>();
        if let Some(current) = current
            .as_deref()
            .and_then(|id| self.preferences.version(id))
            && !choices.iter().any(|version| version.id == current.id)
        {
            choices.push(current.clone());
        }
        if !choices.is_empty() {
            let mut picker = SingleChoiceGroup::new(
                (
                    "service-choice",
                    purpose as usize * 2 + current_task as usize,
                ),
                if purpose == ServicePurpose::Speech {
                    "使用的语音服务"
                } else {
                    "使用的 AI 服务"
                },
            )
            .options(choices.iter().map(|version| {
                (
                    version.id.clone(),
                    format!(
                        "{} · 版本 {}{}",
                        version.config.name,
                        version.number,
                        if self.preferences.is_service_stopped(&version.service_id) {
                            " · 已停用"
                        } else {
                            ""
                        }
                    ),
                )
            }))
            .when_some(current, |picker, current| picker.selected(current))
            .on_change(cx.listener(move |this, id: &SharedString, _, cx| {
                this.bind_service(purpose, Some(id.to_string()), current_task, cx);
            }));
            for version in &choices {
                if self.preferences.is_service_stopped(&version.service_id) {
                    picker = picker.disable_option(&version.id);
                }
            }
            view = view.child(picker);
        }
        view = view.child(
            control((
                "configure-service",
                purpose as usize * 2 + current_task as usize,
            ))
            .ghost()
            .label(if purpose == ServicePurpose::Speech {
                "设置语音服务"
            } else {
                "设置 AI 服务"
            })
            .self_start()
            .on_click(cx.listener(move |this, _, window, cx| {
                if current_task {
                    this.open_task_service_editor(purpose, window, cx)
                } else {
                    this.open_settings_service_editor(purpose, None, window, cx)
                }
            })),
        );
        if current_task
            && self
                .workspace
                .as_ref()
                .and_then(|w| w.state.draft())
                .is_some_and(|draft| match purpose {
                    ServicePurpose::Speech => draft.asr_service.is_some(),
                    ServicePurpose::Ai => draft.ai_service.is_some(),
                })
        {
            view = view.child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(
                        text(
                            ("service-only-this-note", purpose as usize),
                            "仅用于这次笔记",
                        )
                        .text_sm()
                        .text_color(rgb(MUTED)),
                    )
                    .child(
                        control(("inherit-default-service", purpose as usize))
                            .ghost()
                            .label("恢复默认")
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.bind_service(purpose, None, true, cx);
                            })),
                    ),
            );
        }
        if let Some((message, true, _)) = self.settings_ui.feedback.get(&PreferenceGroup::Services)
        {
            view = view.child(
                text(("task-service-error", purpose as usize), message.clone())
                    .text_sm()
                    .text_color(rgb(0xa32626)),
            );
        }
        view
    }
    fn bind_service(
        &mut self,
        purpose: ServicePurpose,
        id: Option<String>,
        current_task: bool,
        cx: &mut Context<Self>,
    ) -> bool {
        let result = if current_task {
            self.workspace
                .as_mut()
                .ok_or_else(|| anyhow!("草稿记录暂时不可用"))
                .and_then(|workspace| {
                    workspace.transaction(|state| {
                        let draft = state.draft_mut().context("找不到当前草稿")?;
                        match purpose {
                            ServicePurpose::Speech => draft.asr_service = id,
                            ServicePurpose::Ai => draft.ai_service = id,
                        };
                        Ok(())
                    })
                })
        } else {
            self.preferences.set_default_service(purpose, id.as_deref())
        };
        match result {
            Ok(()) => {
                cx.notify();
                true
            }
            Err(error) => {
                self.set_settings_feedback(
                    PreferenceGroup::Services,
                    format!("服务选择尚未保存：{error:#}"),
                    true,
                );
                cx.notify();
                false
            }
        }
    }

    pub fn open_task_service_editor(
        &mut self,
        purpose: ServicePurpose,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.save_current_draft(cx) {
            return;
        }
        let Some(target) = self
            .workspace
            .as_ref()
            .and_then(|w| w.state.draft())
            .map(|d| d.id.clone())
        else {
            return;
        };
        let draft = self
            .selected_task_service(purpose)
            .filter(|version| !self.preferences.is_service_stopped(&version.service_id))
            .map(|v| ServiceDraft::from_version(&v))
            .unwrap_or_else(|| ServiceDraft::new(purpose));
        self.open_service_draft(draft, Some(target), window, cx);
    }
    fn open_settings_service_editor(
        &mut self,
        purpose: ServicePurpose,
        version: Option<String>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let draft = version
            .as_deref()
            .and_then(|id| self.preferences.version(id))
            .map(ServiceDraft::from_version)
            .unwrap_or_else(|| ServiceDraft::new(purpose));
        self.open_service_draft(draft, None, window, cx);
    }
    fn open_existing_service_draft(
        &mut self,
        id: &str,
        target: Option<String>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(draft) = self.preferences.draft(id).cloned() {
            self.open_service_draft(draft, target, window, cx);
        }
    }
    fn open_service_draft(
        &mut self,
        draft: ServiceDraft,
        target: Option<String>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.settings_ui.editor.is_some() {
            return;
        }
        let title = match (draft.protocol.purpose(), target.is_some()) {
            (ServicePurpose::Speech, true) => "为这次笔记设置语音服务",
            (ServicePurpose::Ai, true) => "为这次笔记设置 AI 服务",
            (ServicePurpose::Speech, false) => "设置语音服务",
            (ServicePurpose::Ai, false) => "设置 AI 服务",
        };
        for (field, value) in [
            (EditField::Name, draft.name.clone()),
            (EditField::Address, draft.address.clone()),
            (EditField::Model, draft.model.clone()),
            (EditField::Key, String::new()),
        ] {
            self.settings_ui.inputs[&field]
                .update(cx, |input, cx| input.set_value(value, window, cx));
        }
        self.settings_ui.inputs[&EditField::Key]
            .update(cx, |input, cx| input.set_masked(true, window, cx));
        let kind = if draft.protocol.purpose() == ServicePurpose::Speech {
            TestKind::Speech
        } else {
            TestKind::Proofread
        };
        let evidence = draft.configuration().ok().and_then(|config| {
            self.preferences
                .test_evidence(&config, kind.contract())
                .cloned()
        });
        self.settings_ui.editor = Some(ServiceEditor {
            draft,
            target,
            also_default: false,
            return_focus: window.focused(cx),
            errors: Vec::new(),
            status: None,
            test_kind: kind,
            test_running: None,
            evidence,
            pending_binding: None,
            show_key: false,
            save_failed: false,
        });
        self.save_open_service_draft(false, window, cx);
        let desktop = cx.entity();
        let content = cx.new(|cx| ServiceDialog {
            _observation: cx.observe(&desktop, |_, _, cx| cx.notify()),
            desktop,
        });
        let weak = cx.weak_entity();
        window.open_dialog(cx, move |sheet, _, _| {
            let closed = weak.clone();
            let cancel = weak.clone();
            let submit = weak.clone();
            sheet
                .title(title)
                .w(px(620.))
                .margin_top(px(24.))
                .overlay_closable(false)
                .close_button(false)
                .child(content.clone())
                .on_ok(move |_, window, cx| {
                    let _ = submit.update(cx, |this, cx| this.publish_open_service(window, cx));
                    false
                })
                .on_cancel(move |_, window, cx| {
                    cancel
                        .update(cx, |this, cx| this.close_service_editor(window, cx))
                        .unwrap_or(true)
                })
                .on_close(move |_, window, cx| {
                    let _ = closed.update(cx, |this, cx| this.restore_service_focus(window, cx));
                })
        });
        self.settings_ui.inputs[&EditField::Address]
            .update(cx, |input, cx| input.focus(window, cx));
        cx.notify();
    }
    fn service_editor_content(&self, window: &Window, cx: &mut Context<Self>) -> AnyElement {
        let Some(editor) = &self.settings_ui.editor else {
            return div().into_any_element();
        };
        let protocol = editor.draft.protocol;
        let auth = editor.draft.authentication;
        let busy = editor.test_running.is_some();
        let awaiting_binding = editor.pending_binding.is_some();
        let changed = self.service_draft_changed(&editor.draft)
            || !self.setting_value(EditField::Key, cx).is_empty();
        let mut view = v_flex()
            .id("service-editor-body")
            .role(Role::Dialog)
            .aria_label(if protocol.purpose() == ServicePurpose::Speech {
                "设置语音服务"
            } else {
                "设置 AI 服务"
            })
            .gap_4()
            .max_h((window.bounds().size.height - px(150.)).max(px(180.)))
            .overflow_y_scroll()
            .px_1()
            .child(
                text(
                    "service-editor-scope",
                    if editor.target.is_some() {
                        "保存后仅用于这份准备中的笔记，已提交任务保持原版本。"
                    } else {
                        "保存后设为默认服务；已排队和开始生成的笔记保持原版本。"
                    },
                )
                .text_sm()
                .text_color(rgb(MUTED)),
            );
        if let Some(old) = editor
            .draft
            .based_on
            .as_deref()
            .and_then(|id| self.preferences.version(id))
        {
            view = view.child(
                text(
                    "service-previous-version",
                    format!(
                        "{}{} · {} · 版本 {}",
                        if changed {
                            "修改尚未启用；当前仍使用 "
                        } else {
                            "当前使用 "
                        },
                        old.config.host(),
                        old.config.model,
                        old.number
                    ),
                )
                .text_sm(),
            );
        }
        view = view
            .child(self.setting_field(EditField::Name, "服务名称", cx))
            .child(
                v_flex()
                    .gap_2()
                    .child(text("service-protocol-heading", "接口类型"))
                    .child(
                        SingleChoiceGroup::new("service-protocol", "服务接口类型")
                            .options(
                                [
                                    ServiceProtocol::SpeechTranscriptions,
                                    ServiceProtocol::SpeechChat,
                                    ServiceProtocol::AiChat,
                                ]
                                .into_iter()
                                .filter(|candidate| candidate.purpose() == protocol.purpose())
                                .map(|candidate| (candidate.label(), candidate.label())),
                            )
                            .selected(protocol.label())
                            .disabled(awaiting_binding)
                            .on_change(cx.listener(
                                move |this, selected: &SharedString, window, cx| {
                                    let Some(candidate) = [
                                        ServiceProtocol::SpeechTranscriptions,
                                        ServiceProtocol::SpeechChat,
                                        ServiceProtocol::AiChat,
                                    ]
                                    .into_iter()
                                    .find(|candidate| candidate.label() == selected.as_ref()) else {
                                        return;
                                    };
                                    if let Some(editor) = &mut this.settings_ui.editor {
                                        editor.draft.protocol = candidate;
                                        editor.errors.clear();
                                        editor.evidence = None;
                                    }
                                    this.save_open_service_draft(false, window, cx);
                                },
                            )),
                    ),
            )
            .child(self.setting_field(EditField::Address, "服务地址", cx));
        if let Ok(endpoint) =
            preferences::normalize_endpoint(&self.setting_value(EditField::Address, cx), protocol)
        {
            view = view.child(
                text("service-actual-endpoint", format!("将请求 {endpoint}"))
                    .text_sm()
                    .text_color(rgb(MUTED)),
            );
        }
        view = view
            .child(self.setting_field(EditField::Model, "模型 ID", cx))
            .child(
                v_flex()
                    .gap_2()
                    .child(text("service-auth-heading", "认证方式"))
                    .child(
                        SingleChoiceGroup::new("service-auth-mode", "服务认证方式")
                            .options([("api_key", "API Key"), ("none", "无需认证")])
                            .selected(if auth == Authentication::ApiKey {
                                "api_key"
                            } else {
                                "none"
                            })
                            .disabled(awaiting_binding)
                            .on_change(cx.listener(
                                move |this, selected: &SharedString, window, cx| {
                                    let mode = if selected.as_ref() == "api_key" {
                                        Authentication::ApiKey
                                    } else {
                                        Authentication::None
                                    };
                                    if let Some(editor) = &mut this.settings_ui.editor {
                                        editor.draft.authentication = mode;
                                        editor.errors.clear();
                                        editor.evidence = None;
                                    }
                                    this.save_open_service_draft(false, window, cx);
                                },
                            )),
                    ),
            );
        if auth == Authentication::ApiKey {
            view = view.child(self.setting_field(EditField::Key, "API Key", cx));
            for (index, name) in
                preferences::Store::available_environment_credentials(protocol.purpose())
                    .into_iter()
                    .enumerate()
            {
                view = view.child(
                    control(("capture-environment-key", index))
                        .label(format!("使用环境变量 {name}"))
                        .disabled(awaiting_binding)
                        .self_start()
                        .on_click(cx.listener(move |this, _, window, cx| {
                            let Some(draft) = this
                                .settings_ui
                                .editor
                                .as_ref()
                                .map(|editor| editor.draft.clone())
                            else {
                                return;
                            };
                            match this.preferences.capture_environment_credential(draft, name) {
                                Ok(draft) => {
                                    if let Some(editor) = &mut this.settings_ui.editor {
                                        editor.draft = draft;
                                        editor.evidence = None;
                                        editor.save_failed = false;
                                        editor.status = Some(
                                            "已将所选环境变量保存为安全凭据；保存服务后生效".into(),
                                        );
                                    }
                                    this.settings_ui.inputs[&EditField::Key]
                                        .update(cx, |input, cx| input.set_value("", window, cx));
                                }
                                Err(error) => {
                                    if let Some(editor) = &mut this.settings_ui.editor {
                                        editor.status = Some(format!("凭据尚未保存：{error:#}"));
                                    }
                                }
                            }
                            cx.notify();
                        })),
                );
            }
            if editor.draft.credential.is_some() {
                view = view.child(
                    text(
                        "service-key-existing",
                        "API Key 已安全保存。填写新值可替换，留空保留原密钥。",
                    )
                    .text_sm()
                    .text_color(rgb(MUTED)),
                );
            }
            view = view.child(
                control("toggle-service-key")
                    .ghost()
                    .label(if editor.show_key {
                        "隐藏输入的密钥"
                    } else {
                        "显示输入的密钥"
                    })
                    .self_start()
                    .on_click(cx.listener(|this, _, window, cx| {
                        if let Some(editor) = &mut this.settings_ui.editor {
                            editor.show_key = !editor.show_key;
                            let masked = !editor.show_key;
                            this.settings_ui.inputs[&EditField::Key]
                                .update(cx, |input, cx| input.set_masked(masked, window, cx));
                        }
                        cx.notify();
                    })),
            );
        }
        if let Some(source) = &editor.draft.credential_source {
            view = view.child(
                text(
                    "service-credential-source",
                    format!("凭据来源：{source}；已固定为安全存储版本"),
                )
                .text_sm(),
            );
        }
        if editor.target.is_some() {
            view = view.child(preference(
                "同时设为默认服务",
                "准备中笔记未单独选择的服务会采用这个版本。",
                Switch::new("service-also-default")
                    .checked(editor.also_default)
                    .disabled(awaiting_binding)
                    .on_click(cx.listener(|this, value, _, cx| {
                        if let Some(editor) = &mut this.settings_ui.editor {
                            editor.also_default = *value;
                        }
                        cx.notify();
                    })),
            ));
        }
        view = view.child(
            text("service-test-notice", service_test::TEST_NOTICE)
                .text_sm()
                .text_color(rgb(MUTED)),
        );
        if protocol == ServiceProtocol::AiChat {
            view = view.child(
                SingleChoiceGroup::new("service-test-purpose", "要测试的服务能力")
                    .options(
                        [TestKind::Proofread, TestKind::Summary, TestKind::Vision]
                            .into_iter()
                            .map(|kind| (kind.contract(), kind.label())),
                    )
                    .selected(editor.test_kind.contract())
                    .disabled(busy)
                    .on_change(cx.listener(move |this, selected: &SharedString, _, cx| {
                        let Some(kind) = [TestKind::Proofread, TestKind::Summary, TestKind::Vision]
                            .into_iter()
                            .find(|kind| kind.contract() == selected.as_ref())
                        else {
                            return;
                        };
                        let evidence = this
                            .settings_ui
                            .editor
                            .as_ref()
                            .filter(|_| this.setting_value(EditField::Key, cx).is_empty())
                            .and_then(|editor| editor.draft.configuration().ok())
                            .and_then(|config| {
                                this.preferences
                                    .test_evidence(&config, kind.contract())
                                    .cloned()
                            });
                        if let Some(editor) = &mut this.settings_ui.editor {
                            editor.test_kind = kind;
                            editor.evidence = evidence;
                        }
                        cx.notify();
                    })),
            );
        }
        if let Some(evidence) = &editor.evidence {
            view = view.child(text("service-test-result", evidence.message.clone()).text_sm());
            for (index, detail) in evidence.details.iter().enumerate() {
                view = view.child(
                    text(("service-test-detail", index), detail.clone())
                        .text_sm()
                        .text_color(rgb(MUTED)),
                );
            }
            if evidence.outcome == preferences::TestOutcome::Passed {
                view = view.child(
                    text(
                        "service-tested-not-saved",
                        "测试通过；保存服务后才会使用这些更改。",
                    )
                    .text_sm(),
                );
            }
        }
        if let Some(status) = &editor.status
            && (changed
                || !matches!(
                    status.as_str(),
                    "修改尚未启用" | "草稿已保存，修改尚未启用" | "密钥修改尚未保存"
                ))
        {
            view = view.child(text("service-editor-status", status.clone()).text_sm());
        }
        view.child(
            h_flex()
                .gap_2()
                .flex_wrap()
                .child(
                    control("run-service-test")
                        .label("测试服务")
                        .loading(busy)
                        .disabled(busy || awaiting_binding)
                        .on_click(
                            cx.listener(|this, _, window, cx| this.start_service_test(window, cx)),
                        ),
                )
                .child(
                    control("save-service")
                        .primary()
                        .label(if awaiting_binding {
                            "重试保存本次选择"
                        } else {
                            "保存服务"
                        })
                        .on_click(
                            cx.listener(|this, _, window, cx| {
                                this.publish_open_service(window, cx)
                            }),
                        ),
                )
                .child(
                    control("close-service-editor")
                        .ghost()
                        .label("返回")
                        .on_click(cx.listener(|this, _, window, cx| {
                            if this.close_service_editor(window, cx) {
                                window.close_dialog(cx);
                            }
                        })),
                )
                .when(editor.save_failed, |view| {
                    view.child(
                        control("discard-unsaved-service-changes")
                            .ghost()
                            .label("放弃未保存的修改并返回")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.settings_ui.inputs[&EditField::Key]
                                    .update(cx, |input, cx| input.set_value("", window, cx));
                                this.restore_service_focus(window, cx);
                                window.close_dialog(cx);
                            })),
                    )
                }),
        )
        .into_any_element()
    }
    fn setting_input_changed(&mut self, field: EditField, cx: &mut Context<Self>) {
        if !self.settings_ui.initialized
            && matches!(field, EditField::LocalModel | EditField::Languages)
        {
            return;
        }
        if matches!(field, EditField::LocalModel | EditField::Languages) {
            let input = self.setting_value(field, cx);
            let mut next = self.generation_edit_base();
            let previous = if field == EditField::LocalModel {
                next.local_model_draft
                    .clone()
                    .unwrap_or_else(|| next.options.asr_model.clone().unwrap_or_default())
            } else {
                next.subtitle_languages_draft
                    .clone()
                    .unwrap_or_else(|| next.preferred_subtitle_languages.join(", "))
            };
            if previous == input {
                return;
            }
            if field == EditField::LocalModel {
                next.local_model_draft = Some(input);
            } else {
                next.subtitle_languages_draft = Some(input);
            }
            self.save_generation_text_draft(next, cx);
            return;
        }
        let input = self.setting_value(field, cx);
        if let Some(editor) = &mut self.settings_ui.editor {
            match field {
                EditField::Name => editor.draft.name = input,
                EditField::Address => editor.draft.address = input,
                EditField::Model => editor.draft.model = input,
                _ => {}
            }
            editor.errors.clear();
            if field != EditField::Name {
                editor.evidence = None;
            }
            editor.status = Some(
                if field == EditField::Key {
                    "密钥修改尚未保存"
                } else {
                    "修改尚未启用"
                }
                .into(),
            );
        }
        cx.notify();
    }
    fn save_prompt_draft(&mut self, cx: &mut Context<Self>) {
        if !self.settings_ui.initialized {
            return;
        }
        let text = self.settings_ui.prompt.read(cx).value().to_string();
        let mut next = self.generation_edit_base();
        if next
            .prompt_draft
            .as_ref()
            .or(next.prompt.as_ref())
            .map(String::as_str)
            .unwrap_or("")
            == text
        {
            return;
        }
        next.prompt_draft = Some(text);
        self.save_generation_text_draft(next, cx);
    }
    fn save_generation_text_draft(&mut self, next: GenerationPreferences, cx: &mut Context<Self>) {
        let previous = self
            .ordinary_preferences_submit_issue()
            .map(|issue| issue.message);
        let had_pending = self.settings_ui.pending_generation.is_some()
            || self.preferences.generation_intent().is_some();
        match self.preferences.save_generation(next.clone()) {
            Ok(()) => {
                self.settings_ui.pending_generation = None;
                self.set_settings_feedback(
                    PreferenceGroup::Generation,
                    "草稿已保存，应用后生效".into(),
                    false,
                );
                if had_pending {
                    self.refresh_preference_defaults(cx);
                }
                self.clear_resolved_generation_errors(previous);
            }
            Err(error) => {
                self.settings_ui.pending_generation = Some(next);
                self.set_settings_save_failure(PreferenceGroup::Generation, &error);
            }
        }
        cx.notify();
    }
    fn save_open_service_draft(
        &mut self,
        include_key: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(editor) = &self.settings_ui.editor else {
            return true;
        };
        if editor.pending_binding.is_some() {
            return true;
        }
        let mut draft = editor.draft.clone();
        draft.name = self.setting_value(EditField::Name, cx);
        draft.address = self.setting_value(EditField::Address, cx);
        draft.model = self.setting_value(EditField::Model, cx);
        let key = include_key
            .then(|| self.setting_value(EditField::Key, cx))
            .filter(|key| !key.trim().is_empty());
        if key.is_none()
            && draft.revision == 0
            && draft.name.is_empty()
            && draft.address.is_empty()
            && draft.model.is_empty()
            && draft.credential.is_none()
        {
            return true;
        }
        if key.is_none() && self.preferences.draft(&draft.id) == Some(&draft) {
            return true;
        }
        match self
            .preferences
            .save_service_draft(draft, key.map(Secret::new))
        {
            Ok(draft) => {
                if let Some(editor) = &mut self.settings_ui.editor {
                    editor.draft = draft;
                    editor.save_failed = false;
                    editor.status = Some(
                        if self.preferences.is_recovery_draft(&editor.draft.id) {
                            "草稿已保存到恢复记录，修改尚未启用"
                        } else {
                            "草稿已保存，修改尚未启用"
                        }
                        .into(),
                    );
                }
                if include_key {
                    self.settings_ui.inputs[&EditField::Key]
                        .update(cx, |input, cx| input.set_value("", window, cx));
                }
                cx.notify();
                true
            }
            Err(error) => {
                if let Some(editor) = &mut self.settings_ui.editor {
                    editor.status = Some(format!("这些修改尚未保存：{error:#}"));
                    editor.save_failed = true;
                }
                cx.notify();
                false
            }
        }
    }
    fn validate_service_editor(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        if !self.save_open_service_draft(true, window, cx) {
            return false;
        }
        let Some(editor) = &mut self.settings_ui.editor else {
            return false;
        };
        editor.errors = editor.draft.validate();
        if let Some(error) = editor.errors.first() {
            let field = match error.field {
                "address" => EditField::Address,
                "model" => EditField::Model,
                _ => EditField::Key,
            };
            self.settings_ui.inputs[&field].update(cx, |input, cx| input.focus(window, cx));
            cx.notify();
            false
        } else {
            true
        }
    }
    fn start_service_test(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self
            .settings_ui
            .editor
            .as_ref()
            .is_some_and(|editor| editor.test_running.is_some())
        {
            return;
        }
        if !self.validate_service_editor(window, cx) {
            return;
        }
        let Some(editor) = &mut self.settings_ui.editor else {
            return;
        };
        let Ok(config) = editor.draft.configuration() else {
            return;
        };
        let id = editor.draft.id.clone();
        let kind = editor.test_kind;
        let cancel = Arc::new(AtomicBool::new(false));
        editor.test_running = Some(cancel.clone());
        editor.evidence = None;
        editor.status = Some(format!("正在测试{}…", kind.label()));
        let vault = self.preferences.vault();
        let task = cx
            .background_executor()
            .spawn(service_test::test_service(config, kind, vault, cancel));
        cx.spawn(async move |this, cx| {
            let evidence = task.await;
            let _ = this.update(cx, |this, cx| {
                let persisted = this.preferences.record_test(evidence.clone()).is_ok();
                let key_unchanged = this.setting_value(EditField::Key, cx).is_empty();
                if let Some(editor) = &mut this.settings_ui.editor
                    && editor.draft.id == id
                {
                    editor.test_running = None;
                    editor.status = None;
                    if key_unchanged
                        && editor.draft.configuration().ok().is_some_and(|config| {
                            config.fingerprint(&evidence.contract) == evidence.fingerprint
                        })
                    {
                        editor.evidence = Some(evidence);
                        if !persisted {
                            editor.status =
                                Some("测试结果已收到，但尚未保存到设置；不会自动重测".into());
                        }
                    }
                }
                cx.notify();
            });
        })
        .detach();
        cx.notify();
    }
    fn publish_open_service(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.validate_service_editor(window, cx) {
            return;
        }
        let Some(editor) = &self.settings_ui.editor else {
            return;
        };
        let target = editor.target.clone();
        let scope = if target.is_none() || editor.also_default {
            BindingScope::Defaults
        } else {
            BindingScope::CurrentTask
        };
        let result = if let Some(version) = &editor.pending_binding {
            Ok(version.clone())
        } else {
            self.preferences.publish_service(&editor.draft.id, scope)
        };
        let version = match result {
            Ok(version) => version,
            Err(error) => {
                if let Some(editor) = &mut self.settings_ui.editor {
                    editor.status = Some(format!("服务尚未保存：{error:#}"));
                }
                cx.notify();
                return;
            }
        };
        if let Some(target) = target {
            let binding = self
                .workspace
                .as_mut()
                .ok_or_else(|| anyhow!("草稿记录暂时不可用"))
                .and_then(|workspace| {
                    workspace.transaction(|state| {
                        let draft = state
                            .drafts
                            .iter_mut()
                            .find(|draft| draft.id == target)
                            .context("原笔记草稿已不存在；服务已保存，但未改变其他笔记")?;
                        match version.config.protocol.purpose() {
                            ServicePurpose::Speech => draft.asr_service = Some(version.id.clone()),
                            ServicePurpose::Ai => draft.ai_service = Some(version.id.clone()),
                        };
                        Ok(())
                    })
                });
            if let Err(error) = binding {
                if let Some(editor) = &mut self.settings_ui.editor {
                    editor.pending_binding = Some(version);
                    editor.status = Some(format!("服务已保存，本次选择尚未保存：{error:#}"));
                }
                cx.notify();
                return;
            }
        }
        self.refresh_dispatch_controls(cx);
        self.set_settings_feedback(PreferenceGroup::Services, "服务已保存".into(), false);
        self.restore_service_focus(window, cx);
        window.close_dialog(cx);
        cx.notify();
    }
    fn close_service_editor(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        if !self.save_open_service_draft(true, window, cx) {
            return false;
        }
        self.restore_service_focus(window, cx);
        true
    }
    fn restore_service_focus(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(editor) = self.settings_ui.editor.take() {
            if let Some(cancel) = editor.test_running {
                cancel.store(true, Ordering::Release);
            }
            if let Some(focus) = editor.return_focus {
                focus.focus(window, cx);
            }
        }
        cx.notify();
    }
    fn confirm_stop_service(
        &mut self,
        service_id: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let version_ids: Vec<_> = self
            .preferences
            .versions()
            .filter(|v| v.service_id == service_id)
            .map(|v| v.id.clone())
            .collect();
        let affected = self
            .workspace
            .as_ref()
            .map(|w| {
                w.state
                    .tasks
                    .iter()
                    .filter(|task| {
                        !task.state.finished()
                            && (task
                                .plan
                                .asr_service
                                .as_ref()
                                .is_some_and(|id| version_ids.contains(id))
                                || task
                                    .plan
                                    .ai_service
                                    .as_ref()
                                    .is_some_and(|id| version_ids.contains(id)))
                    })
                    .count()
            })
            .unwrap_or(0);
        let weak = cx.weak_entity();
        window.open_alert_dialog(cx, move |dialog, _, _| {
            let weak = weak.clone();
            let id = service_id.clone();
            dialog
                .title("停止使用此服务")
                .child(text(
                    "stop-service-consequence",
                    "将不再向此主机发送新请求。已经发出的请求无法撤回，收到的结果仍会保存。",
                ))
                .when(affected > 0, |dialog| {
                    dialog.child(text(
                        "stop-service-affected",
                        format!("有 {affected} 个未完成任务引用此服务；相关后续请求会停止派发。"),
                    ))
                })
                .button_props(
                    gpui_component::dialog::DialogButtonProps::default()
                        .ok_text("停止使用")
                        .cancel_text("保留服务")
                        .show_cancel(true),
                )
                .on_ok(move |_, _, cx| {
                    weak.update(cx, |this, cx| match this.preferences.stop_service(&id) {
                        Ok(()) => {
                            this.refresh_dispatch_controls(cx);
                            this.set_settings_feedback(
                                PreferenceGroup::Services,
                                "已停止使用此服务".into(),
                                false,
                            );
                            cx.notify();
                            true
                        }
                        Err(error) => {
                            let stopped = this.preferences.is_service_stopped(&id);
                            this.set_settings_feedback(
                                PreferenceGroup::Services,
                                format!(
                                    "{}：{error:#}",
                                    if stopped {
                                        "已停止派发，停用记录尚未完整保存"
                                    } else {
                                        "尚未停止使用此服务"
                                    }
                                ),
                                true,
                            );
                            this.refresh_dispatch_controls(cx);
                            cx.notify();
                            true
                        }
                    })
                    .unwrap_or(false)
                })
        });
    }

    fn commit_generation(&mut self, next: GenerationPreferences, cx: &mut Context<Self>) -> bool {
        let previous = self
            .ordinary_preferences_submit_issue()
            .map(|issue| issue.message);
        match self.preferences.save_generation(next.clone()) {
            Ok(()) => {
                self.settings_ui.pending_generation = None;
                self.set_settings_feedback(PreferenceGroup::Generation, "已保存".into(), false);
                self.refresh_preference_defaults(cx);
                self.clear_resolved_generation_errors(previous);
                true
            }
            Err(error) => {
                self.settings_ui.pending_generation = Some(next);
                self.set_settings_save_failure(PreferenceGroup::Generation, &error);
                cx.notify();
                false
            }
        }
    }
    fn refresh_preference_defaults(&mut self, cx: &mut Context<Self>) {
        let out = self.config.defaults.out.clone();
        self.config = self.preferences.defaults_config();
        self.config.defaults.out = out;
        self.settings_options = ConversionOptions::from_config(&self.config);
        self.desktop_settings = self.preferences.application().desktop.clone();
        let defaults = self.settings_options.clone();
        if let Some(workspace) = &mut self.workspace {
            match workspace.transaction(|state| {
                for draft in &mut state.drafts {
                    if draft.submitted_task.is_none() {
                        draft.inherit(&defaults);
                    }
                }
                Ok(())
            }) {
                Ok(()) => {
                    if let Some(draft) = workspace.state.draft() {
                        self.task_options = draft.options.clone();
                    }
                }
                Err(error) => {
                    self.workspace_error =
                        Some(format!("默认设置已保存，准备中笔记尚未同步：{error:#}"))
                }
            }
        }
        cx.notify();
    }
    fn apply_language_preferences(&mut self, cx: &mut Context<Self>) {
        let raw = self.setting_value(EditField::Languages, cx);
        let languages: Vec<String> = raw
            .split([',', '，', '\n'])
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(str::to_owned)
            .collect();
        if languages
            .iter()
            .any(|v| v.len() > 35 || !v.chars().all(|c| c.is_ascii_alphanumeric() || c == '-'))
        {
            self.set_settings_feedback(
                PreferenceGroup::Generation,
                "语言代码只能包含字母、数字和连字符，请用逗号分隔各语言".into(),
                true,
            );
            cx.notify();
            return;
        }
        let mut next = self.generation_edit_base();
        next.preferred_subtitle_languages = languages;
        next.subtitle_languages_draft = None;
        self.commit_generation(next, cx);
    }
    /// The workbench box's language capsule commits through the same validated
    /// path as the settings form.
    pub(crate) fn choose_preferred_subtitle_languages(
        &mut self,
        languages: Vec<String>,
        cx: &mut Context<Self>,
    ) {
        let mut next = self.generation_edit_base();
        next.preferred_subtitle_languages = languages;
        next.subtitle_languages_draft = None;
        self.commit_generation(next, cx);
    }
    fn set_settings_feedback(&mut self, group: PreferenceGroup, message: String, error: bool) {
        if group == PreferenceGroup::Generation && error {
            self.settings_ui
                .generation_block_messages
                .insert(message.clone());
        }
        self.settings_ui.feedback_details.remove(&group);
        if !error {
            self.settings_ui.expanded_feedback.remove(&group);
        }
        self.settings_ui
            .feedback
            .insert(group, (message, error, Instant::now()));
    }
    fn set_settings_save_failure(&mut self, group: PreferenceGroup, error: &anyhow::Error) {
        self.set_settings_feedback(group, preferences::save_failure_message(group, error), true);
        self.settings_ui.feedback_details.insert(
            group,
            format!("设置目录：{}\n{error:#}", self.preferences.root().display()),
        );
    }
    fn settings_group_notice(&self, group: PreferenceGroup) -> Option<(String, bool)> {
        if let Some((message, error, when)) = self.settings_ui.feedback.get(&group)
            && (*error || when.elapsed() < Duration::from_secs(6))
        {
            return Some((message.clone(), *error));
        }
        self.preferences
            .issues()
            .iter()
            .rev()
            .find(|issue| issue.group == group)
            .map(|issue| {
                (
                    issue.message.clone(),
                    issue.kind != preferences::SettingsIssueKind::Recovered,
                )
            })
    }
    fn group_feedback(&self, group: PreferenceGroup, cx: &mut Context<Self>) -> Div {
        let mut view = v_flex().gap_2();
        if let Some((message, error)) = self.settings_group_notice(group) {
            view = view.child(
                text(("settings-group-feedback", group as usize), message.clone())
                    .text_sm()
                    .text_color(rgb(if error { 0xa32626 } else { MUTED })),
            );
            let has_pending = match group {
                PreferenceGroup::Generation => self.settings_ui.pending_generation.is_some(),
                PreferenceGroup::Application => self.settings_ui.pending_application.is_some(),
                PreferenceGroup::Services => false,
            };
            if error && has_pending {
                view = view.child(
                    control(("retry-settings-save", group as usize))
                        .label("重试保存")
                        .self_start()
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.retry_ordinary_preferences(group, cx);
                        })),
                );
            }
            let detail = self
                .settings_ui
                .feedback_details
                .get(&group)
                .cloned()
                .or_else(|| {
                    self.preferences
                        .issues()
                        .iter()
                        .rev()
                        .find(|issue| issue.group == group)
                        .and_then(|issue| issue.detail.clone())
                });
            if let Some(detail) = detail {
                let expanded = self.settings_ui.expanded_feedback.contains(&group);
                view = view.child(
                    control(("settings-feedback-details", group as usize))
                        .ghost()
                        .self_start()
                        .label(if expanded {
                            "收起技术详情"
                        } else {
                            "查看技术详情"
                        })
                        .on_click(cx.listener(move |this, _, _, cx| {
                            if !this.settings_ui.expanded_feedback.remove(&group) {
                                this.settings_ui.expanded_feedback.insert(group);
                            }
                            cx.notify();
                        })),
                );
                if expanded {
                    view = view.child(
                        text(("settings-feedback-technical", group as usize), detail)
                            .text_sm()
                            .text_color(rgb(MUTED)),
                    );
                }
            }
        }
        if self.preferences.is_blocked(group) {
            view = view.child(
                control(("reset-settings-group", group as usize))
                    .label("保留原文件并重置此组")
                    .self_start()
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.restore_settings_group(group, cx);
                    })),
            );
        }
        view
    }
    fn storage_settings_page(&self, cx: &mut Context<Self>) -> AnyElement {
        let mut view=v_flex().gap_5().child(text("storage-policy","所有已登记位置的笔记都会保留在课程库中。更改默认位置只影响新建笔记，准备中的笔记和已提交任务保持原位置。").text_sm().text_color(rgb(MUTED)));
        view = view.child(self.storage_status_panel(cx));
        if let Some(workspace) = &self.workspace {
            for (index, library) in workspace.state.libraries.iter().enumerate() {
                let root = library.root.clone();
                let id = library.id.clone();
                let move_id = id.clone();
                let default = workspace.state.default_library == id;
                let offline = !root.is_dir();
                view=view.child(v_flex().gap_2().py_3().border_t_1().border_color(rgb(LINE))
                .child(text(("storage-location-name",index),format!("{}{}{}",library.name,if default{" · 新笔记默认位置"}else{""},if offline{" · 位置暂时不可访问"}else{""})).font_weight(FontWeight::MEDIUM))
                .child(text(("storage-location-path",index),root.display().to_string()).text_sm().text_color(rgb(MUTED)))
                .child(h_flex().gap_2().flex_wrap()
                    .child(control(("open-storage-location",index)).label("打开位置").disabled(offline).on_click(move|_,_,cx|cx.open_with_system(&root)))
                    .when(!default,|row|row.child(control(("default-storage-location",index)).label("设为新笔记默认位置").on_click(cx.listener(move|this,_,_,cx|{
                        if let Some(workspace)=&mut this.workspace{
                            let result=workspace.state.library(&id).ok_or_else(||anyhow!("此位置已不在课程库中")).and_then(|library|tempfile::NamedTempFile::new_in(&library.root).map(drop).context("此位置暂时不能写入，请重新连接磁盘或恢复文件夹访问权限"))
                                .and_then(|_|workspace.transaction(|state|{state.default_library=id.clone();Ok(())}));
                            if let Err(error)=result{this.workspace_error=Some(format!("默认位置尚未更改：{error:#}"));}else{this.message=Some("新建笔记将使用此位置，已有笔记和任务保持原位置".into());}
                        }cx.notify();
                    }))))
                    .child(control(("move-storage-location",index)).label("移动课程库…").on_click(cx.listener(move|this,_,window,cx|this.begin_library_move(move_id.clone(),window,cx))))));
            }
        }
        view.child(control("register-storage-location").label("添加存储位置").self_start().on_click(cx.listener(|this,_,window,cx|this.register_storage_location(window,cx))))
            .child(text("storage-managed-files","模型和恢复所需的临时素材由软件管理。普通生成不要求手动清缓存，也不会自动删除原视频。").text_sm().text_color(rgb(MUTED)))
            .into_any_element()
    }
    fn register_storage_location(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let prompt = cx.prompt_for_paths(PathPromptOptions {
            files: false,
            directories: true,
            multiple: false,
            prompt: Some("添加存储位置".into()),
        });
        cx.spawn_in(window, async move |this, cx| {
            let result = prompt.await;
            let _ = this.update(cx, |this, cx| {
                match result {
                    Ok(Ok(Some(paths))) => {
                        if let Some(path) = paths.into_iter().next() {
                            let name = path
                                .file_name()
                                .and_then(|v| v.to_str())
                                .unwrap_or("课程库")
                                .to_owned();
                            let duplicate = this.workspace.as_ref().is_some_and(|w| {
                                w.state
                                    .libraries
                                    .iter()
                                    .any(|l| l.root.canonicalize().ok() == path.canonicalize().ok())
                            });
                            match this
                                .workspace
                                .as_mut()
                                .ok_or_else(|| anyhow!("课程库记录暂时不可用"))
                                .and_then(|w| w.register_library(path, name, false))
                            {
                                Ok(_) => {
                                    this.message = Some(
                                        if duplicate {
                                            "这个存储位置已经在课程库中"
                                        } else {
                                            "存储位置已添加，原有笔记和默认位置保持不变"
                                        }
                                        .into(),
                                    );
                                    this.refresh_library(cx);
                                }
                                Err(error) => {
                                    this.workspace_error =
                                        Some(format!("存储位置尚未添加：{error:#}"))
                                }
                            }
                        }
                    }
                    Ok(Ok(None)) => {}
                    Ok(Err(error)) => {
                        this.workspace_error = Some(format!("无法选择存储位置：{error:#}"))
                    }
                    Err(error) => {
                        this.workspace_error = Some(format!("无法打开文件选择器：{error}"))
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }
    fn application_settings_page(&self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        v_flex()
            .gap_5()
            .child(self.group_feedback(PreferenceGroup::Application, cx))
            .child(
                group("application-text-size", "文字大小").child(
                    SingleChoiceGroup::new("app-font-scale", "应用文字大小")
                        .options([1.0_f32, 1.25, 1.5, 2.0].into_iter().map(|scale| {
                            (
                                (scale * 100.).round().to_string(),
                                format!("{}%", (scale * 100.) as u32),
                            )
                        }))
                        .selected(
                            (self.preferences.application().font_scale * 100.)
                                .round()
                                .to_string(),
                        )
                        .on_change(cx.listener(
                            move |this, selected: &SharedString, window, cx| {
                                let Ok(percent) = selected.parse::<f32>() else {
                                    return;
                                };
                                let scale = percent / 100.;
                                let mut next = this.application_edit_base();
                                next.font_scale = scale;
                                if this.commit_application(next, cx) {
                                    theme::apply_scale(scale, window, cx);
                                }
                            },
                        )),
                ),
            )
            .child(preference(
                "减少动态效果",
                "保留完整内容和操作反馈，减少界面移动。",
                Switch::new("app-reduce-motion")
                    .checked(self.preferences.application().desktop.reduce_motion)
                    .on_click(cx.listener(|this, enabled, _, cx| {
                        let mut next = this.application_edit_base();
                        next.desktop.reduce_motion = *enabled;
                        this.commit_application(next, cx);
                    })),
            ))
            .child(self.legacy_migration_panel(cx))
            .child(self.environment_page(window, cx))
            .child(self.about_page(cx))
            .into_any_element()
    }
    fn commit_application(&mut self, next: ApplicationPreferences, cx: &mut Context<Self>) -> bool {
        match self.preferences.save_application(next.clone()) {
            Ok(()) => {
                self.settings_ui.pending_application = None;
                self.desktop_settings = self.preferences.application().desktop.clone();
                cx.set_reduce_motion(self.desktop_settings.reduce_motion);
                self.set_settings_feedback(PreferenceGroup::Application, "已保存".into(), false);
                cx.notify();
                true
            }
            Err(error) => {
                self.settings_ui.pending_application = Some(next);
                self.set_settings_save_failure(PreferenceGroup::Application, &error);
                cx.notify();
                false
            }
        }
    }
    fn legacy_migration_panel(&self, cx: &mut Context<Self>) -> Div {
        let mut view = v_flex().gap_2();
        if !self.preferences.legacy_imported()
            && self.settings_ui.legacy_available
            && course2md::settings::config_path().is_file()
        {
            view=view.child(text("legacy-settings-found","发现旧版配置。可以一次性导入识别方式、AI 规则和已配置服务，之后以这里的设置为准。").text_sm())
                .child(control("import-legacy-settings").label("导入旧版设置").self_start().on_click(cx.listener(|this,_,window,cx|{
                    match course2md::settings::load().and_then(|config|this.preferences.import_legacy(&config)){
                        Ok(())=>{this.settings_ui.legacy_status=Some("已导入旧版设置；原文件保留，今后不会自动读取其中的修改".into());this.config_error=false;this.settings_ui.initialized=false;this.hydrate_settings_inputs(window,cx);this.refresh_preference_defaults(cx);},
                        Err(error)=>this.settings_ui.legacy_status=Some(format!("旧版设置尚未完整导入：{error:#}。现有设置与原文件均保留"))
                    }cx.notify();
                })));
        }
        if let Some(message) = &self.settings_ui.legacy_status {
            view = view.child(text("legacy-import-status", message.clone()).text_sm());
        }
        view
    }
    fn environment_page(&self, _window: &mut Window, cx: &mut Context<Self>) -> Div {
        let mut view = group("diagnostics-heading", "诊断详情")
            .child(
                text(
                    "diagnostics-demand-policy",
                    "生成时只检查本次需要的能力。这里的检查不会开始下载、识别或发送课程内容。",
                )
                .text_sm()
                .text_color(rgb(MUTED)),
            )
            .child(
                control("refresh-environment")
                    .label("重新检查本机能力")
                    .loading(self.environment.is_none())
                    .disabled(self.environment.is_none())
                    .self_start()
                    .on_click(cx.listener(|this, _, _, cx| this.refresh_environment(cx))),
            );
        if let Some(e) = &self.environment {
            for (index, (name, purpose, found)) in [
                ("转换程序", "运行本地处理步骤", e.engine),
                ("ffmpeg", "读取媒体、声音与截图", e.ffmpeg),
                ("ffprobe", "确认媒体内容与时长", e.ffprobe),
                ("yt-dlp", "读取在线来源", e.ytdlp),
                ("llama-server", "GPU / CPU 识别运行时", e.llama),
            ]
            .into_iter()
            .enumerate()
            {
                view = view.child(
                    text(
                        ("diagnostic-capability", index),
                        format!(
                            "{name} · {purpose} · {}",
                            if found {
                                "已检测到"
                            } else {
                                "未检测到"
                            }
                        ),
                    )
                    .text_sm(),
                );
            }
            if !e.engine {
                view=view.child(text("repair-bundled-engine","应用内的转换程序无法运行。重新安装完整应用可恢复该组件，已有笔记保留。").text_sm()).child(control("download-repair-app").label("下载安装包").self_start().on_click(|_,_,cx|cx.open_url("https://github.com/mizorewww/course2md/releases")));
            }
            if !e.ffmpeg || !e.ffprobe || !e.ytdlp {
                let (help, command) = if cfg!(target_os = "macos") {
                    (
                        "在终端运行，需要先安装 Homebrew。安装成功后回到这里重新检查。",
                        "brew install ffmpeg yt-dlp",
                    )
                } else if cfg!(target_os = "windows") {
                    (
                        "在 PowerShell 运行，需要 Windows 应用安装程序提供 winget。安装成功后回到这里重新检查。",
                        "winget install Gyan.FFmpeg; winget install yt-dlp.yt-dlp",
                    )
                } else {
                    (
                        "在终端使用系统软件包管理器安装 ffmpeg；下载工具命令需要先安装 pipx。完成后回到这里重新检查。",
                        "pipx install yt-dlp",
                    )
                };
                view = view
                    .child(text("install-media-tools-help", help).text_sm())
                    .child(text("install-media-tools-command", command).text_sm())
                    .child(
                        control("copy-media-install-command")
                            .label("复制视频工具安装命令")
                            .self_start()
                            .on_click(move |_, _, cx| {
                                cx.write_to_clipboard(ClipboardItem::new_string(command.into()))
                            }),
                    );
            }
        } else {
            view = view.child(text("diagnostic-checking", "正在检查本机能力…").text_sm());
        }
        view.child(self.model_diagnostics_panel(cx))
    }
    // Wrappers only while the other interface modules are being integrated.
    /// Service editor drafts are deliberately excluded: an existing published service
    /// continues to be usable. An unapplied failed ordinary commit must never be silently
    /// replaced by the older effective defaults when submitting a task.
    pub fn ordinary_preferences_ready_for_submit(&self) -> Result<()> {
        if let Some(issue) = self.ordinary_preferences_submit_issue() {
            return Err(anyhow!(issue.message));
        }
        Ok(())
    }
    pub fn ordinary_preferences_submit_issue(&self) -> Option<OrdinaryPreferenceIssue> {
        let group = PreferenceGroup::Generation;
        if self.preferences.is_blocked(PreferenceGroup::Generation) {
            return Some(OrdinaryPreferenceIssue {
                group,
                message: "生成选项暂时无法读取，原文件与本次草稿已保留。".into(),
                can_retry: false,
            });
        }
        if self.settings_ui.pending_generation.is_some()
            || self.preferences.generation_intent().is_some()
        {
            return Some(OrdinaryPreferenceIssue {
                group,
                message: self
                    .settings_group_notice(group)
                    .filter(|(_, error)| *error)
                    .map(|(message, _)| message)
                    .unwrap_or_else(|| "生成选项的修改尚未保存，本次草稿已保留。".into()),
                can_retry: true,
            });
        }
        None
    }
    pub fn retry_ordinary_preferences(
        &mut self,
        group: PreferenceGroup,
        cx: &mut Context<Self>,
    ) -> bool {
        if self.preferences.is_blocked(group) {
            return false;
        }
        match group {
            PreferenceGroup::Generation => {
                if let Some(next) = self
                    .settings_ui
                    .pending_generation
                    .as_ref()
                    .or_else(|| self.preferences.generation_intent())
                    .cloned()
                {
                    self.commit_generation(next, cx)
                } else {
                    true
                }
            }
            PreferenceGroup::Application => {
                if let Some(next) = self
                    .settings_ui
                    .pending_application
                    .as_ref()
                    .or_else(|| self.preferences.application_intent())
                    .cloned()
                {
                    self.commit_application(next, cx)
                } else {
                    true
                }
            }
            PreferenceGroup::Services => false,
        }
    }
    /// Only the explicitly named group is reset; the form and its source stay in place.
    pub fn restore_ordinary_preferences(
        &mut self,
        group: PreferenceGroup,
        cx: &mut Context<Self>,
    ) -> bool {
        if group == PreferenceGroup::Services {
            return false;
        }
        self.restore_settings_group(group, cx)
    }
    fn restore_settings_group(&mut self, group: PreferenceGroup, cx: &mut Context<Self>) -> bool {
        let previous = self
            .ordinary_preferences_submit_issue()
            .map(|issue| issue.message);
        match self.preferences.reset_group(group) {
            Ok(()) => {
                match group {
                    PreferenceGroup::Generation => self.settings_ui.pending_generation = None,
                    PreferenceGroup::Application => self.settings_ui.pending_application = None,
                    PreferenceGroup::Services => {}
                }
                self.settings_ui.initialized = false;
                self.set_settings_feedback(group, "原文件已保留，此组设置已重置".into(), false);
                if group == PreferenceGroup::Application {
                    self.desktop_settings = self.preferences.application().desktop.clone();
                    cx.set_reduce_motion(self.desktop_settings.reduce_motion);
                } else {
                    self.refresh_preference_defaults(cx);
                }
                if group == PreferenceGroup::Generation {
                    self.clear_resolved_generation_errors(previous);
                }
                cx.notify();
                true
            }
            Err(error) => {
                self.set_settings_save_failure(group, &error);
                cx.notify();
                false
            }
        }
    }
    fn clear_resolved_generation_errors(&mut self, previous: Option<String>) {
        if self.ordinary_preferences_submit_issue().is_some() {
            return;
        }
        let mut messages = std::mem::take(&mut self.settings_ui.generation_block_messages);
        if let Some(previous) = previous {
            messages.insert(previous);
        }
        for error in [&mut self.source_validation, &mut self.message] {
            if error.as_ref().is_some_and(|error| messages.contains(error)) {
                *error = None;
            }
        }
    }
    pub fn relocate_settings_paths(
        &mut self,
        old: &std::path::Path,
        new: &std::path::Path,
        cx: &mut Context<Self>,
    ) -> Result<()> {
        let previous = self
            .ordinary_preferences_submit_issue()
            .map(|issue| issue.message);
        let result = self.preferences.relocate_generation_paths(old, new);
        self.settings_ui.pending_generation = self.preferences.generation_intent().cloned();
        if let Err(error) = &result {
            self.set_settings_feedback(
                PreferenceGroup::Generation,
                format!("模型位置尚未更新，旧位置备份已保留：{error:#}"),
                true,
            );
        } else {
            self.refresh_preference_defaults(cx);
            self.clear_resolved_generation_errors(previous);
        }
        cx.notify();
        result
    }
    /// Called before app quit. A recoverable ordinary edit need not block exit; a new key
    /// or service edit that cannot be saved anywhere must stay visible for the user.
    pub fn flush_settings_for_exit(&mut self, cx: &mut Context<Self>) -> bool {
        if let Some(editor) = &self.settings_ui.editor
            && editor.pending_binding.is_none()
        {
            let mut draft = editor.draft.clone();
            draft.name = self.setting_value(EditField::Name, cx);
            draft.address = self.setting_value(EditField::Address, cx);
            draft.model = self.setting_value(EditField::Model, cx);
            let key = self.setting_value(EditField::Key, cx);
            if !key.trim().is_empty() || self.preferences.draft(&draft.id) != Some(&draft) {
                match self
                    .preferences
                    .save_service_draft(draft, (!key.trim().is_empty()).then(|| Secret::new(key)))
                {
                    Ok(saved) => {
                        if let Some(editor) = &mut self.settings_ui.editor {
                            editor.draft = saved;
                            editor.save_failed = false;
                        }
                    }
                    Err(error) => {
                        if let Some(editor) = &mut self.settings_ui.editor {
                            editor.status =
                                Some(format!("这些修改尚未保存，窗口已保留：{error:#}"));
                            editor.save_failed = true;
                        }
                        cx.notify();
                        return false;
                    }
                }
            }
        }
        if let Some(next) = self.settings_ui.pending_generation.clone() {
            self.commit_generation(next, cx);
        }
        if let Some(next) = self.settings_ui.pending_application.clone() {
            self.commit_application(next, cx);
        }
        if !self.preferences.unsaved_intents_are_preserved() {
            self.message = Some("设置修改暂时无法保存到恢复记录，窗口已保留。请重试保存，或保留当前输入后恢复此组设置。".into());
            cx.notify();
            return false;
        }
        true
    }
    pub fn edited_settings(&self, _cx: &App) -> course2md::settings::ConfigFile {
        self.preferences.defaults_config()
    }
    pub fn invalid_setting(&self, _cx: &App) -> Option<(Field, &'static str)> {
        None
    }
    pub fn save_settings(&mut self, cx: &mut Context<Self>) {
        let mut next = self.generation_edit_base();
        let mut config = self.preferences.defaults_config();
        self.settings_options.apply_to(&mut config);
        next.options = config.defaults;
        next.ai_proofread = config.llm.enabled;
        next.ai_summary = config.llm.summarize;
        next.vision = config.llm.vision;
        self.commit_generation(next, cx);
    }
}
