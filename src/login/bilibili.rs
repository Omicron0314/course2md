use anyhow::Result;

use super::{LoginMethod, LoginStatus};

pub(super) struct BilibiliLogin;
impl LoginMethod for BilibiliLogin {
    fn id(&self) -> &'static str {
        "bilibili"
    }
    fn label(&self) -> &'static str {
        "Bilibili 扫码 / Bilibili QR"
    }
    fn login(&self) -> Result<()> {
        crate::auth::login_bilibili()
    }
    fn logout(&self) -> Result<()> {
        crate::auth::logout_bilibili()
    }
    fn status(&self) -> Result<LoginStatus> {
        Ok(match crate::auth::bilibili_account_status()? {
            crate::auth::AccountStatus::Disconnected => LoginStatus::Disconnected,
            crate::auth::AccountStatus::Connected(profile) => LoginStatus::Connected(profile.name),
            crate::auth::AccountStatus::Expired => LoginStatus::Expired,
        })
    }
}
