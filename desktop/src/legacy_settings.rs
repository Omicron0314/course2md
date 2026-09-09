//! Read-only discovery and explicit, recoverable repair of the retired TOML configuration.
//! Repair never publishes desktop preferences, credentials, or library registrations.
use anyhow::{Context, Result, anyhow, bail};
use course2md::settings::ConfigFile;
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use zeroize::Zeroizing;

#[derive(Clone)]
pub struct Problem {
    pub message: String,
    pub detail: String,
}

#[derive(Clone)]
pub struct Inspection {
    pub path: PathBuf,
    pub problem: Option<Problem>,
    pub backup_available: bool,
    pub backup_detail: Option<String>,
    pub importable: bool,
    pub output: Option<PathBuf>,
}

impl Inspection {
    pub fn inspect(path: PathBuf) -> Self {
        let mut value = Self {
            path,
            problem: None,
            backup_available: false,
            backup_detail: None,
            importable: false,
            output: None,
        };
        match read(&value.path) {
            Ok(None) => return value,
            Ok(Some((_, config))) => {
                value.importable = crate::preferences::has_importable_legacy(&config);
                value.output = config.defaults.out;
            }
            Err(error) => {
                value.problem = Some(Problem {
                    message: "旧版配置暂时无法读取，当前设置和课程库继续使用。".into(),
                    detail: format!("旧配置：{}\n{error:#}", value.path.display()),
                });
            }
        }
        if value.problem.is_some() {
            let backup = value.path.with_extension("toml.bak");
            match read(&backup) {
                Ok(Some(_)) => {
                    value.backup_available = true;
                    value.backup_detail = Some(format!("已验证旧配置备份：{}", backup.display()));
                }
                Ok(None) => {
                    value.backup_detail = Some(format!("未找到旧配置备份：{}", backup.display()));
                }
                Err(error) => {
                    value.backup_detail = Some(format!("备份暂时不可用：{}\n{error:#}", backup.display()));
                }
            }
        }
        value
    }
}

/// A persisted modern record is authoritative even when it needs its own recovery. Merely
/// creating an empty preferences directory does not count as a completed legacy migration.
pub fn has_modern_records(directory: &Path) -> bool {
    [
        "desktop-workspace.json",
        "desktop-workspace.json.bak",
        "desktop-preferences/generation.json",
        "desktop-preferences/generation.json.bak",
        "desktop-preferences/application.json",
        "desktop-preferences/application.json.bak",
        "desktop-preferences/services.json",
        "desktop-preferences/services.json.bak",
    ]
    .iter()
    .any(|name| directory.join(name).exists())
}

/// This only chooses a bootstrap root. Workspace::open subsequently uses its persisted
/// registry. Fresh installs use managed local storage so the welcome screen does not
/// synchronously access a protected Documents folder before the user can choose a location.
pub fn startup_output(
    directory: &Path,
    legacy: &Inspection,
) -> PathBuf {
    if has_modern_records(directory) || legacy.problem.is_some() {
        return directory.join("desktop-local-library");
    }
    legacy
        .output
        .clone()
        .map(course2md::config::expand_tilde)
        .unwrap_or_else(|| directory.join("desktop-local-library"))
}

fn parse(bytes: &[u8]) -> Result<ConfigFile> {
    let source = std::str::from_utf8(bytes).map_err(|_| anyhow!("文件不是有效的 UTF-8 文本"))?;
    let config: ConfigFile = toml::from_str(source).map_err(|error: toml::de::Error| {
        // Displaying a TOML parser error can reproduce an API key from the offending line.
        // Keep location and cause class, never the source excerpt or untrusted field value.
        let location = error.span().map(|range| {
            let before = &source.as_bytes()[..range.start.min(source.len())];
            let line = before.iter().filter(|byte| **byte == b'\n').count() + 1;
            let column = before.iter().rposition(|byte| *byte == b'\n')
                .map_or(before.len() + 1, |last| before.len() - last);
            format!("（第 {line} 行，第 {column} 列）")
        }).unwrap_or_default();
        anyhow!("TOML 格式或配置选项无法识别{location}；未显示原文，以免暴露凭据")
    })?;
    crate::preferences::validate_legacy(&config)?;
    Ok(config)
}

fn read(path: &Path) -> Result<Option<(Zeroizing<Vec<u8>>, ConfigFile)>> {
    let bytes = match std::fs::read(path) {
        Ok(bytes) => Zeroizing::new(bytes),
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error).context("文件暂时无法读取"),
    };
    let config = parse(&bytes)?;
    Ok(Some((bytes, config)))
}

pub struct PreservedImport {
    pub config: ConfigFile,
    pub original: PathBuf,
}

/// The UI calls this before the independent preference transactions. Existing TOML and its
/// .bak remain byte-for-byte intact, including on an interrupted/failed group import.
pub fn preserve_for_import(path: &Path) -> Result<PreservedImport> {
    let (bytes, config) = read(path)?.ok_or_else(|| anyhow!("旧配置文件已经不在原位置，请重新检查"))?;
    let original = preserve(path, &bytes)?;
    ensure_unchanged(path, &bytes)?;
    Ok(PreservedImport { config, original })
}

/// Shared UI entry point: no preference transaction may precede the verified original copy.
pub fn import_preferences(
    path: &Path,
    preferences: &mut crate::preferences::Store,
) -> Result<PathBuf> {
    let preserved = preserve_for_import(path)?;
    preferences.import_legacy(&preserved.config)?;
    Ok(preserved.original)
}

pub fn restore_backup(path: &Path) -> Result<PathBuf> {
    let backup = path.with_extension("toml.bak");
    let (replacement, _) = read(&backup)?
        .ok_or_else(|| anyhow!("没有可恢复的旧配置备份，原文件保持不变"))?;
    repair(path, &replacement)
}

pub fn reset_preserving_original(path: &Path) -> Result<PathBuf> {
    // An empty valid configuration has no old service, provider, model, or output selection.
    // New desktop defaults and its library registry are deliberately not read or written.
    repair(path, b"# Old configuration reset; desktop preferences are stored independently.\n")
}

fn repair(path: &Path, replacement: &[u8]) -> Result<PathBuf> {
    parse(replacement)?;
    let metadata = std::fs::symlink_metadata(path).context("无法确认旧配置原文件")?;
    if !metadata.file_type().is_file() {
        bail!("旧配置路径不是普通文件，尚未替换。可在文件管理器中查看该路径后重新检查。");
    }
    let original = Zeroizing::new(std::fs::read(path).context("无法读取并保留原文件，尚未替换")?);
    if parse(&original).is_ok() {
        bail!("旧配置现在已经可以读取，尚未替换。请重新检查后使用导入入口。");
    }
    let preserved = preserve(path, &original)?;
    publish(path, &original, replacement)?;
    Ok(preserved)
}

fn preserve(path: &Path, bytes: &[u8]) -> Result<PathBuf> {
    let directory = path.parent().ok_or_else(|| anyhow!("旧配置没有可用的上级目录"))?
        .join("legacy-config-recovery");
    if !directory.exists() {
        let mut builder = std::fs::DirBuilder::new();
        builder.recursive(true);
        #[cfg(unix)] {
            use std::os::unix::fs::DirBuilderExt;
            builder.mode(0o700);
        }
        builder.create(&directory).context("无法创建原配置保留目录，尚未替换")?;
    }
    let saved = directory.join(format!("config-{}.original.toml", uuid::Uuid::new_v4()));
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)] {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(&saved).context("无法保留原配置，尚未替换")?;
    file.write_all(bytes).context("原配置保留副本未写完，尚未替换")?;
    file.sync_all().context("原配置保留副本尚未落盘，尚未替换")?;
    if Zeroizing::new(std::fs::read(&saved)?) .as_slice() != bytes {
        bail!("原配置保留副本未通过校验，尚未替换");
    }
    sync_directory(&directory)?;
    Ok(saved)
}

fn ensure_unchanged(path: &Path, expected: &[u8]) -> Result<()> {
    let current = Zeroizing::new(std::fs::read(path).context("旧配置原文件暂时无法读取，尚未替换")?);
    if current.as_slice() != expected {
        bail!("旧配置已被其他程序修改，尚未替换。请重新检查后重试。");
    }
    Ok(())
}

fn publish(path: &Path, expected: &[u8], replacement: &[u8]) -> Result<()> {
    let directory = path.parent().ok_or_else(|| anyhow!("旧配置没有可用的上级目录"))?;
    let mut file = tempfile::NamedTempFile::new_in(directory).context("旧配置目录暂时无法写入")?;
    file.write_all(replacement).context("恢复后的旧配置尚未写完")?;
    file.as_file().sync_all().context("恢复后的旧配置尚未落盘")?;
    ensure_unchanged(path, expected)?;
    file.persist(path).context("无法发布恢复后的旧配置，原文件仍保留")?;
    sync_directory(directory)?;
    Ok(())
}

fn sync_directory(path: &Path) -> Result<()> {
    #[cfg(unix)]
    std::fs::File::open(path)?.sync_all()?;
    Ok(())
}

pub fn failure_message(error: &anyhow::Error) -> &'static str {
    let io = error.chain().find_map(|cause| cause.downcast_ref::<std::io::Error>());
    match io.map(std::io::Error::kind) {
        Some(ErrorKind::PermissionDenied | ErrorKind::ReadOnlyFilesystem) =>
            "旧配置尚未恢复：配置目录暂时无法写入。原文件保留。",
        Some(ErrorKind::StorageFull) => "旧配置尚未恢复：磁盘空间不足。原文件保留。",
        _ => "旧配置尚未恢复。原文件与当前设置均已保留，可查看原因后重试。",
    }
}

#[cfg(test)]
mod tests {
    use super::{Inspection, ensure_unchanged, preserve_for_import, reset_preserving_original, restore_backup, startup_output};
    use std::path::PathBuf;

    #[test]
    fn fresh_install_bootstraps_local_storage_before_any_folder_permission() {
        let directory = tempfile::tempdir().unwrap();
        let inspected = Inspection::inspect(directory.path().join("config.toml"));
        assert_eq!(
            startup_output(directory.path(), &inspected),
            directory.path().join("desktop-local-library")
        );
        assert!(!directory.path().join("desktop-local-library").exists());
    }

    #[test]
    fn import_entry_requires_backup_and_preserves_hidden_options_across_restart() {
        use std::sync::Arc;
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("config.toml");
        let original = b"# edited legacy settings\n[defaults]\nprovider = \"coreml\"\nasr_model = \"qwen3-0.6b\"\n[llm]\nenabled = false\nsummarize = true\nvision = true\nprompt = \"Keep UX-RULE-42 terms.\"\nbase_url = \"http://127.0.0.1:9/v1\"\nmodel = \"legacy-model\"\napi_key = \"synthetic-import-secret\"\n";
        std::fs::write(&path, original).unwrap();
        let group_root = root.path().join("preferences");
        let vault = Arc::new(crate::credentials::MemoryCredentialVault::default());
        let mut preferences = crate::preferences::Store::open(&group_root, vault.clone());
        let recovery = root.path().join("legacy-config-recovery");
        std::fs::write(&recovery, b"cannot create a backup here").unwrap();
        assert!(super::import_preferences(&path, &mut preferences).is_err());
        assert!(!preferences.legacy_imported());
        assert!(!group_root.join("generation.json").exists());
        assert!(!group_root.join("services.json").exists());
        std::fs::remove_file(recovery).unwrap();
        let preserved = super::import_preferences(&path, &mut preferences).unwrap();
        assert_eq!(std::fs::read(&preserved).unwrap(), original);
        assert_eq!(std::fs::read(&path).unwrap(), original);
        let config = preferences.defaults_config();
        assert_eq!(config.defaults.asr_model.as_deref(), Some("qwen3-0.6b"));
        assert!(!config.llm.enabled && config.llm.summarize && !config.llm.vision);
        assert!(preferences.generation().vision);
        assert_eq!(config.llm.prompt.as_deref(), Some("Keep UX-RULE-42 terms."));
        let files: Vec<_> = ["generation.json", "services.json", "application.json"]
            .into_iter()
            .map(|name| {
                (
                    group_root.join(name),
                    std::fs::read(group_root.join(name)).unwrap(),
                )
            })
            .collect();
        for _ in 0..2 {
            let mut reopened = crate::preferences::Store::open(&group_root, vault.clone());
            super::import_preferences(&path, &mut reopened).unwrap();
            assert_eq!(reopened.versions().count(), 1);
            for (path, bytes) in &files {
                assert_eq!(&std::fs::read(path).unwrap(), bytes);
                assert!(!String::from_utf8_lossy(bytes).contains("synthetic-import-secret"));
            }
        }
    }

    #[test]
    fn damaged_primary_restores_verified_backup_and_preserves_both_originals() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("config.toml");
        let damaged = b"[llm]\napi_key = \"isolated-secret-unterminated";
        let backup = b"[defaults]\nasr_model = \"qwen3-0.6b\"\n";
        std::fs::write(&path, damaged).unwrap();
        std::fs::write(path.with_extension("toml.bak"), backup).unwrap();
        let inspection = Inspection::inspect(path.clone());
        assert!(inspection.backup_available);
        let problem = inspection.problem.unwrap();
        assert!(!problem.detail.contains("isolated-secret"));
        assert!(problem.detail.contains("第 2 行"));
        let preserved = restore_backup(&path).unwrap();
        assert_eq!(std::fs::read(&preserved).unwrap(), damaged);
        assert_eq!(std::fs::read(&path).unwrap(), backup);
        assert_eq!(std::fs::read(path.with_extension("toml.bak")).unwrap(), backup);
        assert!(Inspection::inspect(path.clone()).problem.is_none());
        assert!(restore_backup(&path).is_err());
        #[cfg(unix)] {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(std::fs::metadata(preserved).unwrap().permissions().mode() & 0o777, 0o600);
        }
    }

    #[test]
    fn no_valid_backup_never_overwrites_primary_and_reset_requires_preserved_bytes() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("config.toml");
        let damaged = b"[unknown]\nsend_to_cloud = true\n";
        std::fs::write(&path, damaged).unwrap();
        assert!(restore_backup(&path).is_err());
        std::fs::write(path.with_extension("toml.bak"), b"[bad").unwrap();
        assert!(!Inspection::inspect(path.clone()).backup_available);
        assert!(restore_backup(&path).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), damaged);
        std::fs::write(directory.path().join("legacy-config-recovery"), b"obstruction").unwrap();
        assert!(reset_preserving_original(&path).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), damaged);
        std::fs::remove_file(directory.path().join("legacy-config-recovery")).unwrap();
        let preserved = reset_preserving_original(&path).unwrap();
        assert_eq!(std::fs::read(preserved).unwrap(), damaged);
        let inspected = Inspection::inspect(path.clone());
        assert!(inspected.problem.is_none());
        assert!(!inspected.importable);
        assert!(inspected.output.is_none());
        assert_eq!(std::fs::read(path.with_extension("toml.bak")).unwrap(), b"[bad");
    }

    #[test]
    fn unusable_backup_and_external_change_are_not_repaired_silently() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("config.toml");
        std::fs::write(&path, b"[bad").unwrap();
        std::fs::create_dir(path.with_extension("toml.bak")).unwrap();
        assert!(!Inspection::inspect(path.clone()).backup_available);
        assert!(restore_backup(&path).is_err());
        std::fs::write(&path, b"# externally fixed").unwrap();
        assert!(ensure_unchanged(&path, b"[bad").is_err());
        assert!(reset_preserving_original(&path).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), b"# externally fixed");
    }

    #[test]
    fn corrupt_or_retired_configuration_never_selects_the_users_default_library() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("config.toml");
        std::fs::write(&path, b"[defaults]\nout = \"/users/real-library\"\n[bad").unwrap();
        let safe = directory.path().join("desktop-local-library");
        let inspected = Inspection::inspect(path.clone());
        assert_eq!(startup_output(directory.path(), &inspected), safe);
        assert!(!safe.exists(), "inspection must not create or scan any library");
        std::fs::write(&path, b"[defaults]\nout = \"/users/retired-library\"\n").unwrap();
        let inspected = Inspection::inspect(path);
        std::fs::create_dir(directory.path().join("desktop-preferences")).unwrap();
        assert_eq!(startup_output(directory.path(), &inspected), PathBuf::from("/users/retired-library"));
        std::fs::write(directory.path().join("desktop-preferences/application.json"), b"modern record").unwrap();
        assert_eq!(startup_output(directory.path(), &inspected), safe);
    }

    #[test]
    fn explicit_import_has_a_verified_backup_without_rewriting_source_or_backup() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("config.toml");
        let source = b"# user comment\n[llm]\napi_key = \"isolated-test-key\"\n";
        std::fs::write(&path, source).unwrap();
        std::fs::write(path.with_extension("toml.bak"), b"# previous backup").unwrap();
        let imported = preserve_for_import(&path).unwrap();
        assert_eq!(imported.config.llm.api_key, "isolated-test-key");
        assert_eq!(std::fs::read(imported.original).unwrap(), source);
        assert_eq!(std::fs::read(&path).unwrap(), source);
        assert_eq!(std::fs::read(path.with_extension("toml.bak")).unwrap(), b"# previous backup");
    }
}
