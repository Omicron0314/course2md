//! Bilibili account settings and cancellable QR login. No ticket is logged or stored in UI text.
use super::*;
use crate::theme::*;
use course2md::auth::{AccountProfile, AccountStatus, QrPoll, QrSession};
use gpui_component::button::*;
use std::sync::Arc;

#[derive(Default)]
pub(crate) struct AccountUi {
    generation: u64,
    status_generation: u64,
    dialog: Option<QrDialogState>,
    modules: Option<Arc<Vec<Vec<bool>>>>,
    status: Option<AccountStatus>,
    checking: bool,
    has_saved_login: bool,
    status_error: Option<String>,
    retry_source: Option<(u64, String)>,
    return_focus: Option<FocusHandle>,
}
#[derive(Clone)]
enum QrDialogState {
    Generating,
    Waiting(u64),
    Confirming(u64),
    Expired,
    Error(String),
    Success(AccountProfile),
}
impl AccountUi {
    fn current(&self, generation: u64) -> bool {
        self.generation == generation && self.dialog.is_some()
    }
    fn invalidate(&mut self) {
        self.generation = self.generation.wrapping_add(1);
        self.dialog = None;
        self.modules = None;
    }
}

struct AccountDialog {
    desktop: Entity<Desktop>,
    _observation: Subscription,
}
impl Render for AccountDialog {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.desktop
            .update(cx, |desktop, cx| desktop.account_dialog_content(window, cx))
    }
}

impl Desktop {
    pub fn refresh_account(&mut self, cx: &mut Context<Self>) {
        self.account.status_generation = self.account.status_generation.wrapping_add(1);
        let generation = self.account.status_generation;
        self.account.has_saved_login = course2md::auth::cookie_path().is_file();
        self.account.checking = true;
        self.account.status_error = None;
        let task = cx
            .background_executor()
            .spawn(async { course2md::auth::bilibili_account_status() });
        cx.spawn(async move |this, cx| {
            let result = task.await;
            let _ = this.update(cx, |this, cx| {
                if this.account.status_generation != generation {
                    return;
                }
                this.account.checking = false;
                match result {
                    Ok(status) => this.account.status = Some(status),
                    Err(error) => this.account.status_error = Some(error.to_string()),
                }
                cx.notify();
            });
        })
        .detach();
        cx.notify();
    }

    pub fn account_settings_page(&self, cx: &mut Context<Self>) -> AnyElement {
        let status = self.account_status_text();
        let saved = self.account.has_saved_login;
        let expired = matches!(self.account.status, Some(AccountStatus::Expired));
        let show_login = !saved || expired;
        let connected = matches!(self.account.status, Some(AccountStatus::Connected(_)));
        let (kind, label) = if self.account.checking {
            (BadgeKind::Progress, "验证中")
        } else if self.account.status_error.is_some() {
            (BadgeKind::Warning, "暂时无法验证")
        } else if expired {
            (BadgeKind::Warning, "登录已失效")
        } else if connected {
            (BadgeKind::Success, "已登录")
        } else if saved {
            (BadgeKind::Neutral, "已保留登录")
        } else {
            (BadgeKind::Neutral, "未登录")
        };
        let detail = status
            .trim_start_matches("Bilibili：")
            .trim_start_matches("已登录 · ")
            .to_owned();
        let show_detail = !self.account.checking && detail != label;
        v_flex()
            .w_full()
            .gap_3()
            .child(
                h_flex()
                    .w_full()
                    .min_w_0()
                    .gap_2()
                    .items_start()
                    .when(self.account.checking, |row| {
                        row.child(crate::motion::spinner("account-checking", cx))
                    })
                    .child(badge(kind).child(label))
                    .when(show_detail, |row| {
                        row.child(
                            accessible_text("bilibili-account-status", detail)
                                .flex_1()
                                .min_w_0()
                                .whitespace_normal()
                                .py(px(3.)),
                        )
                    }),
            )
            .child(accessible_text("bilibili-account-policy", if saved {
                "获取字幕和视频将使用此账号的访问权限。退出登录后停止后续使用，课程和笔记保留。"
            } else {
                "公开课程可以直接读取；遇到账号权限限制时，再登录继续。"
            }).text_sm().text_color(color(MUTED)))
            .child(
                h_flex()
                    .gap_2()
                    .flex_wrap()
                    .child(
                        control("account-refresh")
                            .ghost()
                            .icon(icons::refresh()).label("重新检查")
                            .disabled(self.account.checking)
                            .on_click(cx.listener(|this, _, _, cx| this.refresh_account(cx))),
                    )
                    .when(show_login, |view| {
                        view.child(
                            control("account-login")
                                .icon(icons::login()).primary().label(if expired {
                                    "重新登录 Bilibili"
                                } else {
                                    "登录 Bilibili"
                                })
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.open_account_dialog(window, cx)
                                })),
                        )
                    })
                    .when(saved, |view| {
                        view.child(
                            control("account-logout")
                                .ghost()
                                .icon(icons::logout()).label("退出登录")
                                .on_click(cx.listener(|this, _, _, cx| this.clear_account(cx))),
                        )
                    }),
            )
            .into_any_element()
    }

    fn account_status_text(&self) -> String {
        if self.account.checking {
            return "Bilibili：正在验证登录状态…".into();
        }
        if self.account.status_error.as_deref() == Some("退出登录尚未完成，原登录状态已保留。")
        {
            return "Bilibili：退出登录尚未完成，原登录状态已保留".into();
        }
        if self.account.status_error.is_some() {
            return if self.account.has_saved_login {
                "Bilibili：暂时无法验证，已保留登录".into()
            } else {
                "Bilibili：暂时无法检查登录状态".into()
            };
        }
        match &self.account.status {
            Some(AccountStatus::Connected(profile)) if !profile.name.trim().is_empty() => {
                format!("Bilibili：已登录 · {}", profile.name)
            }
            Some(AccountStatus::Connected(_)) => "Bilibili：已登录".into(),
            Some(AccountStatus::Expired) => "Bilibili：登录已失效".into(),
            Some(AccountStatus::Disconnected) => "Bilibili：未登录".into(),
            None if self.account.has_saved_login => "Bilibili：已保留登录，尚未验证".into(),
            None => "Bilibili：未登录".into(),
        }
    }

    /// Only rendered inside source details or a permission repair, never a permanent status row.
    pub fn source_account_row(&self, cx: &mut Context<Self>) -> Div {
        let status = self.account_status_text();
        let temporary = self.account.status_error.is_some() || self.account.checking;
        let connected = matches!(self.account.status, Some(AccountStatus::Connected(_)));
        let expired = matches!(self.account.status, Some(AccountStatus::Expired));
        h_flex()
            .w_full()
            .items_center()
            .gap_3()
            .child(
                div()
                    .id("source-bilibili-account-status")
                    .role(Role::Label)
                    .aria_label(status.clone())
                    .child(status)
                    .flex_1()
                    .min_w_0()
                    .text_sm()
                    .text_color(color(MUTED)),
            )
            .child(
                control("source-account-refresh")
                    .ghost()
                    .icon(icons::refresh())
                    .label("重新检查")
                    .disabled(self.account.checking)
                    .on_click(cx.listener(|this, _, _, cx| this.refresh_account(cx))),
            )
            .when(!temporary && !connected, |view| {
                view.child(
                    control("source-account-login")
                        .icon(icons::login())
                        .ghost()
                        .label(if expired {
                            "重新登录 Bilibili"
                        } else {
                            "登录 Bilibili"
                        })
                        .on_click(
                            cx.listener(|this, _, window, cx| this.open_account_dialog(window, cx)),
                        ),
                )
            })
    }

    fn can_retry_account_source(&self, cx: &App) -> bool {
        self.account
            .retry_source
            .as_ref()
            .is_some_and(|(generation, input)| {
                *generation == self.preview_generation
                    && *input == self.value(Field::Source, cx)
                    && matches!(self.page, Page::New)
                    && self.online
                    && self.preview_error.is_some()
            })
    }

    fn clear_account(&mut self, cx: &mut Context<Self>) {
        self.account.invalidate();
        self.account.status_generation = self.account.status_generation.wrapping_add(1);
        self.account.checking = false;
        match course2md::auth::clear_bilibili_login() {
            Ok(()) => {
                self.account.has_saved_login = false;
                self.account.status = Some(AccountStatus::Disconnected);
                self.account.status_error = None;
            }
            Err(_) => {
                self.account.status_error = Some("退出登录尚未完成，原登录状态已保留。".into())
            }
        }
        cx.notify();
    }

    pub fn close_account_dialog(&mut self, cx: &mut Context<Self>) {
        self.account.invalidate();
        self.account.retry_source = None;
        cx.notify();
    }

    pub fn open_account_dialog(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.account.dialog.is_some() {
            return;
        }
        self.account.retry_source = if matches!(self.page, Page::New)
            && self.online
            && self.preview_error.is_some()
            && course2md::auth::is_bilibili_url(&self.value(Field::Source, cx))
        {
            Some((self.preview_generation, self.value(Field::Source, cx)))
        } else {
            None
        };
        self.account.return_focus = window.focused(cx);
        self.start_account_qr(cx);
        let desktop = cx.entity();
        let content = cx.new(|cx| AccountDialog {
            _observation: cx.observe(&desktop, |_, _, cx| cx.notify()),
            desktop,
        });
        let weak = cx.weak_entity();
        window.open_dialog(cx, move |dialog, _, _| {
            let weak = weak.clone();
            let closed = weak.clone();
            dialog
                .title("登录 Bilibili")
                .w(px(420.))
                .overlay_closable(false)
                .child(content.clone())
                .on_close(move |_, window, cx| {
                    let _ = closed.update(cx, |this, cx| {
                        this.close_account_dialog(cx);
                        if let Some(focus) = this.account.return_focus.take() {
                            focus.focus(window, cx);
                        }
                    });
                })
                .on_cancel(move |_, _, cx| {
                    let _ = weak.update(cx, |this, cx| this.close_account_dialog(cx));
                    true
                })
        });
    }

    fn start_account_qr(&mut self, cx: &mut Context<Self>) {
        self.account.invalidate();
        self.account.dialog = Some(QrDialogState::Generating);
        let generation = self.account.generation;
        let task = cx
            .background_executor()
            .spawn(async { QrSession::generate() });
        cx.spawn(async move |this, cx| {
            let result = task.await;
            let mut session = match result {
                Ok(session) => session,
                Err(_) => {
                    let _ = this.update(cx, |this, cx| {
                        if this.account.current(generation) {
                            this.account.dialog = Some(QrDialogState::Error("获取二维码失败，请检查网络后重试。".into()));
                            cx.notify();
                        }
                    });
                    return;
                }
            };
            let modules = Arc::new(session.modules());
            let active = this.update(cx, |this, cx| {
                if !this.account.current(generation) { return false; }
                this.account.modules = Some(modules);
                this.account.dialog = Some(QrDialogState::Waiting(session.remaining().as_secs()));
                cx.notify();
                true
            }).unwrap_or(false);
            if !active { return; }
            loop {
                smol::Timer::after(Duration::from_secs(2)).await;
                if !this.update(cx, |this, _| this.account.current(generation)).unwrap_or(false) { break; }
                let task = cx.background_executor().spawn(async move {
                    let result = session.poll();
                    (session, result)
                });
                let (next, result) = task.await;
                session = next;
                let remaining = session.remaining().as_secs();
                let keep_polling = this.update(cx, |this, cx| {
                    if !this.account.current(generation) { return false; }
                    let (state, keep_polling) = match result {
                        Ok(QrPoll::Waiting) => (QrDialogState::Waiting(remaining), true),
                        Ok(QrPoll::AwaitingConfirmation) => (QrDialogState::Confirming(remaining), true),
                        Ok(QrPoll::Expired) => (QrDialogState::Expired, false),
                        Ok(QrPoll::Authenticated(login)) => {
                            // The generation check and atomic credential commit happen together on
                            // the UI thread. A cancelled request can never save a late result.
                            match login.save() {
                                Ok(profile) => {
                                    this.account.status_generation = this.account.status_generation.wrapping_add(1);
                                    this.account.checking = false;
                                    this.account.has_saved_login = true;
                                    this.account.status = Some(AccountStatus::Connected(profile.clone()));
                                    this.account.status_error = None;
                                    (QrDialogState::Success(profile), false)
                                }
                                Err(_) => (QrDialogState::Error("账号已确认，但保存登录状态失败。请检查本地权限后重新扫码。".into()), false),
                            }
                        }
                        Err(_) => (QrDialogState::Error("登录请求失败，请检查网络后重新获取二维码。".into()), false),
                    };
                    if !keep_polling { this.account.modules = None; }
                    this.account.dialog = Some(state);
                    cx.notify();
                    keep_polling
                }).unwrap_or(false);
                if !keep_polling { break; }
            }
        }).detach();
        cx.notify();
    }

    fn account_dialog_content(&mut self, window: &Window, cx: &mut Context<Self>) -> AnyElement {
        let state = self
            .account
            .dialog
            .clone()
            .unwrap_or(QrDialogState::Generating);
        let (message, hint): (String, String) = match &state {
            QrDialogState::Generating => ("正在获取二维码".into(), String::new()),
            QrDialogState::Waiting(seconds) => (
                "请使用哔哩哔哩 App 扫码".into(),
                format!("二维码约 {seconds} 秒后过期"),
            ),
            QrDialogState::Confirming(seconds) => (
                "已扫码，请在手机上确认登录".into(),
                format!("请在约 {seconds} 秒内确认"),
            ),
            QrDialogState::Expired => ("二维码已过期".into(), "请刷新二维码后重新扫码。".into()),
            QrDialogState::Error(error) => ("登录未完成".into(), error.clone()),
            QrDialogState::Success(profile) => (
                "已登录 Bilibili".into(),
                if profile.name.trim().is_empty() {
                    "登录状态已保存。".into()
                } else {
                    format!("{}，登录状态已保存。", profile.name)
                },
            ),
        };
        let retry = matches!(state, QrDialogState::Expired | QrDialogState::Error(_));
        let success = matches!(state, QrDialogState::Success(_));
        let retry_source = success && self.can_retry_account_source(cx);
        let mut visual = v_flex()
            .w(px(260.))
            .h(px(260.))
            .flex_shrink_0()
            .items_center()
            .justify_center()
            .rounded(RADIUS_CARD)
            .bg(color(if success {
                SUCCESS_BG
            } else if retry {
                WARNING_BG
            } else {
                INSET
            }));
        if let Some(modules) = &self.account.modules {
            let modules = modules.clone();
            // A real QR code intentionally keeps its high-contrast white scanning surface.
            visual = visual.bg(gpui::rgb(0xffffff)).child(
                canvas(
                    |_, _, _| {},
                    move |bounds, _, window, _| {
                        // Whole physical-pixel modules and a >=4-module quiet zone keep QR edges crisp.
                        let scale = window.scale_factor();
                        let unit = ((bounds.size.width.as_f32() * scale)
                            / (modules.len() + 8) as f32)
                            .floor()
                            / scale;
                        let extent = unit * (modules.len() + 8) as f32;
                        let offset = (bounds.size.width.as_f32() - extent) / 2. + unit * 4.;
                        for (y, row) in modules.iter().enumerate() {
                            for (x, dark) in row.iter().enumerate() {
                                if *dark {
                                    window.paint_quad(fill(
                                        Bounds::new(
                                            point(
                                                px(((bounds.origin.x.as_f32()
                                                    + offset
                                                    + x as f32 * unit)
                                                    * scale)
                                                    .round()
                                                    / scale),
                                                px(((bounds.origin.y.as_f32()
                                                    + offset
                                                    + y as f32 * unit)
                                                    * scale)
                                                    .round()
                                                    / scale),
                                            ),
                                            size(px(unit), px(unit)),
                                        ),
                                        gpui::rgb(0x000000),
                                    ));
                                }
                            }
                        }
                    },
                )
                .w_full()
                .h_full(),
            );
        } else if success || retry {
            visual = visual.child(
                if success {
                    icons::check_circle()
                } else {
                    icons::warning()
                }
                .size_8()
                .text_color(color(if success { SUCCESS } else { WARNING })),
            );
        } else {
            visual = visual
                .child(crate::motion::spinner("qr-code-generating", cx))
                .child(
                    accessible_text("qr-preparing-label", "正在获取二维码…")
                        .text_sm()
                        .text_color(color(MUTED))
                        .mt_3(),
                );
        }
        let phase = match state {
            QrDialogState::Generating => 0,
            QrDialogState::Waiting(_) => 1,
            QrDialogState::Confirming(_) => 2,
            QrDialogState::Expired => 3,
            QrDialogState::Error(_) => 4,
            QrDialogState::Success(_) => 5,
        };
        v_flex()
            .id("account-login-body")
            .max_h((window.bounds().size.height - px(150.)).max(px(180.)))
            .overflow_y_scroll()
            .gap_4()
            .items_center()
            .child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(icons::bilibili().size_6())
                    .child(
                        accessible_text("bilibili-login-brand", "Bilibili 账号")
                            .font_weight(FontWeight::SEMIBOLD),
                    ),
            )
            .child(crate::motion::enter(
                SharedString::from(format!("qr-visual-{}-{phase}", self.account.generation)),
                visual,
                cx,
            ))
            .child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .when(matches!(state, QrDialogState::Confirming(_)), |row| {
                        row.child(crate::motion::spinner("qr-awaiting-confirmation", cx))
                    })
                    .child(
                        accessible_text("bilibili-login-step", message)
                            .font_weight(FontWeight::SEMIBOLD),
                    ),
            )
            .when(!hint.is_empty(), |view| {
                view.child(
                    accessible_text("bilibili-login-detail", hint)
                        .text_sm()
                        .text_color(color(MUTED)),
                )
            })
            .child(
                h_flex()
                    .w_full()
                    .justify_end()
                    .gap_3()
                    .flex_wrap()
                    .when(retry, |view| {
                        view.child(
                            control("account-qr-retry")
                                .icon(icons::refresh())
                                .primary()
                                .label("刷新二维码")
                                .on_click(cx.listener(|this, _, _, cx| this.start_account_qr(cx))),
                        )
                    })
                    .when(retry_source, |view| {
                        view.child(
                            control("account-qr-source-retry")
                                .primary()
                                .icon(icons::refresh())
                                .label("重新读取课程")
                                .on_click(cx.listener(|this, _, window, cx| {
                                    let retry = this.can_retry_account_source(cx);
                                    this.close_account_dialog(cx);
                                    window.close_dialog(cx);
                                    if retry {
                                        this.inspect_source(cx);
                                    }
                                })),
                        )
                    })
                    .child(
                        control("account-qr-close")
                            .icon(if success {
                                icons::check_circle()
                            } else {
                                icons::close()
                            })
                            .label(if success { "完成" } else { "取消" })
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.close_account_dialog(cx);
                                window.close_dialog(cx);
                            })),
                    ),
            )
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::{AccountUi, QrDialogState};
    #[test]
    fn closing_and_refreshing_reject_late_qr_results() {
        let mut account = AccountUi {
            dialog: Some(QrDialogState::Generating),
            ..Default::default()
        };
        let first = account.generation;
        assert!(account.current(first));
        account.invalidate();
        assert!(!account.current(first));
        account.dialog = Some(QrDialogState::Generating);
        assert!(!account.current(first));
        assert!(account.current(account.generation));
    }
}
