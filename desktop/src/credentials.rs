//! Secret storage is deliberately separate from serializable preferences and task records.
//!
//! Tests inject `MemoryCredentialVault`; they never access the user's Keychain.

use anyhow::{Result, anyhow, bail};
#[cfg(test)]
use std::collections::BTreeMap;
use std::fmt;
use std::sync::Arc;
#[cfg(test)]
use std::sync::Mutex;
use zeroize::Zeroizing;

pub type CredentialRef = String;

/// A secret cannot accidentally be serialized or printed by a derived Debug implementation.
pub struct Secret(Zeroizing<String>);

impl Secret {
    pub fn new(value: impl Into<String>) -> Self {
        Self(Zeroizing::new(value.into()))
    }

    pub fn expose(&self) -> &str {
        self.0.as_str()
    }

    pub fn is_empty(&self) -> bool {
        self.expose().trim().is_empty()
    }
}

impl fmt::Debug for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Secret([REDACTED])")
    }
}

/// Insert always creates a new immutable identity. Updating a service cannot change the
/// credential used by an already queued task. Resolving never consults mutable environment
/// variables; a user-selected environment credential must be captured at publication time.
pub trait CredentialVault: Send + Sync {
    fn insert(&self, secret: Secret) -> Result<CredentialRef>;
    fn resolve(&self, reference: &str) -> Result<Secret>;
    fn remove(&self, reference: &str) -> Result<()>;
}

fn new_reference() -> String {
    format!("credential-{}", uuid::Uuid::new_v4())
}

fn validate_reference(reference: &str) -> Result<()> {
    let id = reference.strip_prefix("credential-").unwrap_or("");
    if uuid::Uuid::parse_str(id).is_err() {
        bail!("凭据引用无效，请为此服务重新保存 API Key");
    }
    Ok(())
}

/// In-memory isolated store for tests and caller-controlled temporary use.
/// It intentionally does not provide a plaintext file fallback.
#[cfg(test)]
#[derive(Default)]
pub struct MemoryCredentialVault {
    values: Mutex<BTreeMap<CredentialRef, Zeroizing<String>>>,
}

#[cfg(test)]
impl MemoryCredentialVault {
    pub fn new() -> Self {
        Self::default()
    }
}

#[cfg(test)]
impl CredentialVault for MemoryCredentialVault {
    fn insert(&self, secret: Secret) -> Result<CredentialRef> {
        if secret.is_empty() {
            bail!("请输入 API Key");
        }
        let reference = new_reference();
        self.values
            .lock()
            .map_err(|_| anyhow!("暂时无法保存凭据"))?
            .insert(
                reference.clone(),
                Zeroizing::new(secret.expose().to_owned()),
            );
        Ok(reference)
    }

    fn resolve(&self, reference: &str) -> Result<Secret> {
        validate_reference(reference)?;
        self.values
            .lock()
            .map_err(|_| anyhow!("暂时无法读取凭据"))?
            .get(reference)
            .map(|value| Secret::new(value.as_str()))
            .ok_or_else(|| anyhow!("找不到此服务的凭据，请重新保存 API Key"))
    }

    fn remove(&self, reference: &str) -> Result<()> {
        validate_reference(reference)?;
        self.values
            .lock()
            .map_err(|_| anyhow!("暂时无法移除凭据"))?
            .remove(reference);
        Ok(())
    }
}

#[cfg(target_os = "macos")]
pub struct KeychainCredentialVault {
    service: String,
}

#[cfg(target_os = "macos")]
impl Default for KeychainCredentialVault {
    fn default() -> Self {
        Self {
            service: "com.course2md.desktop.service-credentials.v1".into(),
        }
    }
}

#[cfg(target_os = "macos")]
impl CredentialVault for KeychainCredentialVault {
    fn insert(&self, secret: Secret) -> Result<CredentialRef> {
        if secret.is_empty() {
            bail!("请输入 API Key");
        }
        let reference = new_reference();
        security_framework::passwords::set_generic_password(
            &self.service,
            &reference,
            secret.expose().as_bytes(),
        )
        .map_err(|error| anyhow!("无法将 API Key 保存到钥匙串（系统代码 {}）", error.code()))?;
        Ok(reference)
    }

    fn resolve(&self, reference: &str) -> Result<Secret> {
        validate_reference(reference)?;
        let bytes = security_framework::passwords::get_generic_password(&self.service, reference)
            .map_err(|error| match error.code() {
            -25300 => anyhow!("钥匙串中找不到此服务的凭据，请重新保存 API Key"),
            code => anyhow!("暂时无法读取此服务的钥匙串凭据（系统代码 {code}）"),
        })?;
        // Keep the intermediate byte buffer zeroizing as well as the returned UTF-8 value.
        let bytes = Zeroizing::new(bytes);
        let value = std::str::from_utf8(&bytes)
            .map_err(|_| anyhow!("此服务的凭据无法读取，请重新保存 API Key"))?;
        Ok(Secret::new(value))
    }

    fn remove(&self, reference: &str) -> Result<()> {
        validate_reference(reference)?;
        match security_framework::passwords::delete_generic_password(&self.service, reference) {
            Ok(()) => Ok(()),
            Err(error) if error.code() == -25300 => Ok(()),
            Err(error) => bail!("无法移除此服务的钥匙串凭据（系统代码 {}）", error.code()),
        }
    }
}

#[cfg(not(target_os = "macos"))]
struct UnavailableCredentialVault;

#[cfg(not(target_os = "macos"))]
impl CredentialVault for UnavailableCredentialVault {
    fn insert(&self, _secret: Secret) -> Result<CredentialRef> {
        bail!("此系统尚未提供安全凭据存储；可以使用无需认证的本机服务")
    }
    fn resolve(&self, _reference: &str) -> Result<Secret> {
        bail!("此系统尚未提供安全凭据存储")
    }
    fn remove(&self, _reference: &str) -> Result<()> {
        bail!("此系统尚未提供安全凭据存储")
    }
}

pub fn system_vault() -> Arc<dyn CredentialVault> {
    #[cfg(target_os = "macos")]
    {
        Arc::new(KeychainCredentialVault::default())
    }
    #[cfg(not(target_os = "macos"))]
    {
        Arc::new(UnavailableCredentialVault)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn credential_rotation_keeps_old_task_reference_immutable() {
        let vault = MemoryCredentialVault::new();
        let original = vault.insert(Secret::new("test-only-old-key")).unwrap();
        let replacement = vault.insert(Secret::new("test-only-new-key")).unwrap();
        assert_ne!(original, replacement);
        assert_eq!(
            vault.resolve(&original).unwrap().expose(),
            "test-only-old-key"
        );
        assert_eq!(
            vault.resolve(&replacement).unwrap().expose(),
            "test-only-new-key"
        );
        vault.remove(&replacement).unwrap();
        assert!(vault.resolve(&replacement).is_err());
        assert!(vault.resolve(&original).is_ok());
    }

    #[test]
    fn debug_and_errors_cannot_expose_secret() {
        let secret = Secret::new("test-only-do-not-print");
        assert_eq!(format!("{secret:?}"), "Secret([REDACTED])");
        let vault = MemoryCredentialVault::new();
        let error = vault.resolve("test-only-do-not-print").unwrap_err();
        assert!(!error.to_string().contains("test-only-do-not-print"));
        assert!(vault.insert(Secret::new("   ")).is_err());
    }
}
