//! Published versions are immutable. Work directories and diagnostic logs are not notes.

use crate::{
    checkpoint::atomic_write,
    execution,
    fetch::VideoMeta,
    render,
    summarize::Summary,
    timeline::{FrameEvent, Section},
};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    path::{Component, Path, PathBuf},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    NotRequested,
    Succeeded,
    Partial,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Outcome {
    pub status: Status,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completed: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total: Option<usize>,
}
impl Outcome {
    pub fn succeeded() -> Self {
        Self {
            status: Status::Succeeded,
            message: None,
            completed: None,
            total: None,
        }
    }
    pub fn not_requested() -> Self {
        Self {
            status: Status::NotRequested,
            ..Self::succeeded()
        }
    }
    pub fn failed(message: impl Into<String>) -> Self {
        Self {
            status: Status::Failed,
            message: Some(message.into()),
            ..Self::succeeded()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Outcomes {
    pub transcript: Outcome,
    pub screenshots: Outcome,
    pub proofreading: Outcome,
    pub summary: Outcome,
    pub exports: BTreeMap<String, Outcome>,
}
impl Default for Outcomes {
    fn default() -> Self {
        Self {
            transcript: Outcome::not_requested(),
            screenshots: Outcome::not_requested(),
            proofreading: Outcome::not_requested(),
            summary: Outcome::not_requested(),
            exports: BTreeMap::new(),
        }
    }
}
impl Outcomes {
    pub fn is_partial(&self) -> bool {
        [
            &self.transcript,
            &self.screenshots,
            &self.proofreading,
            &self.summary,
        ]
        .into_iter()
        .chain(self.exports.values())
        .any(|o| matches!(o.status, Status::Failed | Status::Partial))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub schema: u32,
    pub meta: VideoMeta,
    pub sections: Vec<Section>,
    pub summary: Option<Summary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    pub path: String,
    pub bytes: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub schema: u32,
    pub task_id: String,
    pub course_id: String,
    pub source_id: String,
    pub version_id: String,
    pub title: String,
    pub created_at_ms: u64,
    #[serde(default)]
    pub revision: u64,
    pub document: String,
    pub markdown: String,
    pub frames: Vec<FrameEvent>,
    pub assets: Vec<Asset>,
    pub outputs: Vec<String>,
    pub outcomes: Outcomes,
    pub partial: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentVersion {
    pub schema: u32,
    pub course_id: String,
    pub version_id: String,
    pub manifest: String,
}

#[derive(Debug, Clone)]
pub struct Target {
    pub task_id: String,
    pub course_id: String,
    pub source_id: String,
    pub version_id: String,
    pub course_dir: PathBuf,
}
impl Target {
    pub fn from_request(request: &execution::Request) -> Self {
        Self {
            task_id: request.task_id.clone(),
            course_id: request.course_id.clone(),
            source_id: request.source_id.clone(),
            version_id: request.version_id.clone(),
            course_dir: request.course_dir.clone(),
        }
    }
    pub fn version_dir(&self) -> PathBuf {
        self.course_dir.join("versions").join(&self.version_id)
    }
}

pub fn read_manifest(path: &Path) -> Result<Manifest> {
    let value: Manifest = serde_json::from_slice(&std::fs::read(path)?)
        .context("笔记清单损坏 / Invalid note manifest")?;
    anyhow::ensure!(
        value.schema == 1,
        "此笔记版本需要其他版本的软件 / Unsupported manifest schema"
    );
    Ok(value)
}

pub fn published(target: &Target) -> Result<Option<Manifest>> {
    let dir = target.version_dir();
    if !dir.exists() {
        return Ok(None);
    }
    let manifest = read_manifest(&dir.join("manifest.json"))?;
    anyhow::ensure!(
        manifest.task_id == target.task_id
            && manifest.course_id == target.course_id
            && manifest.source_id == target.source_id
            && manifest.version_id == target.version_id,
        "此版本位置已有其他笔记，已保留原文件 / Version identity conflict"
    );
    validate_version(&dir, &manifest)?;
    Ok(Some(manifest))
}

pub fn safe_asset_path(root: &Path, relative: &str) -> Result<PathBuf> {
    let path = Path::new(relative);
    anyhow::ensure!(
        !relative.is_empty() && path.components().all(|c| matches!(c, Component::Normal(_))),
        "笔记包含无效资源路径 / Invalid note asset path"
    );
    let full = root.join(path);
    let canonical = full.canonicalize()?;
    anyhow::ensure!(
        canonical.starts_with(root.canonicalize()?),
        "资源位置超出此笔记 / Asset escapes note directory"
    );
    Ok(full)
}

pub fn validate_version(dir: &Path, manifest: &Manifest) -> Result<()> {
    for asset in &manifest.assets {
        // Optional exports are independent files; their loss never invalidates the body.
        if asset.path.starts_with("exports/") {
            continue;
        }
        let path = safe_asset_path(dir, &asset.path)?;
        anyhow::ensure!(
            std::fs::metadata(&path)?.len() == asset.bytes
                && execution::file_digest(&path)? == asset.sha256,
            "笔记文件已更改或损坏：{} / Note asset changed",
            asset.path
        );
    }
    let body: Document =
        serde_json::from_slice(&std::fs::read(safe_asset_path(dir, &manifest.document)?)?)?;
    anyhow::ensure!(
        has_readable_body(&body.sections),
        "笔记没有可读正文 / Note has no readable body"
    );
    for frame in &manifest.frames {
        safe_asset_path(dir, &frame.image)?;
    }
    Ok(())
}

pub fn has_readable_body(sections: &[Section]) -> bool {
    sections
        .iter()
        .flat_map(|s| &s.speech)
        .any(|e| !e.text.trim().is_empty())
}

/// Build and fsync all files on the target volume, then publish one directory rename.
/// The pointer is a separate atomic commit. A restart can finish it without rerunning AI.
pub async fn publish(
    target: &Target,
    work_dir: &Path,
    meta: &VideoMeta,
    sections: &[Section],
    summary: Option<&Summary>,
    formats: &[crate::config::OutputFormat],
    mut outcomes: Outcomes,
) -> Result<Manifest> {
    anyhow::ensure!(
        has_readable_body(sections),
        "文字未生成，已保留可用素材 / No readable text; available materials are retained in the task"
    );
    std::fs::create_dir_all(&target.course_dir)?;
    let _lock = crate::runtime::lock_file(&target.course_dir.join(".publish.lock"))?;
    if let Some(manifest) = published(target)? {
        commit_current(target, &manifest)?;
        return Ok(manifest);
    }
    let versions = target.course_dir.join("versions");
    std::fs::create_dir_all(&versions)?;
    let staging = tempfile::Builder::new()
        .prefix(".publishing-")
        .tempdir_in(&versions)?;
    let mut frames = Vec::new();
    let mut copied = std::collections::HashSet::new();
    for section in sections.iter().filter(|s| !s.image.is_empty()) {
        let source = safe_asset_path(work_dir, &section.image)?;
        if copied.insert(section.image.clone()) {
            let dest = staging.path().join(&section.image);
            std::fs::create_dir_all(dest.parent().unwrap())?;
            std::fs::copy(&source, &dest)?;
        }
        frames.push(FrameEvent {
            t: section.t,
            image: section.image.clone(),
        });
    }
    let document = Document {
        schema: 1,
        meta: meta.clone(),
        sections: sections.to_vec(),
        summary: summary.cloned(),
    };
    atomic_write(
        &staging.path().join("document.json"),
        &serde_json::to_vec_pretty(&document)?,
    )?;
    let mut markdown = render::render_markdown(meta, sections);
    if let Some(summary) = summary {
        markdown = crate::summarize::insert_into_md(&markdown, summary);
    }
    atomic_write(&staging.path().join("course.md"), markdown.as_bytes())?;
    meta.save(&staging.path().join("meta.json"))?;
    // Preserve original fine-grained transcript for later independent local/AI work.
    if work_dir.join("timeline.jsonl").is_file() {
        std::fs::copy(
            work_dir.join("timeline.jsonl"),
            staging.path().join("timeline.jsonl"),
        )?;
    }
    let mut outputs = Vec::new();
    for format in formats {
        let name = crate::portable::file_name(*format);
        let path = staging.path().join("exports").join(name);
        match crate::portable::write_document(staging.path(), &document, *format, &path) {
            Ok(_) => {
                outputs.push(format!("exports/{name}"));
                outcomes
                    .exports
                    .insert(format.to_string(), Outcome::succeeded());
            }
            Err(error) => {
                outcomes
                    .exports
                    .insert(format.to_string(), Outcome::failed(format!("{error:#}")));
            }
        }
    }
    let mut paths = vec![
        "document.json".to_string(),
        "course.md".to_string(),
        "meta.json".to_string(),
    ];
    if staging.path().join("timeline.jsonl").is_file() {
        paths.push("timeline.jsonl".into());
    }
    paths.extend(copied);
    paths.extend(outputs.iter().cloned());
    paths.sort();
    paths.dedup();
    let assets = paths
        .into_iter()
        .map(|path| -> Result<Asset> {
            let full = staging.path().join(&path);
            std::fs::File::open(&full)?.sync_all()?;
            Ok(Asset {
                bytes: std::fs::metadata(&full)?.len(),
                sha256: execution::file_digest(&full)?,
                path,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let revision = if target.course_dir.join("current.json").exists() {
        let current: CurrentVersion =
            serde_json::from_slice(&std::fs::read(target.course_dir.join("current.json"))?)?;
        read_manifest(&safe_asset_path(&target.course_dir, &current.manifest)?)?
            .revision
            .saturating_add(1)
    } else {
        1
    };
    let manifest = Manifest {
        revision,
        schema: 1,
        task_id: target.task_id.clone(),
        course_id: target.course_id.clone(),
        source_id: target.source_id.clone(),
        version_id: target.version_id.clone(),
        title: meta.title.clone(),
        created_at_ms: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_millis()
            .try_into()?,
        document: "document.json".into(),
        markdown: "course.md".into(),
        frames,
        assets,
        outputs,
        partial: outcomes.is_partial(),
        outcomes,
    };
    atomic_write(
        &staging.path().join("manifest.json"),
        &serde_json::to_vec_pretty(&manifest)?,
    )?;
    sync_dir(staging.path())?;
    std::fs::rename(staging.path(), target.version_dir())
        .context("无法发布笔记版本；旧笔记已保留 / Could not publish note version")?;
    sync_dir(&versions)?;
    commit_current(target, &manifest)?;
    Ok(manifest)
}

fn commit_current(target: &Target, manifest: &Manifest) -> Result<()> {
    let path = target.course_dir.join("current.json");
    if path.is_file() {
        let current: CurrentVersion = serde_json::from_slice(&std::fs::read(&path)?)?;
        anyhow::ensure!(
            current.course_id == target.course_id,
            "课程位置已被其他课程使用 / Course identity conflict"
        );
        // Recovery of an older completed task must never roll back a later version.
        if current.version_id != target.version_id {
            let current_manifest =
                read_manifest(&safe_asset_path(&target.course_dir, &current.manifest)?)?;
            if current_manifest.revision > manifest.revision
                || (current_manifest.revision == manifest.revision
                    && current_manifest.created_at_ms >= manifest.created_at_ms)
            {
                return Ok(());
            }
        }
    }
    atomic_write(
        &path,
        &serde_json::to_vec_pretty(&CurrentVersion {
            schema: 1,
            course_id: target.course_id.clone(),
            version_id: target.version_id.clone(),
            manifest: format!("versions/{}/manifest.json", target.version_id),
        })?,
    )?;
    sync_dir(&target.course_dir)
}

pub fn sync_dir(path: &Path) -> Result<()> {
    #[cfg(unix)]
    std::fs::File::open(path)?.sync_all()?;
    #[cfg(not(unix))]
    let _ = path;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::timeline::TranscriptEvent;
    #[cfg(unix)]
    #[tokio::test]
    async fn interrupted_pointer_commit_recovers_once_and_preserves_edited_old_exports() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let work = dir.path().join("work");
        std::fs::create_dir(&work).unwrap();
        let mut target = Target {
            task_id: "task-1".into(),
            course_id: "course".into(),
            source_id: "source".into(),
            version_id: "v1".into(),
            course_dir: dir.path().join("course"),
        };
        let meta = VideoMeta {
            title: "Existing note".into(),
            uploader: String::new(),
            duration: 1.,
            webpage_url: "https://example.invalid/video".into(),
            extractor: "example".into(),
            id: "source".into(),
        };
        let sections = vec![Section {
            t: 0.,
            end: 1.,
            image: String::new(),
            speech: vec![TranscriptEvent {
                start: 0.,
                end: 1.,
                text: "Preserved body".into(),
                raw: None,
            }],
        }];
        publish(
            &target,
            &work,
            &meta,
            &sections,
            None,
            &[crate::config::OutputFormat::Html],
            Outcomes::default(),
        )
        .await
        .unwrap();
        let old_export = target.version_dir().join("exports/course.html");
        let edited = b"<p>My manual corrections must survive.</p>";
        std::fs::write(&old_export, edited).unwrap();
        let pointer = target.course_dir.join("current.json");
        let original_pointer = std::fs::read(&pointer).unwrap();
        target.task_id = "task-2".into();
        target.version_id = "v2".into();
        // Version storage stays writable; only publishing the current pointer fails.
        std::fs::set_permissions(&target.course_dir, std::fs::Permissions::from_mode(0o555))
            .unwrap();
        let interrupted = publish(
            &target,
            &work,
            &meta,
            &sections,
            None,
            &[],
            Outcomes::default(),
        )
        .await;
        std::fs::set_permissions(&target.course_dir, std::fs::Permissions::from_mode(0o755))
            .unwrap();
        assert!(interrupted.is_err());
        assert!(target.version_dir().join("manifest.json").is_file());
        assert_eq!(std::fs::read(&pointer).unwrap(), original_pointer);
        assert_eq!(std::fs::read(&old_export).unwrap(), edited);
        let version_bytes = std::fs::read(target.version_dir().join("manifest.json")).unwrap();
        for _ in 0..2 {
            publish(
                &target,
                &work,
                &meta,
                &sections,
                None,
                &[],
                Outcomes::default(),
            )
            .await
            .unwrap();
            assert_eq!(
                std::fs::read(target.version_dir().join("manifest.json")).unwrap(),
                version_bytes
            );
            assert_eq!(std::fs::read(&old_export).unwrap(), edited);
        }
        let current: CurrentVersion =
            serde_json::from_slice(&std::fs::read(&pointer).unwrap()).unwrap();
        assert_eq!(current.version_id, "v2");
        // A delayed restart of the old task cannot roll back the newer pointer.
        target.task_id = "task-1".into();
        target.version_id = "v1".into();
        publish(
            &target,
            &work,
            &meta,
            &sections,
            None,
            &[],
            Outcomes::default(),
        )
        .await
        .unwrap();
        let current: CurrentVersion =
            serde_json::from_slice(&std::fs::read(pointer).unwrap()).unwrap();
        assert_eq!(current.version_id, "v2");
    }

    #[tokio::test]
    async fn empty_export_selection_still_publishes_body_and_failure_preserves_old_version() {
        let dir = tempfile::tempdir().unwrap();
        let work = dir.path().join("work");
        std::fs::create_dir_all(&work).unwrap();
        let mut target = Target {
            task_id: "task-1".into(),
            course_id: "course".into(),
            source_id: "video:part1".into(),
            version_id: "v1".into(),
            course_dir: dir.path().join("course"),
        };
        let meta = VideoMeta {
            title: "Confirmed title".into(),
            uploader: String::new(),
            duration: 1.,
            webpage_url: "https://example.com/video".into(),
            extractor: "example".into(),
            id: "video:part1".into(),
        };
        let sections = vec![Section {
            t: 0.,
            end: 1.,
            image: String::new(),
            speech: vec![TranscriptEvent {
                start: 0.,
                end: 1.,
                text: "Readable text".into(),
                raw: None,
            }],
        }];
        let first = publish(
            &target,
            &work,
            &meta,
            &sections,
            None,
            &[],
            Outcomes::default(),
        )
        .await
        .unwrap();
        assert!(first.outputs.is_empty());
        assert!(target.version_dir().join("course.md").is_file());
        let old_pointer = std::fs::read(target.course_dir.join("current.json")).unwrap();
        target.task_id = "task-2".into();
        target.version_id = "v2".into();
        let mut broken = sections.clone();
        broken[0].image = "frames/missing.jpg".into();
        assert!(
            publish(
                &target,
                &work,
                &meta,
                &broken,
                None,
                &[],
                Outcomes::default()
            )
            .await
            .is_err()
        );
        assert_eq!(
            std::fs::read(target.course_dir.join("current.json")).unwrap(),
            old_pointer
        );
        assert!(
            target
                .course_dir
                .join("versions/v1/document.json")
                .is_file()
        );
        assert!(!target.version_dir().exists());
        assert!(
            publish(&target, &work, &meta, &[], None, &[], Outcomes::default())
                .await
                .is_err()
        );
    }
}
