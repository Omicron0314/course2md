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
    menu::{DropdownMenu, PopupMenuItem},
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
    test_details_open: bool,
    save_failed: bool,
}

pub struct OrdinaryPreferenceIssue {
    pub group: PreferenceGroup,
    pub message: String,
    pub can_retry: bool,
}

pub(crate) struct State {
    model_diagnostics: model_diagnostics::State,
    tab_focus: [FocusHandle; 5],
    inputs: BTreeMap<EditField, Entity<InputState>>,
    prompt: Entity<TextareaState>,
    editor: Option<ServiceEditor>,
    initialized: bool,
    show_prompt: bool,
    language_details_open: bool,
    asr_details_open: bool,
    model_details_open: bool,
    storage_details_open: bool,
    diagnostics_details_open: bool,
    feedback: BTreeMap<PreferenceGroup, (String, bool, Instant)>,
    feedback_details: BTreeMap<PreferenceGroup, String>,
    expanded_feedback: std::collections::BTreeSet<PreferenceGroup>,
    generation_block_messages: std::collections::BTreeSet<String>,
    pending_generation: Option<GenerationPreferences>,
    pending_application: Option<ApplicationPreferences>,
    legacy_status: Option<String>,
    legacy: crate::legacy_settings::Inspection,
    legacy_details_open: bool,
    legacy_action_detail: Option<String>,
    legacy_preserved: Option<PathBuf>,
    _subscriptions: Vec<Subscription>,
}

impl State {
    pub fn new(window: &mut Window, cx: &mut Context<Desktop>) -> Self {
        cx.bind_keys([
            KeyBinding::new("right", NextSettingsTab, Some("SettingsTabs")),
            KeyBinding::new("down", NextSettingsTab, Some("SettingsTabs")),
            KeyBinding::new("left", PreviousSettingsTab, Some("SettingsTabs")),
            KeyBinding::new("up", PreviousSettingsTab, Some("SettingsTabs")),
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
            let placeholder = match field {
                EditField::Address => "https://api.example.com/v1",
                EditField::Model => "服务商提供的模型 ID",
                _ => "",
            };
            (
                field,
                cx.new(|cx| {
                    InputState::new(window, cx)
                        .placeholder(placeholder)
                        .masked(field == EditField::Key)
                }),
            )
        })
        .collect();
        let mut subscriptions = Vec::new();
        for (field, input) in &inputs {
            let field = *field;
            subscriptions.push(
                cx.subscribe_in(input, window, move |this, _, event, _, cx| {
                    if matches!(event, InputEvent::Change) {
                        this.setting_input_changed(field, cx);
                    }
                }),
            );
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
            language_details_open: false,
            asr_details_open: false,
            model_details_open: false,
            storage_details_open: false,
            diagnostics_details_open: false,
            feedback: BTreeMap::new(),
            feedback_details: BTreeMap::new(),
            expanded_feedback: Default::default(),
            generation_block_messages: Default::default(),
            pending_generation: None,
            pending_application: None,
            legacy_status: None,
            legacy: crate::legacy_settings::Inspection::inspect(course2md::settings::config_path()),
            legacy_details_open: false,
            legacy_action_detail: None,
            legacy_preserved: None,
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
        self.desktop.update(cx, |desktop, cx| {
            desktop.service_editor_content(false, window, cx)
        })
    }
}

fn text(id: impl Into<ElementId>, value: impl Into<SharedString>) -> Stateful<Div> {
    theme::accessible_text(id, value)
}
pub(super) fn field_label(
    id: impl Into<ElementId>,
    value: impl Into<SharedString>,
) -> Stateful<Div> {
    text(id, value)
        .text_size(TEXT_BODY)
        .font_weight(FontWeight::MEDIUM)
}

/// A field and its control share a row; the control moves below the label when
/// the pane cannot accommodate both columns. Widths follow the app's text scale.
pub(super) fn settings_row(
    id: impl Into<ElementId>,
    label: &'static str,
    hint: &'static str,
    control: impl IntoElement,
) -> Div {
    settings_form_row(id, label, hint, control, false)
}

/// Keep the label aligned with the input when its column also contains field
/// feedback or an action, using the same form columns as ordinary preferences.
pub(super) fn settings_field_row(
    id: impl Into<ElementId>,
    label: &'static str,
    hint: &'static str,
    field: impl IntoElement,
) -> Div {
    settings_form_row(id, label, hint, field, true)
}

fn settings_form_row(
    id: impl Into<ElementId>,
    label: &'static str,
    hint: &'static str,
    control: impl IntoElement,
    align_first_control: bool,
) -> Div {
    let id = id.into();
    h_flex()
        .w_full()
        .min_w_0()
        .min_h(CONTROL_HEIGHT)
        .items_center()
        .when(align_first_control, |row| row.items_start())
        .flex_wrap()
        .gap_4()
        .child(
            v_flex()
                .flex_1()
                .flex_basis(rems(160. / 14.))
                .min_w(rems(140. / 14.))
                .when(align_first_control, |label| {
                    label.min_h(CONTROL_HEIGHT).justify_center()
                })
                .gap_1()
                .child(field_label(id.clone(), label))
                .when(!hint.is_empty(), |view| {
                    view.child(
                        text(SharedString::from(format!("{id:?}-hint")), hint)
                            .min_w_0()
                            .whitespace_normal()
                            .text_size(TEXT_AUX)
                            .text_color(color(MUTED)),
                    )
                }),
        )
        .child(
            h_flex()
                .w(rems(360. / 14.))
                .max_w_full()
                .min_w_0()
                .when(align_first_control, |field| field.min_h(CONTROL_HEIGHT))
                .flex_shrink_0()
                .justify_start()
                .items_center()
                .child(control),
        )
}

/// Section boundaries use hierarchy and space, without a second card outline.
pub(super) fn settings_section(id: impl Into<ElementId>, title: &'static str, icon: Icon) -> Div {
    v_flex().w_full().min_w_0().gap_4().child(
        h_flex()
            .gap_2()
            .items_center()
            .child(icon.size_5().flex_shrink_0().text_color(color(MUTED)))
            .child(
                text(id, title)
                    .role(Role::Heading)
                    .text_size(TEXT_TITLE)
                    .font_weight(FontWeight::SEMIBOLD),
            ),
    )
}

pub(super) fn settings_detail_group(id: impl Into<ElementId>, title: &'static str) -> Div {
    v_flex().w_full().min_w_0().gap_3().child(
        field_label(id, title)
            .role(Role::Heading)
            .font_weight(FontWeight::SEMIBOLD),
    )
}

/// Diagnostic values begin on a common reading axis. They can wrap and grow
/// vertically without moving short values to the far edge of the window.
pub(super) fn settings_detail_row(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    value: impl IntoElement,
) -> Div {
    h_flex()
        .w_full()
        .min_w_0()
        .min_h(rems(28. / 14.))
        .items_start()
        .flex_wrap()
        .gap_x_4()
        .gap_y_1()
        .child(
            field_label(id, label)
                .w(rems(160. / 14.))
                .max_w_full()
                .flex_shrink_0()
                .whitespace_normal(),
        )
        .child(
            v_flex()
                .flex_1()
                .flex_basis(rems(240. / 14.))
                .min_w_0()
                .items_start()
                .child(value),
        )
}

pub(super) fn settings_value(
    id: impl Into<ElementId>,
    value: impl Into<SharedString>,
) -> Stateful<Div> {
    text(id, value)
        .w_full()
        .min_w_0()
        .whitespace_normal()
        .text_size(TEXT_BODY)
        .text_color(color(INK))
}
fn service_protocol_label(protocol: ServiceProtocol) -> &'static str {
    match protocol {
        ServiceProtocol::SpeechTranscriptions => "语音转录",
        ServiceProtocol::SpeechChat => "音频对话",
        ServiceProtocol::AiChat => "AI 对话",
    }
}

fn settings_tab_icon(index: usize) -> Icon {
    match index {
        4 => icons::palette(),
        0 => icons::tune(),
        1 => icons::cloud(),
        2 => icons::storage(),
        _ => icons::info(),
    }
}
const SETTINGS_TABS: [(usize, &str); 5] = [
    (4, "外观"),
    (0, "生成笔记"),
    (1, "服务与账号"),
    (2, "存储"),
    (3, "应用"),
];

fn settings_tab_label(index: usize) -> &'static str {
    ["生成笔记", "服务与账号", "存储", "应用", "外观"][index.min(4)]
}

fn settings_tab_position(index: usize) -> usize {
    SETTINGS_TABS
        .iter()
        .position(|(tab, _)| *tab == index)
        .unwrap_or(0)
}

fn group(id: &'static str, title: &'static str) -> Div {
    let icon = match id {
        "language-settings" => icons::subtitles(),
        "asr-default-settings" | "speech-services-heading" => icons::microphone(),
        "ai-default-settings" | "ai-services-heading" => icons::science(),
        "export-default-settings" => icons::download(),
        "account-settings-heading" => icons::login(),
        "appearance-motion" => icons::tune(),
        "diagnostics-heading" => icons::settings(),
        _ => icons::info(),
    };
    settings_section(id, title, icon)
}
pub(super) fn preference(label: &'static str, hint: &'static str, control: Switch) -> Div {
    settings_row(
        SharedString::from(format!("preference-label-{label}")),
        label,
        hint,
        crate::focus_scroll::FocusRing::new(
            SharedString::from(format!("preference-focus-{label}")),
            coral_switch(control).accessibility_label(label).p_2(),
        ),
    )
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
    pub(crate) fn application_edit_base(&self) -> ApplicationPreferences {
        self.settings_ui
            .pending_application
            .as_ref()
            .or_else(|| self.preferences.application_intent())
            .unwrap_or_else(|| self.preferences.application())
            .clone()
    }
    fn setting_value(&self, field: EditField, cx: &App) -> String {
        self.settings_ui.inputs[&field].read(cx).value().to_string()
    }
    fn reveal_setting(&self, id: impl Into<ElementId>, child: impl IntoElement) -> Div {
        let child = child.into_any_element();
        if self.page == Page::Settings {
            div()
                .w_full()
                .min_w_0()
                .child(crate::focus_scroll::RevealFocus::new(
                    id,
                    child,
                    self.scrolls[Page::Settings as usize].clone(),
                ))
        } else {
            div().w_full().min_w_0().child(child)
        }
    }
    fn setting_choices(
        &self,
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
    ) -> SingleChoiceGroup {
        SingleChoiceGroup::new(id, label).when(self.page == Page::Settings, |group| {
            group.reveal_in(self.scrolls[Page::Settings as usize].clone())
        })
    }
    fn setting_preference(&self, label: &'static str, hint: &'static str, control: Switch) -> Div {
        self.reveal_setting(
            SharedString::from(format!("preference-reveal-{label}")),
            preference(label, hint, control),
        )
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
        self.reveal_setting(
            ("setting-field-reveal", field as usize),
            v_flex()
                .w_full()
                .gap_2()
                .child(field_label(("setting-field-label", field as usize), label))
                .child(
                    text_input(&self.settings_ui.inputs[&field])
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
                        .when(error.is_some(), |input| input.border_color(color(DANGER))),
                )
                .when_some(error, |view, error| {
                    view.child(
                        text(("setting-error", field as usize), error)
                            .text_sm()
                            .text_color(color(DANGER)),
                    )
                })
                .when(field == EditField::Address, |view| {
                    view.child(
                        text(
                            "service-address-hint",
                            "可填写基础地址或完整接口地址，需包含 http:// 或 https://。",
                        )
                        .text_size(TEXT_AUX)
                        .text_color(color(MUTED)),
                    )
                }),
        )
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

    pub(crate) fn open_settings(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.page != Page::Settings {
            self.settings_origin = Some(self.page);
            self.settings_return_focus = window.focused(cx);
        }
        self.navigate(Page::Settings, cx);
        self.settings_ui.tab_focus[self.settings_tab.min(4)].focus(window, cx);
    }

    fn select_settings_tab(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        self.settings_tab = index;
        self.scrolls[Page::Settings as usize].set_offset(point(px(0.), px(0.)));
        self.settings_ui.tab_focus[index].focus(window, cx);
        cx.notify();
    }

    fn settings_navigation(
        &self,
        sidebar: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let scale = f32::from(window.rem_size()) / 14.;
        let short = !sidebar && crate::views::settings_content_width(window) < 640. * scale;
        let group = SingleChoiceGroup::new("settings-tabs", "设置分类")
            .tabs()
            .full_width()
            .options(SETTINGS_TABS.into_iter().map(|(index, label)| {
                (
                    index.to_string(),
                    if short {
                        match index {
                            0 => "生成",
                            1 => "服务",
                            _ => label,
                        }
                    } else {
                        label
                    }
                    .to_owned(),
                )
            }))
            .selected(self.settings_tab.to_string())
            .focus_handles(
                SETTINGS_TABS
                    .into_iter()
                    .map(|(index, _)| self.settings_ui.tab_focus[index].clone()),
            )
            .icon("4", settings_tab_icon(4))
            .icon("0", settings_tab_icon(0))
            .icon("1", settings_tab_icon(1))
            .icon("2", settings_tab_icon(2))
            .icon("3", settings_tab_icon(3))
            .on_change(cx.listener(|this, value: &SharedString, window, cx| {
                if let Ok(index) = value.parse() {
                    this.select_settings_tab(index, window, cx);
                }
            }));
        if sidebar {
            div()
                .w(px(crate::views::settings_sidebar_width(window)))
                .flex_shrink_0()
                .child(group.vertical())
                .into_any_element()
        } else {
            div()
                .w_full()
                .flex_shrink_0()
                .child(group)
                .into_any_element()
        }
    }

    pub fn settings_page(&mut self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        self.hydrate_settings_inputs(window, cx);
        self.ensure_settings_model_diagnostic(cx);
        if self.settings_tab == 5 {
            self.settings_tab = 1;
        }
        if self.settings_tab > 4 {
            self.settings_tab = 4;
        }
        for (index, focus) in self.settings_ui.tab_focus.iter_mut().enumerate() {
            *focus = focus.clone().tab_stop(index == self.settings_tab);
        }
        let sidebar = crate::views::settings_uses_sidebar(window);
        let content_width = crate::views::settings_content_width(window);
        let layout_width = content_width
            + if sidebar {
                crate::views::settings_sidebar_width(window) + crate::views::SETTINGS_COLUMN_GAP
            } else {
                0.
            };
        let header = text("settings-page-title", "设置")
            .role(Role::Heading)
            .text_size(TEXT_DISPLAY)
            .font_weight(FontWeight::SEMIBOLD)
            .flex_shrink_0();
        let header = h_flex()
            .w_full()
            .min_w_0()
            .items_center()
            .justify_between()
            .gap_4()
            .child(header)
            .when(
                self.settings_origin == Some(Page::Result) && self.preview.is_some(),
                |row| {
                    row.child(
                        quiet("return-to-reading")
                            .icon(icons::arrow_left())
                            .label("返回笔记")
                            .on_click(cx.listener(|this, _, window, cx| {
                                let focus = this.settings_return_focus.take();
                                this.settings_origin = None;
                                this.navigate(Page::Result, cx);
                                if let Some(focus) = focus {
                                    window.on_next_frame(move |window, cx| focus.focus(window, cx));
                                }
                            })),
                    )
                },
            );
        let navigation = self.settings_navigation(sidebar, window, cx);
        let mut panel = v_flex()
            .id("settings-panel")
            .role(Role::TabPanel)
            .aria_label(settings_tab_label(self.settings_tab))
            .flex_1()
            .min_w_0()
            .min_h_0()
            .w(px(content_width))
            .gap(px(24.));
        for group in [
            PreferenceGroup::Generation,
            PreferenceGroup::Services,
            PreferenceGroup::Application,
        ] {
            let current_group = match self.settings_tab {
                0 => Some(PreferenceGroup::Generation),
                1 => Some(PreferenceGroup::Services),
                3 | 4 => Some(PreferenceGroup::Application),
                _ => None,
            };
            if current_group == Some(group) {
                if self.preferences.is_blocked(group)
                    || self
                        .settings_group_notice(group)
                        .is_some_and(|(_, error)| error)
                {
                    panel = panel.child(
                        div()
                            .id(("fixed-settings-feedback", group as usize))
                            .w_full()
                            .min_w_0()
                            .flex_shrink_0()
                            .max_h(px(180.))
                            .overflow_y_scroll()
                            .p(px(12.))
                            .rounded(RADIUS_SMALL)
                            .bg(color(DANGER_BG))
                            .child(self.group_feedback(group, cx)),
                    );
                }
                continue;
            }
            let target = match group {
                PreferenceGroup::Generation => 0,
                PreferenceGroup::Services => 1,
                PreferenceGroup::Application => 4,
            };
            if target != self.settings_tab
                && let Some((message, true)) = self.settings_group_notice(group)
            {
                panel = panel.child(
                    h_flex()
                        .w_full()
                        .min_w_0()
                        .flex_shrink_0()
                        .gap_2()
                        .flex_wrap()
                        .p_3()
                        .rounded(RADIUS_SMALL)
                        .bg(color(DANGER_BG))
                        .child(
                            text(
                                ("settings-failed-group", target),
                                if message.starts_with(group.label()) {
                                    message
                                } else {
                                    format!("{}：{message}", group.label())
                                },
                            )
                            .role(Role::Alert)
                            .flex_1()
                            .min_w_0()
                            .text_color(color(DANGER))
                            .text_sm(),
                        )
                        .child(
                            control(("open-failed-settings-group", target))
                                .icon(icons::settings())
                                .label("查看设置")
                                .on_click(cx.listener(move |this, _, window, cx| {
                                    this.select_settings_tab(target, window, cx);
                                })),
                        ),
                );
            }
        }
        let content = match self.settings_tab {
            0 => self.generation_settings_page(window, cx),
            1 => self.services_settings_page(window, cx),
            2 => self.storage_settings_page(cx),
            3 => self.application_settings_page(window, cx),
            _ => self.appearance_page(window, cx),
        };
        let panel = panel.child(
            div()
                .id("settings-content-scroll")
                .flex_1()
                .min_w_0()
                .min_h_0()
                .w_full()
                .overflow_y_scroll()
                .track_scroll(&self.scrolls[Page::Settings as usize])
                .child(div().w_full().min_w_0().pb(px(24.)).child(content)),
        );
        let body = div()
            .flex()
            .w_full()
            .min_w_0()
            .min_h_0()
            .flex_1()
            .when(sidebar, |view| {
                view.flex_row()
                    .items_stretch()
                    .gap(px(crate::views::SETTINGS_COLUMN_GAP))
            })
            .when(!sidebar, |view| view.flex_col().gap(px(24.)))
            .child(navigation)
            .child(panel);
        v_flex()
            .w_full()
            .h_full()
            .min_w_0()
            .min_h_0()
            .max_w(px(layout_width))
            .mx_auto()
            .pt(px(24.))
            .gap(px(24.))
            .child(header)
            .child(body)
            .into_any_element()
    }

    fn generation_settings_page(&self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let value = self.preferences.generation().clone();
        let provider = value.options.provider.map(|p| p.as_str()).unwrap_or("");
        let language = match value.preferred_subtitle_languages.as_slice() {
            [] => "",
            [one] if ["zh-Hans", "zh-Hant", "en", "ja"].contains(&one.as_str()) => one.as_str(),
            _ => "custom",
        };
        let languages = group("language-settings", "文字来源")
            .child(
                text("subtitle-policy", "优先使用字幕，没有字幕时识别视频声音。")
                    .text_sm()
                    .text_color(color(MUTED)),
            )
            .child(settings_row(
                "subtitle-language-label",
                "字幕语言",
                "",
                self.setting_choices("default-subtitle-language", "字幕语言")
                    .options([
                        ("", "自动"),
                        ("zh-Hans", "简体"),
                        ("zh-Hant", "繁體"),
                        ("en", "英语"),
                        ("ja", "日语"),
                    ])
                    .full_width()
                    .selected(language.to_owned())
                    .on_change(cx.listener(|this, selected: &SharedString, window, cx| {
                        let mut next = this.generation_edit_base();
                        next.preferred_subtitle_languages = if selected.is_empty() {
                            Vec::new()
                        } else {
                            vec![selected.to_string()]
                        };
                        next.subtitle_languages_draft = None;
                        if this.commit_generation(next, cx) {
                            this.settings_ui.language_details_open = false;
                            this.settings_ui.inputs[&EditField::Languages]
                                .update(cx, |input, cx| {
                                    input.set_value(selected.clone(), window, cx)
                                });
                        }
                    })),
            ))
            .when(language == "custom", |view| {
                view.child(
                    text(
                        "subtitle-custom-summary",
                        format!(
                            "优先顺序：{}",
                            value.preferred_subtitle_languages.join(" → ")
                        ),
                    )
                    .text_sm()
                    .text_color(color(MUTED)),
                )
            })
            .child(
                quiet("toggle-language-details")
                    .icon(icons::tune())
                    .label("自定义语言优先级")
                    .self_start()
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.settings_ui.language_details_open =
                            !this.settings_ui.language_details_open;
                        cx.notify();
                    })),
            )
            .child(crate::motion::disclosure(
                "subtitle-language-details",
                self.settings_ui.language_details_open || value.subtitle_languages_draft.is_some(),
                v_flex()
                    .gap_3()
                    .child(
                        self.setting_field(EditField::Languages, "语言优先级", cx)
                            .max_w(rems(24.)),
                    )
                    .child(
                        text(
                            "subtitle-language-help",
                            "用逗号分隔语言代码，例如 zh-Hans, en。留空表示自动。",
                        )
                        .text_sm()
                        .text_color(color(MUTED)),
                    )
                    .child(
                        outline_pill("apply-languages")
                            .icon(icons::check_circle())
                            .label("应用语言偏好")
                            .self_start()
                            .on_click(
                                cx.listener(|this, _, _, cx| this.apply_language_preferences(cx)),
                            ),
                    ),
                window,
                cx,
            ));
        let mut recognition = group("asr-default-settings", "语音识别")
            .child(settings_row(
                "asr-method-label",
                "识别方式",
                "",
                self.setting_choices("default-asr-device", "默认语音识别方式")
                    .options([("", "自动"), ("local", "本机识别"), ("api", "在线语音服务")])
                    .full_width()
                    .selected(if provider.is_empty() {
                        ""
                    } else if provider == "api" {
                        "api"
                    } else {
                        "local"
                    })
                    .on_change(cx.listener(|this, selected: &SharedString, _, cx| {
                        let mut next = this.generation_edit_base();
                        next.options.provider = match selected.as_ref() {
                            "local" => Some(this.recommended_local_provider()),
                            "api" => Some(course2md::config::AsrProvider::Api),
                            _ => None,
                        };
                        this.commit_generation(next, cx);
                    })),
            ))
            .child(
                text(
                    "asr-default-help",
                    if provider == "api" {
                        "视频声音会发送到所选语音服务。"
                    } else {
                        "在这台电脑上识别，课程声音不会上传。"
                    },
                )
                .text_sm()
                .text_color(color(MUTED)),
            );
        if provider == "api" {
            recognition = recognition.child(self.service_picker(ServicePurpose::Speech, false, cx));
        } else {
            let (settled, conclusion) = self.default_model_conclusion();
            recognition = recognition
                .child(self.local_model_picker(cx))
                .child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .child(
                            if settled {
                                icons::info()
                            } else {
                                icons::warning()
                            }
                            .size_4()
                            .text_color(color(if settled {
                                MUTED
                            } else {
                                WARNING
                            })),
                        )
                        .child(
                            text("model-readiness-conclusion", conclusion)
                                .text_sm()
                                .text_color(color(if settled { MUTED } else { WARNING })),
                        ),
                )
                .child(
                    quiet("toggle-asr-advanced")
                        .icon(icons::tune())
                        .label("高级识别设置")
                        .self_start()
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.settings_ui.asr_details_open = !this.settings_ui.asr_details_open;
                            cx.notify();
                        })),
                )
                .child(crate::motion::disclosure(
                    "asr-advanced-settings",
                    self.settings_ui.asr_details_open,
                    v_flex()
                        .gap_3()
                        .child(field_label("asr-hardware-heading", "固定识别方式"))
                        .child(
                            self.setting_choices("default-asr-hardware", "固定识别方式")
                                .options(
                                    [
                                        ("", "自动"),
                                        ("coreml", "Apple 原生"),
                                        ("gpu", "GPU"),
                                        ("cpu", "CPU"),
                                        ("npu", "Intel NPU"),
                                    ]
                                    .into_iter()
                                    .filter(|(id, _)| {
                                        *id != "coreml"
                                            || cfg!(target_os = "macos")
                                            || provider == "coreml"
                                    })
                                    .filter(|(id, _)| {
                                        *id != "npu"
                                            || !cfg!(target_os = "macos")
                                            || provider == "npu"
                                    }),
                                )
                                .selected(provider.to_owned())
                                .on_change(cx.listener(|this, id: &SharedString, _, cx| {
                                    let mut next = this.generation_edit_base();
                                    next.options.provider = match id.as_ref() {
                                        "coreml" => Some(course2md::config::AsrProvider::Coreml),
                                        "gpu" => Some(course2md::config::AsrProvider::Gpu),
                                        "cpu" => Some(course2md::config::AsrProvider::Cpu),
                                        "npu" => Some(course2md::config::AsrProvider::Npu),
                                        _ => None,
                                    };
                                    this.commit_generation(next, cx);
                                })),
                        )
                        .child(
                            text(
                                "asr-fixed-choice-help",
                                "固定方式不可用时会提示原因；自动识别只使用本机能力。",
                            )
                            .text_sm()
                            .text_color(color(MUTED)),
                        ),
                    window,
                    cx,
                ))
                .child(
                    quiet("toggle-model-details")
                        .icon(icons::download())
                        .label(if self.settings_ui.model_details_open {
                            "收起模型管理"
                        } else {
                            "管理模型与下载"
                        })
                        .self_start()
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.settings_ui.model_details_open =
                                !this.settings_ui.model_details_open;
                            cx.notify();
                        })),
                );
            let model_panel = self.default_model_readiness_panel(window, cx);
            recognition = recognition.child(crate::motion::disclosure(
                "default-model-management",
                self.settings_ui.model_details_open,
                model_panel,
                window,
                cx,
            ));
        }
        let mut ai = group("ai-default-settings", "AI 校对与摘要")
            .child(
                self.setting_preference(
                    "AI 校对",
                    "修正识别错误和标点，保留原意与原语言。",
                    Switch::new("default-ai-proofread")
                        .checked(value.ai_proofread)
                        .on_click(cx.listener(|this, enabled, _, cx| {
                            let mut next = this.generation_edit_base();
                            next.ai_proofread = *enabled;
                            this.commit_generation(next, cx);
                        })),
                ),
            )
            .child(
                self.setting_preference(
                    "生成摘要",
                    "提炼课程要点并放在笔记开头。",
                    Switch::new("default-ai-summary")
                        .checked(value.ai_summary)
                        .on_click(cx.listener(|this, enabled, _, cx| {
                            let mut next = this.generation_edit_base();
                            next.ai_summary = *enabled;
                            this.commit_generation(next, cx);
                        })),
                ),
            );
        let mut ai_options =
            v_flex()
                .gap_3()
                .child(self.service_picker(ServicePurpose::Ai, false, cx));
        if value.ai_proofread {
            ai_options = ai_options.child(
                self.setting_preference(
                    "发送截图辅助校对",
                    "文字及对应截图会发送到所选 AI 服务。",
                    Switch::new("default-ai-vision")
                        .checked(value.vision)
                        .on_click(cx.listener(|this, enabled, _, cx| {
                            let mut next = this.generation_edit_base();
                            next.vision = *enabled;
                            this.commit_generation(next, cx);
                        })),
                ),
            );
        }
        ai = ai
            .child(crate::motion::disclosure(
                "ai-default-options",
                value.needs_ai(),
                ai_options,
                window,
                cx,
            ))
            .child(
                quiet("show-proofread-rules")
                    .icon(icons::edit())
                    .label(if value.prompt.is_some() {
                        "编辑自定义校对规则"
                    } else {
                        "自定义校对规则"
                    })
                    .self_start()
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.settings_ui.show_prompt = !this.settings_ui.show_prompt;
                        cx.notify();
                    })),
            )
            .child(crate::motion::disclosure(
                "proofread-rules-content",
                self.settings_ui.show_prompt,
                v_flex()
                    .gap_3()
                    .child(
                        text("standard-proofread-rule", course2md::llm::DEFAULT_PROMPT)
                            .text_sm()
                            .text_color(color(MUTED)),
                    )
                    .child(
                        text("prompt-contract", "填写你希望校对时遵循的写作要求。")
                            .text_sm()
                            .text_color(color(MUTED)),
                    )
                    .child(
                        Textarea::new(&self.settings_ui.prompt)
                            .h(px(160.))
                            .w_full()
                            .aria_label("自定义校对规则"),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .flex_wrap()
                            .child(
                                outline_pill("apply-proofread-rules")
                                    .icon(icons::save())
                                    .label("应用校对规则")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        let prompt = this
                                            .settings_ui
                                            .prompt
                                            .read(cx)
                                            .value()
                                            .trim()
                                            .to_owned();
                                        let mut next = this.generation_edit_base();
                                        next.prompt = (!prompt.is_empty()).then_some(prompt);
                                        next.prompt_draft = None;
                                        this.commit_generation(next, cx);
                                    })),
                            )
                            .child(
                                quiet("restore-proofread-rules")
                                    .icon(icons::refresh())
                                    .label("恢复标准规则")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        let mut next = this.generation_edit_base();
                                        next.prompt = None;
                                        next.prompt_draft = None;
                                        if this.commit_generation(next, cx) {
                                            this.settings_ui.prompt.update(cx, |input, cx| {
                                                input.set_value("", window, cx)
                                            });
                                        }
                                    })),
                            ),
                    ),
                window,
                cx,
            ));
        let selected = value.options.formats.clone().unwrap_or_default();
        v_flex()
            .gap_6()
            .child(
                text(
                    "generation-default-scope",
                    "用于后续生成，也会更新当前未单独修改的选项。",
                )
                .text_sm()
                .text_color(color(MUTED)),
            )
            .child(languages)
            .child(recognition)
            .child(ai)
            .child(
                group("export-default-settings", "导出与离线保存")
                    .child(
                        text(
                            "internal-note-policy",
                            "笔记会自动保存在应用内，也可以同时导出文件。",
                        )
                        .text_sm()
                        .text_color(color(MUTED)),
                    )
                    .child(
                        h_flex().gap_3().flex_wrap().children(
                            [
                                (course2md::config::OutputFormat::Md, "Markdown 包"),
                                (course2md::config::OutputFormat::Html, "网页文件"),
                                (course2md::config::OutputFormat::Json, "JSON 数据"),
                            ]
                            .into_iter()
                            .enumerate()
                            .map(|(index, (format, label))| {
                                Checkbox::new(("default-export", index))
                                    .label(label)
                                    .checked(selected.contains(&format))
                                    .min_h(rems(2.6))
                                    .on_click(cx.listener(move |this, enabled, _, cx| {
                                        let mut next = this.generation_edit_base();
                                        let formats =
                                            next.options.formats.get_or_insert_with(Vec::new);
                                        if !enabled {
                                            formats.retain(|value| *value != format);
                                        } else if !formats.contains(&format) {
                                            formats.push(format);
                                        }
                                        this.commit_generation(next, cx);
                                    }))
                            }),
                        ),
                    )
                    .child(
                        self.setting_preference(
                            "保留视频供离线播放",
                            "保存在线来源的视频，会占用额外空间。",
                            Switch::new("default-keep-video")
                                .checked(value.options.keep_video.unwrap_or(false))
                                .on_click(cx.listener(|this, enabled, _, cx| {
                                    let mut next = this.generation_edit_base();
                                    next.options.keep_video = Some(*enabled);
                                    this.commit_generation(next, cx);
                                })),
                        ),
                    ),
            )
            .into_any_element()
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
        let mut models = vec![("qwen3-1.7b", "Qwen3 1.7B")];
        if provider == Some(AsrProvider::Coreml)
            || (provider.is_none() && cfg!(target_os = "macos"))
            || provider == Some(AsrProvider::Npu)
        {
            models.extend([("qwen3-0.6b", "Qwen3 0.6B"), ("whisper", "Whisper")]);
        }
        if provider == Some(AsrProvider::Npu) {
            models.extend([
                ("whisper-tiny", "Whisper Tiny"),
                ("whisper-base", "Whisper Base"),
                ("whisper-small", "Whisper Small"),
            ]);
        }
        let known = models.iter().any(|(id, _)| *id == selected);
        let picker = if models.len() > 3 {
            let current = selected.to_owned();
            let label = models
                .iter()
                .find(|(id, _)| *id == selected)
                .map(|(_, label)| *label)
                .unwrap_or("自定义模型");
            let entity = cx.entity().downgrade();
            self.reveal_setting(
                "default-local-model-menu-reveal",
                control("default-local-model-menu")
                    .w_full()
                    .label(label)
                    .child(Icon::new(IconName::ChevronDown).size_4())
                    .dropdown_menu(move |menu, _, _| {
                        models.iter().fold(menu, |menu, (id, label)| {
                            let id = (*id).to_owned();
                            let entity = entity.clone();
                            menu.item(PopupMenuItem::new(*label).checked(id == current).on_click(
                                move |_, window, cx| {
                                    let _ =
                                        entity.update(cx, |this, cx| {
                                            let mut next = this.generation_edit_base();
                                            next.options.asr_model = Some(id.clone());
                                            next.local_model_draft = None;
                                            if this.commit_generation(next, cx) {
                                                this.settings_ui.inputs[&EditField::LocalModel]
                                                    .update(cx, |input, cx| {
                                                        input.set_value(id.clone(), window, cx);
                                                    });
                                            }
                                        });
                                },
                            ))
                        })
                    }),
            )
            .into_any_element()
        } else {
            self.setting_choices("default-local-model", "默认识别模型")
                .options(models)
                .full_width()
                .selected(selected.to_owned())
                .on_change(cx.listener(move |this, id: &SharedString, window, cx| {
                    let mut next = this.generation_edit_base();
                    next.options.asr_model = Some(id.to_string());
                    next.local_model_draft = None;
                    if this.commit_generation(next, cx) {
                        this.settings_ui.inputs[&EditField::LocalModel]
                            .update(cx, |input, cx| input.set_value(id.clone(), window, cx));
                    }
                }))
                .into_any_element()
        };
        let mut view = v_flex().w_full().min_w_0().gap_2().child(settings_row(
            "local-model-heading",
            "识别模型",
            "",
            picker,
        ));
        if !known && provider != Some(AsrProvider::Npu) {
            view = view.child(text("unavailable-fixed-model", format!("已保留指定模型 {selected}，当前识别方式不支持此模型。请选择上面的实际模型。")).text_sm().text_color(color(DANGER)));
        }
        if provider == Some(AsrProvider::Npu) {
            view = view
                .child(self.setting_field(EditField::LocalModel, "NPU 模型 ID 或仓库 ID", cx))
                .child(
                    control("apply-custom-npu-model")
                        .icon(icons::check_circle())
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
                view = view.child(text("selected-device-unavailable", "尚未检测到所选识别方式需要的运行环境。已保留这个选择，可在应用的诊断详情中查看原因。").text_sm().text_color(color(WARNING)));
            }
        }
        view
    }

    fn services_settings_page(&self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let mut view = v_flex().w_full().min_w_0().gap_5().child(
            text("service-purpose-help", "连接你使用的语音或 AI 服务。")
                .text_sm()
                .text_color(color(MUTED)),
        );
        // Keep the active form near the start of the page, including in a long service list.
        let inline_editor = self
            .settings_ui
            .editor
            .as_ref()
            .filter(|editor| editor.target.is_none());
        if inline_editor.is_some() {
            return view
                .child(self.service_editor_card(window, cx))
                .into_any_element();
        }
        let mut latest = BTreeMap::<String, ServiceVersion>::new();
        for version in self.preferences.versions() {
            if latest
                .get(&version.service_id)
                .is_none_or(|old| old.number < version.number)
            {
                latest.insert(version.service_id.clone(), version.clone());
            }
        }
        for (purpose, heading_id, heading, add_label) in [
            (
                ServicePurpose::Speech,
                "speech-services-heading",
                "语音服务",
                "添加语音服务",
            ),
            (
                ServicePurpose::Ai,
                "ai-services-heading",
                "AI 服务",
                "添加 AI 服务",
            ),
        ] {
            let mut section = group(heading_id, heading);
            for version in latest
                .values()
                .filter(|version| version.config.protocol.purpose() == purpose)
            {
                section = section.child(self.service_card(version, cx));
            }
            section = section.child(
                outline_pill(("add-service", purpose as usize))
                    .icon(icons::add())
                    .label(add_label)
                    .self_start()
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.open_settings_service_editor(purpose, None, window, cx);
                    })),
            );
            view = view.child(section);
        }
        view.child(
            group("account-settings-heading", "来源账号").child(
                v_flex()
                    .w_full()
                    .gap_3()
                    .p_4()
                    .bg(color(SURFACE))
                    .border_1()
                    .border_color(color(CARD_LINE))
                    .rounded(RADIUS_CARD)
                    .child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .child(icons::bilibili().size_5())
                            .child(
                                text("account-card-title", "Bilibili 账号")
                                    .font_weight(FontWeight::SEMIBOLD),
                            ),
                    )
                    .child(self.account_settings_page(cx)),
            ),
        )
        .into_any_element()
    }

    /// One saved service as a mock 服务行卡: name + status badge + meta + quiet actions.
    fn service_card(&self, version: &ServiceVersion, cx: &mut Context<Self>) -> Div {
        let stopped = self
            .preferences
            .service_stopped_in_snapshot(&version.service_id);
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
                    " · 本次调整未设为默认".to_owned()
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
        let passed = self
            .preferences
            .test_evidence(&version.config, kinds[0].contract())
            .filter(|evidence| evidence.outcome == preferences::TestOutcome::Passed);
        let has_failed_test = kinds.iter().any(|kind| {
            self.preferences
                .test_evidence(&version.config, kind.contract())
                .is_some_and(|evidence| evidence.outcome != preferences::TestOutcome::Passed)
        });
        let (status_kind, status_label) = if stopped {
            (BadgeKind::Neutral, "已停用".to_owned())
        } else if has_failed_test {
            (BadgeKind::Warning, "有测试未通过".to_owned())
        } else if let Some(evidence) = passed {
            let stamp = crate::reader_navigation::timestamp_local(evidence.tested_at * 1000);
            let date = stamp.get(..10).unwrap_or(&stamp).to_owned();
            (BadgeKind::Success, format!("测试通过 · {date}"))
        } else {
            (BadgeKind::Neutral, "配置已保存".to_owned())
        };
        let status_badge = badge(status_kind).child(text(
            SharedString::from(format!("saved-service-badge-{id}")),
            status_label,
        ));
        let edit_id = id.clone();
        let test_id = id.clone();
        v_flex()
            .w_full()
            .gap_2()
            .p_4()
            .bg(color(SURFACE))
            .border_1()
            .border_color(color(CARD_LINE))
            .rounded(RADIUS_CARD)
            .child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .flex_wrap()
                    .child(
                        text(
                            SharedString::from(format!("saved-service-title-{id}")),
                            version.config.name.clone(),
                        )
                        .font_weight(FontWeight::SEMIBOLD),
                    )
                    .child(status_badge)
                    .child(div().flex_1())
                    .when(!stopped, |row| {
                        row.child(
                            quiet(SharedString::from(format!("test-saved-service-{id}")))
                                .icon(icons::science())
                                .label("测试服务")
                                .on_click(cx.listener(move |this, _, window, cx| {
                                    if this.open_settings_service_editor(
                                        purpose,
                                        Some(test_id.clone()),
                                        window,
                                        cx,
                                    ) {
                                        this.start_service_test(window, cx);
                                    }
                                })),
                        )
                        .child(
                            quiet(SharedString::from(format!("edit-saved-service-{id}")))
                                .icon(icons::edit())
                                .label("编辑")
                                .on_click(cx.listener(move |this, _, window, cx| {
                                    this.open_settings_service_editor(
                                        purpose,
                                        Some(edit_id.clone()),
                                        window,
                                        cx,
                                    );
                                })),
                        )
                    }),
            )
            .child(
                text(
                    SharedString::from(format!("saved-service-description-{id}")),
                    format!(
                        "{} · {}{}",
                        version.config.model,
                        version.config.host(),
                        default_description,
                    ),
                )
                .text_size(TEXT_AUX)
                .text_color(color(GRAY)),
            )
            .child(
                text(
                    SharedString::from(format!("saved-service-test-status-{id}")),
                    if tests.is_empty() {
                        "尚未测试".to_owned()
                    } else {
                        tests.join("\n")
                    },
                )
                .text_size(TEXT_AUX)
                .text_color(color(GRAY)),
            )
            .when(!stopped, |card| {
                card.child(
                    quiet(SharedString::from(format!(
                        "stop-saved-service-{service_id}"
                    )))
                    .icon(icons::close())
                    .label("停止使用此服务")
                    .self_start()
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.confirm_stop_service(service_id.clone(), window, cx)
                    })),
                )
            })
    }

    /// Ordinary service editing with explicit Save and Cancel actions.
    fn service_editor_card(&self, window: &mut Window, cx: &mut Context<Self>) -> Div {
        let Some(editor) = &self.settings_ui.editor else {
            return v_flex();
        };
        let purpose = editor.draft.protocol.purpose();
        let kind = if purpose == ServicePurpose::Speech {
            "语音服务"
        } else {
            "AI 服务"
        };
        let title = if editor.draft.based_on.is_some() {
            format!("编辑{kind}")
        } else {
            format!("添加 {kind}")
        };
        v_flex()
            .w_full()
            .gap_3()
            .p_4()
            .bg(color(SURFACE))
            .border_1()
            .border_color(color(CONTROL))
            .rounded(RADIUS_CARD)
            .child(text("service-editor-inline-title", title).font_weight(FontWeight::SEMIBOLD))
            .child(self.service_editor_content(true, window, cx))
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
        let editor_version = current
            .as_deref()
            .and_then(|id| self.preferences.version(id))
            .filter(|version| {
                !self
                    .preferences
                    .service_stopped_in_snapshot(&version.service_id)
            })
            .map(|version| version.id.clone());
        let mut latest = BTreeMap::<String, ServiceVersion>::new();
        for version in self.preferences.versions().filter(|v| {
            v.config.protocol.purpose() == purpose
                && !self.preferences.service_stopped_in_snapshot(&v.service_id)
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
                        "{} · {}{}",
                        if version.config.name == version.config.host() {
                            version.config.name.clone()
                        } else {
                            format!("{} · {}", version.config.name, version.config.host())
                        },
                        version.config.model,
                        if self
                            .preferences
                            .service_stopped_in_snapshot(&version.service_id)
                        {
                            " · 已停用"
                        } else {
                            ""
                        }
                    ),
                )
                .text_sm(),
            );
        }
        let old_current_service = current
            .as_deref()
            .and_then(|id| self.preferences.version(id))
            .filter(|version| {
                latest
                    .get(&version.service_id)
                    .is_some_and(|latest| latest.id != version.id)
            })
            .map(|version| version.service_id.clone());
        let mut choices = latest.into_values().collect::<Vec<_>>();
        if let Some(current) = current
            .as_deref()
            .and_then(|id| self.preferences.version(id))
            && !choices.iter().any(|version| version.id == current.id)
        {
            choices.push(current.clone());
        }
        if !choices.is_empty() {
            let mut picker = self
                .setting_choices(
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
                            "{} · {}{}{}",
                            version.config.name,
                            version.config.model,
                            if old_current_service.as_ref() == Some(&version.service_id) {
                                if current.as_ref() == Some(&version.id) {
                                    " · 当前使用"
                                } else {
                                    " · 更新后的配置"
                                }
                            } else {
                                ""
                            },
                            if self
                                .preferences
                                .service_stopped_in_snapshot(&version.service_id)
                            {
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
                if self
                    .preferences
                    .service_stopped_in_snapshot(&version.service_id)
                {
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
            .icon(icons::settings())
            .ghost()
            .label(match (purpose, editor_version.is_some()) {
                (ServicePurpose::Speech, true) => "编辑语音服务",
                (ServicePurpose::Speech, false) => "添加语音服务",
                (ServicePurpose::Ai, true) => "编辑 AI 服务",
                (ServicePurpose::Ai, false) => "添加 AI 服务",
            })
            .self_start()
            .on_click(cx.listener(move |this, _, window, cx| {
                if current_task {
                    this.open_task_service_editor(purpose, window, cx)
                } else {
                    this.open_settings_service_editor(purpose, editor_version.clone(), window, cx);
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
                        .text_color(color(MUTED)),
                    )
                    .child(
                        control(("inherit-default-service", purpose as usize))
                            .ghost()
                            .icon(icons::refresh())
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
                    .text_color(color(DANGER)),
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
                .ok_or_else(|| anyhow!("当前笔记信息暂时不可用"))
                .and_then(|workspace| {
                    workspace.transaction(|state| {
                        let draft = state.draft_mut().context("找不到当前笔记")?;
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
                self.set_settings_feedback(
                    PreferenceGroup::Services,
                    "服务选择已保存".into(),
                    false,
                );
                self.advance_conversion_when_ready(cx);
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
    ) -> bool {
        let draft = version
            .as_deref()
            .and_then(|id| self.preferences.version(id))
            .map(ServiceDraft::from_version)
            .unwrap_or_else(|| ServiceDraft::new(purpose));
        self.open_service_draft(draft, None, window, cx)
    }
    fn open_service_draft(
        &mut self,
        draft: ServiceDraft,
        target: Option<String>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if let Some(editor) = &self.settings_ui.editor {
            if editor.target.is_some() {
                return false;
            }
            if editor.draft.id == draft.id && editor.target == target {
                self.settings_tab = 1;
                self.navigate(Page::Settings, cx);
                self.scrolls[Page::Settings as usize].set_offset(point(px(0.), px(0.)));
                self.settings_ui.inputs[&EditField::Name]
                    .update(cx, |input, cx| input.focus(window, cx));
                return true;
            }
            // A second action must not reuse the previous editor's endpoint or scope.
            // Switching forms cancels the earlier edit; only Save can publish it.
            if !self.close_service_editor(window, cx) {
                self.settings_tab = 1;
                self.navigate(Page::Settings, cx);
                self.scrolls[Page::Settings as usize].set_offset(point(px(0.), px(0.)));
                return false;
            }
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
        let inline = target.is_none();
        let defaults = self.preferences.default_refs();
        let default = match draft.protocol.purpose() {
            ServicePurpose::Speech => defaults.asr,
            ServicePurpose::Ai => defaults.llm,
        };
        let also_default = default
            .as_deref()
            .and_then(|id| self.preferences.version(id))
            .is_none_or(|version| {
                self.preferences
                    .service_stopped_in_snapshot(&version.service_id)
            });
        self.settings_ui.editor = Some(ServiceEditor {
            draft,
            target,
            also_default,
            return_focus: window.focused(cx),
            errors: Vec::new(),
            status: None,
            test_kind: kind,
            test_running: None,
            evidence,
            pending_binding: None,
            show_key: false,
            test_details_open: false,
            save_failed: false,
        });
        if inline {
            self.settings_tab = 1;
            self.navigate(Page::Settings, cx);
            self.scrolls[Page::Settings as usize].set_offset(point(px(0.), px(0.)));
            self.settings_ui.inputs[&EditField::Name]
                .update(cx, |input, cx| input.focus(window, cx));
            cx.notify();
            return true;
        }
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
        true
    }
    fn service_editor_content(
        &self,
        inline: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let Some(editor) = &self.settings_ui.editor else {
            return div().into_any_element();
        };
        let protocol = editor.draft.protocol;
        let auth = editor.draft.authentication;
        let busy = editor.test_running.is_some();
        let awaiting_binding = editor.pending_binding.is_some();
        let mut view = v_flex()
            .id("service-editor-body")
            .role(if inline { Role::Group } else { Role::Dialog })
            .aria_label(if protocol.purpose() == ServicePurpose::Speech {
                "设置语音服务"
            } else {
                "设置 AI 服务"
            })
            .gap_4()
            .px_1();
        if !inline {
            view = view
                .max_h((window.bounds().size.height - window.rem_size() * 10.).max(px(80.)))
                .min_h_0()
                .overflow_y_scroll();
        }
        view = view.child(
            text(
                "service-editor-scope",
                if editor.target.is_some() {
                    "保存后用于本次笔记。已提交的任务不受影响。"
                } else {
                    "保存后设为默认服务。已提交的任务不受影响。"
                },
            )
            .text_sm()
            .text_color(color(MUTED)),
        );
        view = view
            .child(self.setting_field(EditField::Name, "服务名称", cx))
            .when(protocol.purpose() == ServicePurpose::Speech, |view| {
                view.child(
                v_flex()
                    .gap_2()
                    .child(field_label("service-protocol-heading", "接口类型"))
                    .child(
                        self.setting_choices("service-protocol", "服务接口类型")
                            .options(
                                [
                                    ServiceProtocol::SpeechTranscriptions,
                                    ServiceProtocol::SpeechChat,
                                    ServiceProtocol::AiChat,
                                ]
                                .into_iter()
                                .filter(|candidate| candidate.purpose() == protocol.purpose())
                                .map(|candidate| {
                                    (candidate.label(), service_protocol_label(candidate))
                                }),
                            )
                            .selected(protocol.label())
                            .disabled(awaiting_binding)
                            .on_change(cx.listener(move |this, selected: &SharedString, _, cx| {
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
                                cx.notify();
                            })),
                    ),
            )
            })
            .child(self.setting_field(EditField::Address, "服务地址", cx));
        if let Ok(endpoint) =
            preferences::normalize_endpoint(&self.setting_value(EditField::Address, cx), protocol)
        {
            view = view.child(
                text("service-actual-endpoint", format!("将请求 {endpoint}"))
                    .text_sm()
                    .text_color(color(MUTED)),
            );
        }
        view = view
            .child(self.setting_field(EditField::Model, "模型 ID", cx))
            .child(
                v_flex()
                    .gap_2()
                    .child(field_label("service-auth-heading", "认证方式"))
                    .child(
                        self.setting_choices("service-auth-mode", "服务认证方式")
                            .options([("api_key", "API Key"), ("none", "无需认证")])
                            .selected(if auth == Authentication::ApiKey {
                                "api_key"
                            } else {
                                "none"
                            })
                            .disabled(awaiting_binding)
                            .on_change(cx.listener(move |this, selected: &SharedString, _, cx| {
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
                                cx.notify();
                            })),
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
                        .icon(icons::content_copy())
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
                                        editor.status =
                                            Some("已使用环境变量中的密钥，点击保存后生效".into());
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
                    .text_color(color(MUTED)),
                );
            }
            view = view.child(
                control("toggle-service-key")
                    .icon(if editor.show_key {
                        icons::eye_off()
                    } else {
                        icons::eye()
                    })
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
                    format!("密钥已安全保存 · 来源：{source}"),
                )
                .text_sm(),
            );
        }
        if editor.target.is_some() {
            view = view.child(
                self.setting_preference(
                    "同时设为默认服务",
                    "后续转换自动使用",
                    Switch::new("service-also-default")
                        .checked(editor.also_default)
                        .disabled(awaiting_binding)
                        .on_click(cx.listener(|this, value, _, cx| {
                            if let Some(editor) = &mut this.settings_ui.editor {
                                editor.also_default = *value;
                            }
                            cx.notify();
                        })),
                ),
            );
        }
        view = view.child(
            text("service-test-notice", service_test::TEST_NOTICE)
                .text_sm()
                .text_color(color(MUTED)),
        );
        if protocol == ServiceProtocol::AiChat {
            view = view.child(
                self.setting_choices("service-test-purpose", "要测试的服务能力")
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
        if busy {
            view = view.child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(crate::motion::spinner("service-test-busy", cx))
                    .child(
                        text(
                            "service-test-busy-label",
                            editor
                                .status
                                .clone()
                                .unwrap_or_else(|| "正在测试服务…".into()),
                        )
                        .text_sm(),
                    ),
            );
        }
        if let Some(evidence) = &editor.evidence {
            let (kind, label) = match evidence.outcome {
                preferences::TestOutcome::Passed => (
                    BadgeKind::Success,
                    format!("{}测试通过", editor.test_kind.label()),
                ),
                preferences::TestOutcome::OutcomeUnknown => {
                    (BadgeKind::Warning, "结果尚未确认".to_owned())
                }
                preferences::TestOutcome::NotSent => (BadgeKind::Neutral, "测试未发送".to_owned()),
                _ => (BadgeKind::Danger, "测试未通过".to_owned()),
            };
            let mut result = v_flex()
                .gap_3()
                .p_4()
                .rounded(RADIUS_CARD)
                .bg(color(match evidence.outcome {
                    preferences::TestOutcome::Passed => SUCCESS_BG,
                    preferences::TestOutcome::OutcomeUnknown => WARNING_BG,
                    preferences::TestOutcome::NotSent => INSET,
                    _ => DANGER_BG,
                }))
                .child(badge(kind).child(label))
                .when(
                    evidence.outcome != preferences::TestOutcome::Passed,
                    |view| {
                        view.child(text("service-test-result", evidence.message.clone()).text_sm())
                    },
                );
            if evidence.outcome == preferences::TestOutcome::Passed {
                result = result
                    .child(text("service-tested-not-saved", "保存服务后使用这些更改。").text_sm());
            }
            if !evidence.details.is_empty() {
                result = result.child(
                    quiet("service-test-details")
                        .icon(icons::info())
                        .self_start()
                        .label(if editor.test_details_open {
                            "收起测试详情"
                        } else {
                            "查看测试详情"
                        })
                        .on_click(cx.listener(|this, _, _, cx| {
                            if let Some(editor) = &mut this.settings_ui.editor {
                                editor.test_details_open = !editor.test_details_open;
                            }
                            cx.notify();
                        })),
                );
                let details = v_flex()
                    .gap_2()
                    .children(evidence.details.iter().enumerate().map(|(index, detail)| {
                        text(("service-test-detail", index), detail.clone())
                            .text_sm()
                            .text_color(color(MUTED))
                    }));
                result = result.child(crate::motion::disclosure(
                    "service-test-details-content",
                    editor.test_details_open,
                    details,
                    window,
                    cx,
                ));
            }
            view = view.child(crate::motion::enter(
                SharedString::from(format!(
                    "service-test-result-{}-{:?}",
                    evidence.tested_at, evidence.outcome
                )),
                result,
                cx,
            ));
        }
        if let Some(status) = &editor.status
            && !busy
        {
            view = view.child(text("service-editor-status", status.clone()).text_sm());
        }
        let actions = h_flex()
            .w_full()
            .flex_shrink_0()
            .gap_2()
            .flex_wrap()
            .child(
                quiet("close-service-editor")
                    .icon(icons::close())
                    .label("取消")
                    .on_click(cx.listener(move |this, _, window, cx| {
                        if this.close_service_editor(window, cx) && !inline {
                            window.close_dialog(cx);
                        }
                    })),
            )
            .child(div().flex_1())
            .child(
                outline_pill("run-service-test")
                    .icon(icons::science())
                    .label(
                        match editor.evidence.as_ref().map(|evidence| evidence.outcome.clone()) {
                            Some(preferences::TestOutcome::Passed) => "测试通过 · 重测",
                            Some(
                                preferences::TestOutcome::AuthenticationRefused
                                | preferences::TestOutcome::ModelRefused
                                | preferences::TestOutcome::ContractMismatch
                                | preferences::TestOutcome::SampleMismatch,
                            ) => "测试未通过 · 重试",
                            _ => "测试服务",
                        },
                    )
                    .loading(busy)
                    .disabled(busy || awaiting_binding)
                    .on_click(
                        cx.listener(|this, _, window, cx| this.start_service_test(window, cx)),
                    ),
            )
            .child(
                primary_pill("save-service")
                    .icon(icons::save())
                    .label(if awaiting_binding {
                        "重试保存本次选择"
                    } else {
                        "保存"
                    })
                    .on_click(
                        cx.listener(|this, _, window, cx| this.publish_open_service(window, cx)),
                    ),
            );
        if inline {
            view.child(actions).into_any_element()
        } else {
            v_flex()
                .gap_4()
                .child(view)
                .child(actions)
                .into_any_element()
        }
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
            editor.status = None;
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
                    "点击应用后生效".into(),
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
                    editor.status = None;
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
        let inline = target.is_none();
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
                .ok_or_else(|| anyhow!("当前笔记信息暂时不可用"))
                .and_then(|workspace| {
                    workspace.transaction(|state| {
                        let draft = state
                            .drafts
                            .iter_mut()
                            .find(|draft| draft.id == target)
                            .context("原笔记已关闭；服务已保存，但未用于其他笔记")?;
                        let selected =
                            (scope == BindingScope::CurrentTask).then(|| version.id.clone());
                        match version.config.protocol.purpose() {
                            ServicePurpose::Speech => draft.asr_service = selected,
                            ServicePurpose::Ai => draft.ai_service = selected,
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
        if !inline {
            window.close_dialog(cx);
        }
        self.advance_conversion_when_ready(cx);
        cx.notify();
    }
    pub(crate) fn close_service_editor(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if let Some(editor) = &self.settings_ui.editor {
            // Tests can stage a private edit, but cancelling never publishes
            // it. A cleanup failure cannot make these hidden records active.
            let _ = self.preferences.discard_service_draft(&editor.draft.id);
        }
        self.restore_service_focus(window, cx);
        self.settings_ui.inputs[&EditField::Key]
            .update(cx, |input, cx| input.set_value("", window, cx));
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

    pub(crate) fn commit_generation(
        &mut self,
        next: GenerationPreferences,
        cx: &mut Context<Self>,
    ) -> bool {
        let previous = self
            .ordinary_preferences_submit_issue()
            .map(|issue| issue.message);
        match self.preferences.save_generation(next.clone()) {
            Ok(()) => {
                self.settings_ui.pending_generation = None;
                self.set_settings_feedback(PreferenceGroup::Generation, "已保存".into(), false);
                self.refresh_preference_defaults(cx);
                self.clear_resolved_generation_errors(previous);
                if !self.preference_defaults_pending {
                    self.advance_conversion_when_ready(cx);
                }
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
    pub(crate) fn refresh_preference_defaults(&mut self, cx: &mut Context<Self>) {
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
                    if self.preference_defaults_pending {
                        self.preference_defaults_pending = false;
                        self.workspace_error = None;
                    }
                }
                Err(error) => {
                    self.preference_defaults_pending = true;
                    self.workspace_error =
                        Some(format!("默认设置已保存，当前视频的选项尚未同步：{error:#}"))
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
    pub(crate) fn settings_group_notice(&self, group: PreferenceGroup) -> Option<(String, bool)> {
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
    pub(crate) fn group_feedback(&self, group: PreferenceGroup, cx: &mut Context<Self>) -> Div {
        self.group_feedback_with_retry_emphasis(group, false, cx)
    }
    pub(crate) fn group_feedback_with_retry_emphasis(
        &self,
        group: PreferenceGroup,
        primary_retry: bool,
        cx: &mut Context<Self>,
    ) -> Div {
        let mut view = v_flex().w_full().min_w_0().gap_2();
        if let Some((message, error)) = self.settings_group_notice(group) {
            view = view.child(crate::motion::enter(
                SharedString::from(format!("preference-feedback-{}-{message}", group as usize)),
                h_flex()
                    .w_full()
                    .min_w_0()
                    .gap_2()
                    .items_center()
                    .child(
                        if error {
                            icons::warning()
                        } else {
                            icons::check_circle()
                        }
                        .size_4()
                        .flex_shrink_0()
                        .text_color(color(if error {
                            DANGER
                        } else {
                            SUCCESS
                        })),
                    )
                    .child(
                        text(("settings-group-feedback", group as usize), message.clone())
                            .flex_1()
                            .min_w_0()
                            .text_size(TEXT_BODY)
                            .text_color(color(if error { DANGER } else { MUTED })),
                    ),
                cx,
            ));
            let has_pending = match group {
                PreferenceGroup::Generation => self.settings_ui.pending_generation.is_some(),
                PreferenceGroup::Application => self.settings_ui.pending_application.is_some(),
                PreferenceGroup::Services => false,
            };
            if error && has_pending {
                let id = ("retry-settings-save", group as usize);
                let retry = if primary_retry {
                    primary_pill(id)
                } else {
                    control(id)
                };
                view = view.child(
                    retry
                        .icon(icons::refresh())
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
                        .icon(icons::info())
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
                            .text_color(color(MUTED)),
                    );
                }
            }
        }
        if self.preferences.is_blocked(group) {
            view = view.child(
                control(("reset-settings-group", group as usize))
                    .icon(icons::refresh())
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
        let mut view = v_flex().gap_5().child(
            h_flex()
                .w_full()
                .min_w_0()
                .gap_3()
                .flex_wrap()
                .child(
                    text(
                        "storage-policy",
                        "在这些位置保存笔记。更改默认位置只影响后续生成的笔记。",
                    )
                    .flex_1()
                    .min_w_0()
                    .text_sm()
                    .text_color(color(MUTED)),
                )
                .child(
                    quiet("refresh-storage-locations")
                        .icon(icons::refresh())
                        .label("刷新")
                        .loading(self.loading)
                        .disabled(self.loading)
                        .on_click(cx.listener(|this, _, _, cx| this.refresh_library(cx))),
                ),
        );
        view = view.child(self.storage_status_panel(cx));
        if let Some(workspace) = &self.workspace {
            for (index, library) in workspace.state.libraries.iter().enumerate() {
                let root = library.root.clone();
                let id = library.id.clone();
                let move_id = id.clone();
                let relocate_id = id.clone();
                let default = workspace.state.default_library == id;
                let check = self.cached_location_check(library);
                let offline = check.is_some_and(|check| !check.available);
                let needs_relocation = check.is_some_and(|check| {
                    !check.available || check.problem.is_some() || check.needs_reassociation
                });
                view=view.child(v_flex().gap_2().w_full().p_4().bg(color(SURFACE)).border_1().border_color(color(CARD_LINE)).rounded(RADIUS_CARD)
                .child(h_flex().gap_2().items_center().flex_wrap()
                    .child(text(("storage-location-name",index),library.name.clone()).font_weight(FontWeight::SEMIBOLD))
                    .when(default,|row|row.child(badge(BadgeKind::Neutral).child("默认保存位置")))
                    .when(offline,|row|row.child(badge(BadgeKind::Warning).child("暂时不可访问")))
                    .child(div().flex_1())
                    .when(!default, |row| row.child(outline_pill(("default-storage-location",index)).icon(icons::check_circle()).label("设为默认")
                        .on_click(cx.listener(move|this,_,_,cx|{
                        if let Some(workspace)=&mut this.workspace{
                            let result=workspace.state.library(&id).ok_or_else(||anyhow!("此位置已不在课程库中")).and_then(|library|tempfile::NamedTempFile::new_in(&library.root).map(drop).context("此位置暂时不能写入，请重新连接磁盘或恢复文件夹访问权限"))
                                .and_then(|_|workspace.transaction(|state|{state.default_library=id.clone();Ok(())}));
                            if let Err(error)=result{this.workspace_error=Some(format!("默认位置尚未更改：{error:#}"));}else{this.message=Some("后续生成的笔记将保存在此位置，已有笔记和任务保持原位置".into());}
                        }cx.notify();
                    }))))
                    .when(!needs_relocation, |row| row.child(outline_pill(("move-storage-location",index)).icon(icons::storage()).label("移动课程库…").disabled(self.storage_ui.busy).on_click(cx.listener(move|this,_,window,cx|this.begin_library_move(move_id.clone(),window,cx))))))
                .child(text(("storage-location-path",index),root.display().to_string()).text_size(TEXT_AUX).text_color(color(GRAY)))
                .when(needs_relocation, |row| row.child(text(("storage-relocation-help",index),"文件夹已移动？选择它现在的位置即可恢复关联。").text_size(TEXT_AUX).text_color(color(MUTED))))
                .child(h_flex().gap_2().flex_wrap()
                    .child(quiet(("open-storage-location",index)).icon(icons::folder_open()).label("打开位置").disabled(offline).on_click(move|_,_,cx|cx.open_with_system(&root)))
                    .when(needs_relocation, |row| row.child(outline_pill(("relocate-storage-location",index)).icon(icons::folder_open()).label("重新定位…").disabled(self.storage_ui.busy).on_click(cx.listener(move|this,_,window,cx|this.begin_library_relocation(relocate_id.clone(),window,cx)))))));
            }
        }
        view = view.child(
            outline_pill("register-storage-location")
                .icon(icons::add())
                .label("添加课程库位置")
                .self_start()
                .on_click(
                    cx.listener(|this, _, window, cx| this.register_storage_location(window, cx)),
                ),
        );
        if let Some(workspace) = &self.workspace {
            let details: Vec<String> = workspace
                .state
                .libraries
                .iter()
                .map(|library| {
                    format!(
                        "{} · 库 ID {} · {}",
                        library.name,
                        library.id,
                        library.root.display()
                    )
                })
                .collect();
            if !details.is_empty() {
                let open = self.settings_ui.storage_details_open;
                view = view
                    .child(
                        quiet("toggle-storage-details")
                            .icon(icons::info())
                            .self_start()
                            .label(if open {
                                "收起技术详情"
                            } else {
                                "查看存储详情"
                            })
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.settings_ui.storage_details_open =
                                    !this.settings_ui.storage_details_open;
                                cx.notify();
                            })),
                    )
                    .when(open, |view| {
                        view.children(details.into_iter().enumerate().map(|(index, line)| {
                            text(("storage-detail", index), line)
                                .text_size(TEXT_AUX)
                                .text_color(color(GRAY))
                        }))
                    });
            }
        }
        view.into_any_element()
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
                            if !duplicate
                                && let Ok(identity) = std::fs::read_to_string(path.join(".course2md-library-id"))
                                && let Some(existing) = this.workspace.as_ref().and_then(|workspace| workspace.state.library(identity.trim()))
                            {
                                this.workspace_error = Some(format!(
                                    "「{}」已登记。请在这个课程库下选择“重新定位…”，恢复移动后的关联。",
                                    existing.name
                                ));
                                cx.notify();
                                return;
                            }
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
    pub(crate) fn appearance_controls(&self, cx: &mut Context<Self>) -> Div {
        group("appearance-motion", "界面偏好")
            .child(settings_row(
                "app-font-scale-label",
                "界面文字大小",
                "",
                self.setting_choices("app-font-scale", "应用文字大小")
                    .options([1.0_f32, 1.25, 1.5, 2.0].into_iter().map(|scale| {
                        (
                            (scale * 100.).round().to_string(),
                            format!("{}%", (scale * 100.) as u32),
                        )
                    }))
                    .full_width()
                    .selected(
                        (self.preferences.application().font_scale * 100.)
                            .round()
                            .to_string(),
                    )
                    .on_change(
                        cx.listener(move |this, selected: &SharedString, window, cx| {
                            let Ok(percent) = selected.parse::<f32>() else {
                                return;
                            };
                            let scale = percent / 100.;
                            let mut next = this.application_edit_base();
                            next.font_scale = scale;
                            if this.commit_application(next, cx) {
                                theme::apply_scale(scale, window, cx);
                            }
                        }),
                    ),
            ))
            .child(
                self.setting_preference(
                    "减少动态效果",
                    "关闭过渡与循环动画。",
                    Switch::new("app-reduce-motion")
                        .checked(self.preferences.application().desktop.reduce_motion)
                        .on_click(cx.listener(|this, enabled, _, cx| {
                            let mut next = this.application_edit_base();
                            next.desktop.reduce_motion = *enabled;
                            this.commit_application(next, cx);
                        })),
                ),
            )
    }
    fn application_settings_page(&self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let legacy = &self.settings_ui.legacy;
        let show_legacy = legacy.problem.is_some()
            || (!self.preferences.legacy_imported() && legacy.importable)
            || self.settings_ui.legacy_status.is_some()
            || self.settings_ui.legacy_preserved.is_some()
            || self.settings_ui.legacy_action_detail.is_some();
        v_flex()
            .w_full()
            .min_w_0()
            .gap_6()
            .child(group("about-heading", "关于").child(self.about_page(cx)))
            .child(settings_row(
                "restart-onboarding-label",
                "用户引导",
                "选择默认引擎、配置 AI 服务和准备模型",
                outline_pill("restart-onboarding")
                    .label("重新开始用户引导")
                    .icon(icons::restart())
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.start_onboarding(window, cx);
                    })),
            ))
            .when(show_legacy, |view| {
                view.child(
                    group("legacy-settings-heading", "旧版设置")
                        .child(self.legacy_migration_panel(cx)),
                )
            })
            .child(self.environment_page(window, cx))
            .into_any_element()
    }
    pub(crate) fn commit_application(
        &mut self,
        next: ApplicationPreferences,
        cx: &mut Context<Self>,
    ) -> bool {
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
    fn refresh_legacy_inspection(&mut self) {
        self.settings_ui.legacy =
            crate::legacy_settings::Inspection::inspect(course2md::settings::config_path());
        self.config_error = self.settings_ui.legacy.problem.is_some();
    }
    fn repair_legacy_configuration(&mut self, use_backup: bool, cx: &mut Context<Self>) {
        let path = self.settings_ui.legacy.path.clone();
        let result = if use_backup {
            crate::legacy_settings::restore_backup(&path)
        } else {
            crate::legacy_settings::reset_preserving_original(&path)
        };
        self.settings_ui.legacy_action_detail = None;
        match result {
            Ok(original) => {
                self.settings_ui.legacy_preserved = Some(original);
                self.settings_ui.legacy_status = Some(
                    if use_backup {
                        "已恢复旧配置备份。可明确导入；当前设置继续使用。"
                    } else {
                        "已保留损坏原文件并重建旧配置。当前设置与笔记保持原样。"
                    }
                    .into(),
                );
            }
            Err(error) => {
                self.settings_ui.legacy_status =
                    Some(crate::legacy_settings::failure_message(&error).into());
                self.settings_ui.legacy_action_detail = Some(format!("{error:#}"));
            }
        }
        self.refresh_legacy_inspection();
        cx.notify();
    }
    fn legacy_migration_panel(&self, cx: &mut Context<Self>) -> Div {
        let legacy = &self.settings_ui.legacy;
        let mut view = v_flex().gap_2();
        if let Some(problem) = &legacy.problem {
            view = view
                .child(text("legacy-settings-problem", problem.message.clone()))
                .child(
                    h_flex()
                        .gap_2()
                        .flex_wrap()
                        .when(legacy.backup_available, |row| {
                            row.child(
                                control("restore-legacy-backup")
                                    .icon(icons::refresh())
                                    .label("恢复已验证的旧配置备份")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.repair_legacy_configuration(true, cx)
                                    })),
                            )
                        })
                        .child(
                            control("reset-legacy-preserving-original")
                                .icon(icons::refresh())
                                .label("保留原文件并重建旧配置")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.repair_legacy_configuration(false, cx)
                                })),
                        ),
                );
        } else if !self.preferences.legacy_imported() && legacy.importable {
            view = view.child(text("legacy-settings-found", "发现旧版配置。导入前会备份原文件；已修改的当前设置保持原样。"))
                .child(control("import-legacy-settings").icon(icons::file_upload()).label("备份并导入旧版设置").self_start()
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.settings_ui.legacy_action_detail = None;
                        match crate::legacy_settings::import_preferences(&this.settings_ui.legacy.path, &mut this.preferences) {
                            Ok(original) => {
                                this.settings_ui.legacy_preserved = Some(original);
                                this.settings_ui.legacy_status = Some("已导入旧版设置；原文件及备份保留，后续以当前设置为准。".into());
                                this.settings_ui.initialized = false;
                                this.hydrate_settings_inputs(window, cx);
                                this.refresh_preference_defaults(cx);
                            }
                            Err(error) => {
                                this.settings_ui.legacy_status = Some("旧版设置尚未完整导入。现有设置与原文件均保留，可查看原因后重试。".into());
                                this.settings_ui.legacy_action_detail = Some(format!("{error:#}"));
                            }
                        }
                        this.refresh_legacy_inspection();
                        cx.notify();
                    })));
        }
        if let Some(message) = &self.settings_ui.legacy_status {
            view = view.child(text("legacy-import-status", message.clone()).text_sm());
        }
        if let Some(original) = &self.settings_ui.legacy_preserved {
            let path = original.clone();
            view = view
                .child(
                    text(
                        "legacy-original-backup",
                        format!("原文件备份：{}", original.display()),
                    )
                    .text_sm(),
                )
                .child(
                    control("reveal-legacy-original")
                        .icon(icons::folder_open())
                        .label("显示原文件备份")
                        .self_start()
                        .on_click(move |_, _, cx| cx.reveal_path(&path)),
                );
        }
        if legacy.problem.is_some() || self.settings_ui.legacy_action_detail.is_some() {
            let original = legacy.path.clone();
            view = view.child(
                h_flex()
                    .gap_2()
                    .flex_wrap()
                    .child(
                        control("recheck-legacy-settings")
                            .icon(icons::refresh())
                            .label("重新检查旧配置")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.refresh_legacy_inspection();
                                this.settings_ui.legacy_action_detail = None;
                                this.settings_ui.legacy_status = None;
                                cx.notify();
                            })),
                    )
                    .child(
                        control("reveal-legacy-settings")
                            .icon(icons::folder_open())
                            .label("显示旧配置文件")
                            .on_click(move |_, _, cx| cx.reveal_path(&original)),
                    )
                    .child(
                        control("legacy-settings-details")
                            .icon(icons::info())
                            .label(if self.settings_ui.legacy_details_open {
                                "收起旧配置详情"
                            } else {
                                "查看旧配置详情"
                            })
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.settings_ui.legacy_details_open =
                                    !this.settings_ui.legacy_details_open;
                                cx.notify();
                            })),
                    ),
            );
            if self.settings_ui.legacy_details_open {
                let details = [
                    legacy
                        .problem
                        .as_ref()
                        .map(|problem| problem.detail.as_str()),
                    legacy.backup_detail.as_deref(),
                    self.settings_ui.legacy_action_detail.as_deref(),
                ]
                .into_iter()
                .flatten()
                .collect::<Vec<_>>()
                .join("\n");
                view = view.child(text("legacy-settings-detail-text", details).text_sm());
            }
        }
        view
    }
    fn environment_page(&self, window: &mut Window, cx: &mut Context<Self>) -> Div {
        let mut card = v_flex().w_full().min_w_0().gap_3();
        if let Some(e) = &self.environment {
            let speech_service = self.preferences.generation().options.provider
                == Some(course2md::config::AsrProvider::Api);
            for (index, (label, ready, optional)) in [
                ("视频处理与导出", e.engine && e.ffmpeg && e.ffprobe, false),
                ("读取在线视频", e.ytdlp, false),
                (
                    "本机语音识别",
                    e.engine && (e.apple || e.npu || e.llama),
                    speech_service,
                ),
            ]
            .into_iter()
            .enumerate()
            {
                card = card.child(settings_detail_row(
                    ("diagnostic-capability", index),
                    label,
                    badge(if ready {
                        BadgeKind::Success
                    } else if optional {
                        BadgeKind::Neutral
                    } else {
                        BadgeKind::Warning
                    })
                    .child(if ready {
                        if index == 2 {
                            "运行环境就绪"
                        } else {
                            "已就绪"
                        }
                    } else if optional {
                        "使用在线服务"
                    } else {
                        "需要设置"
                    }),
                ));
            }
            if !e.engine {
                card = card
                    .child(
                        settings_value(
                            "repair-bundled-engine",
                            "转换程序暂时无法运行。重新安装应用后可继续，已有笔记会保留。",
                        )
                        .text_size(TEXT_AUX),
                    )
                    .child(
                        control("download-repair-app")
                            .icon(icons::download())
                            .label("下载安装包")
                            .self_start()
                            .on_click(|_, _, cx| {
                                cx.open_url("https://github.com/mizorewww/course2md/releases")
                            }),
                    );
            }
            if !e.ffmpeg || !e.ffprobe || !e.ytdlp {
                card = card.child(
                    settings_value(
                        "media-tools-unavailable",
                        "部分媒体功能暂不可用。展开诊断详情可查看修复方式。",
                    )
                    .text_size(TEXT_AUX)
                    .text_color(color(MUTED)),
                );
            }
            if !speech_service && !(e.apple || e.npu || e.llama) {
                card = card.child(
                    settings_value(
                        "local-recognition-help",
                        "本机识别尚未就绪。展开详情查看安装方式，或在生成设置中选择在线语音服务。",
                    )
                    .text_size(TEXT_AUX)
                    .text_color(color(MUTED)),
                );
            }
        } else {
            card = card.child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(crate::motion::spinner("diagnostic-checking-spinner", cx))
                    .child(settings_value("diagnostic-checking", "正在检查本机能力…")),
            );
        }
        let open = self.settings_ui.diagnostics_details_open;
        card = card.child(
            h_flex()
                .gap_2()
                .flex_wrap()
                .child(
                    control("refresh-environment")
                        .icon(icons::refresh())
                        .label("重新检查")
                        .loading(self.environment.is_none())
                        .disabled(self.environment.is_none())
                        .on_click(cx.listener(|this, _, _, cx| this.refresh_environment(cx))),
                )
                .child(
                    quiet("toggle-diagnostics-details")
                        .icon(icons::info())
                        .label(if open {
                            "收起诊断详情"
                        } else {
                            "查看诊断详情"
                        })
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.settings_ui.diagnostics_details_open =
                                !this.settings_ui.diagnostics_details_open;
                            cx.notify();
                        })),
                ),
        );
        let mut details = v_flex().w_full().min_w_0().gap_6().pt_3();
        if let Some(e) = &self.environment {
            let mut programs = settings_detail_group("diagnostic-programs-heading", "所需程序");
            for (index, (name, found)) in [
                ("转换程序", e.engine),
                ("ffmpeg", e.ffmpeg),
                ("ffprobe", e.ffprobe),
                ("yt-dlp", e.ytdlp),
                ("llama-server", e.llama),
            ]
            .into_iter()
            .enumerate()
            {
                programs = programs.child(settings_detail_row(
                    ("diagnostic-tool-label", index),
                    name,
                    settings_value(
                        ("diagnostic-tool", index),
                        if found { "已安装" } else { "未安装" },
                    ),
                ));
            }
            if !e.ffmpeg || !e.ffprobe || !e.ytdlp {
                let (help, command) = if cfg!(target_os = "macos") {
                    (
                        "在终端运行以下命令，安装完成后重新检查。",
                        "brew install ffmpeg yt-dlp",
                    )
                } else if cfg!(target_os = "windows") {
                    (
                        "在 PowerShell 运行以下命令，安装完成后重新检查。",
                        "winget install Gyan.FFmpeg; winget install yt-dlp.yt-dlp",
                    )
                } else {
                    (
                        "使用系统软件包管理器安装 ffmpeg，并使用 pipx 安装下载工具，完成后重新检查。",
                        "pipx install yt-dlp",
                    )
                };
                programs = programs.child(settings_detail_row(
                    "media-tools-repair-label",
                    "修复方式",
                    v_flex()
                        .w_full()
                        .min_w_0()
                        .gap_2()
                        .child(settings_value("install-media-tools-help", help))
                        .child(settings_value("install-media-tools-command", command))
                        .child(
                            quiet("copy-media-install-command")
                                .icon(icons::content_copy())
                                .label("复制安装命令")
                                .self_start()
                                .on_click(move |_, _, cx| {
                                    cx.write_to_clipboard(ClipboardItem::new_string(command.into()))
                                }),
                        ),
                ));
            }
            details = details.child(programs);
        }
        details = details.child(self.model_diagnostics_panel(window, cx));
        card = card.child(crate::motion::disclosure(
            "diagnostics-detail-content",
            open,
            details,
            window,
            cx,
        ));
        group("diagnostics-heading", "运行检查").child(card)
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
                message: "生成选项暂时无法读取，原文件与当前输入已保留。".into(),
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
                    .unwrap_or_else(|| "生成选项的修改尚未保存，当前输入已保留。".into()),
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
                if !self.preference_defaults_pending {
                    self.advance_conversion_when_ready(cx);
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
                format!("模型位置尚未更新，修改已保留，可重试保存：{error:#}"),
                true,
            );
        } else {
            self.refresh_preference_defaults(cx);
            self.clear_resolved_generation_errors(previous);
        }
        cx.notify();
        result
    }
    /// Ordinary preferences save immediately. Service form edits become active
    /// only through Save and do not create a separate quit-time save workflow.
    pub fn flush_settings_for_exit(&mut self, cx: &mut Context<Self>) -> bool {
        if let Some(editor) = &self.settings_ui.editor {
            if let Some(cancel) = &editor.test_running {
                cancel.store(true, Ordering::Release);
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
