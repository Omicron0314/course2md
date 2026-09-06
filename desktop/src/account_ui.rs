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
    status_error: Option<String>,
    retry_source: Option<(u64, String)>,
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
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.desktop
            .update(cx, |desktop, cx| desktop.account_dialog_content(cx))
    }
}

impl Desktop {
    pub fn refresh_account(&mut self, cx: &mut Context<Self>) {
        self.account.status_generation = self.account.status_generation.wrapping_add(1);
        let generation = self.account.status_generation;
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

    pub fn account_settings_page(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let status = if self.account.checking {
            "正在验证登录状态…".to_owned()
        } else if self.account.status_error.is_some() {
            "暂时无法验证".to_owned()
        } else {
            match &self.account.status {
                Some(AccountStatus::Connected(profile)) => format!("已连接 · {}", profile.name),
                Some(AccountStatus::Expired) => "登录已失效".into(),
                Some(AccountStatus::Disconnected) => "未连接".into(),
                None => "尚未检查登录状态".into(),
            }
        };
        let connected = matches!(self.account.status, Some(AccountStatus::Connected(_)));
        let saved = matches!(
            self.account.status,
            Some(AccountStatus::Connected(_) | AccountStatus::Expired)
        );
        v_flex()
            .w_full()
            .max_w(px(760.))
            .gap_4()
            .child(
                div()
                    .text_size(px(18.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("连接账号"),
            )
            .child(
                v_flex()
                    .w_full()
                    .gap_4()
                    .p_4()
                    .bg(rgb(SURFACE))
                    .border_1()
                    .border_color(rgb(LINE))
                    .rounded_lg()
                    .child(
                        h_flex()
                            .items_center()
                            .gap_3()
                            .child(
                                Icon::new(if connected {
                                    IconName::CircleCheck
                                } else {
                                    IconName::Info
                                })
                                .text_color(rgb(if connected {
                                    SUCCESS
                                } else {
                                    MUTED
                                })),
                            )
                            .child(
                                v_flex()
                                    .flex_1()
                                    .min_w_0()
                                    .gap_1()
                                    .child(
                                        div().font_weight(FontWeight::SEMIBOLD).child("Bilibili"),
                                    )
                                    .child(div().text_color(rgb(MUTED)).child(status)),
                            )
                            .child(
                                control("account-refresh")
                                    .ghost()
                                    .icon(icons::refresh())
                                    .tooltip("检查账号状态")
                                    .accessibility_label("检查账号状态")
                                    .disabled(self.account.checking)
                                    .on_click(
                                        cx.listener(|this, _, _, cx| this.refresh_account(cx)),
                                    ),
                            ),
                    )
                    .when_some(self.account.status_error.clone(), |view, error| {
                        view.child(div().text_sm().text_color(rgb(MUTED)).child(error))
                    })
                    .child(div().text_sm().text_color(rgb(MUTED)).child(if saved {
                        "获取字幕和视频将使用此账号的访问权限。清除登录不会删除课程或笔记。"
                    } else {
                        "公开课程可直接添加；需要账号权限时，再登录继续。"
                    }))
                    .child(
                        h_flex()
                            .gap_3()
                            .flex_wrap()
                            .child(
                                control("account-login")
                                    .primary()
                                    .label(if connected {
                                        "重新登录"
                                    } else {
                                        "登录 Bilibili"
                                    })
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.open_account_dialog(window, cx)
                                    })),
                            )
                            .when(saved, |view| {
                                view.child(
                                    control("account-logout")
                                        .ghost()
                                        .label("清除本地登录")
                                        .on_click(
                                            cx.listener(|this, _, _, cx| this.clear_account(cx)),
                                        ),
                                )
                            }),
                    ),
            )
            .into_any_element()
    }

    /// Compact account status for the Bilibili source row; it never triggers work from render.
    pub fn source_account_row(&self, cx: &mut Context<Self>) -> Div {
        let (status, label) = if self.account.checking {
            ("正在验证账号".to_owned(), "登录 Bilibili")
        } else if self.account.status_error.is_some() {
            ("账号暂时无法验证".to_owned(), "重新登录")
        } else {
            match &self.account.status {
                Some(AccountStatus::Connected(profile)) => {
                    (format!("已连接 · {}", profile.name), "重新登录")
                }
                Some(AccountStatus::Expired) => ("登录已失效".into(), "重新登录"),
                Some(AccountStatus::Disconnected) => ("未连接 Bilibili".into(), "登录 Bilibili"),
                None => ("Bilibili 账号尚未检查".into(), "登录 Bilibili"),
            }
        };
        h_flex()
            .w_full()
            .items_center()
            .gap_3()
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .text_sm()
                    .text_color(rgb(MUTED))
                    .child(status),
            )
            .child(
                control("source-account-refresh")
                    .ghost()
                    .icon(icons::refresh())
                    .tooltip("检查账号状态")
                    .accessibility_label("检查账号状态")
                    .disabled(self.account.checking)
                    .on_click(cx.listener(|this, _, _, cx| this.refresh_account(cx))),
            )
            .child(
                control("source-account-login")
                    .ghost()
                    .label(label)
                    .on_click(
                        cx.listener(|this, _, window, cx| this.open_account_dialog(window, cx)),
                    ),
            )
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
                self.account.status = Some(AccountStatus::Disconnected);
                self.account.status_error = None;
            }
            Err(_) => {
                self.account.status_error =
                    Some("清除本地登录失败，请检查文件访问权限后重试。".into())
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
                .on_close(move |_, _, cx| {
                    let _ = closed.update(cx, |this, cx| this.close_account_dialog(cx));
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

    fn account_dialog_content(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let state = self
            .account
            .dialog
            .clone()
            .unwrap_or(QrDialogState::Generating);
        let (message, hint): (String, String) = match &state {
            QrDialogState::Generating => ("正在获取二维码".into(), "请稍候…".into()),
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
                "已连接 Bilibili".into(),
                format!("{}，登录状态已保存。", profile.name),
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
            .bg(rgb(0xffffff));
        if let Some(modules) = &self.account.modules {
            let modules = modules.clone();
            visual = visual.child(
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
                                        rgb(0x000000),
                                    ));
                                }
                            }
                        }
                    },
                )
                .w_full()
                .h_full(),
            );
        } else {
            visual = visual.child(
                Icon::new(if success {
                    IconName::CircleCheck
                } else if retry {
                    IconName::TriangleAlert
                } else {
                    IconName::LoaderCircle
                })
                .size_8()
                .text_color(rgb(if success { SUCCESS } else { MUTED })),
            );
        }
        v_flex()
            .gap_4()
            .items_center()
            .child(visual)
            .child(div().font_weight(FontWeight::SEMIBOLD).child(message))
            .child(div().text_sm().text_color(rgb(MUTED)).child(hint))
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(MUTED))
                    .child("请仅在本人手机上确认。关闭窗口会取消本次等待。"),
            )
            .child(
                h_flex()
                    .w_full()
                    .justify_end()
                    .gap_3()
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
