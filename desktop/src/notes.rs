//! Read published versions and isolate damaged library entries.
use crate::backend::Completed;
use anyhow::{Context, Result};
use std::{
    collections::VecDeque,
    path::{Path, PathBuf},
    time::SystemTime,
};
#[derive(Clone)]
pub struct Course {
    pub dir: PathBuf,
    pub title: String,
    pub modified: SystemTime,
    pub slides: usize,
    pub segments: usize,
    pub thumbnail: Option<PathBuf>,
    pub manifest: Option<course2md::artifact::Manifest>,
    pub warning: Option<String>,
}
impl Course {
    pub fn from_completed(done: &Completed) -> Self {
        Self {
            dir: done.out_dir.clone(),
            title: done.title.clone(),
            modified: SystemTime::now(),
            slides: done.slides,
            segments: done.segments,
            thumbnail: None,
            manifest: course2md::artifact::read_manifest(&done.out_dir.join("manifest.json")).ok(),
            warning: None,
        }
    }
    pub fn description(&self) -> String {
        if self.warning.is_some() && self.manifest.is_none() {
            return "已有笔记 · 正在只读原稿".into();
        }
        let counts = format!("{} 段笔记 · {} 张截图", self.segments, self.slides);
        if self.manifest.as_ref().is_some_and(|m| m.partial) {
            format!("{counts} · 部分处理未完成")
        } else {
            counts
        }
    }
    pub fn storage_dir(&self) -> PathBuf {
        if self.manifest.is_some() {
            let storage = self
                .dir
                .parent()
                .and_then(|p| p.parent())
                .unwrap_or(&self.dir)
                .to_path_buf();
            if storage
                .file_name()
                .is_some_and(|name| name == course2md::legacy::IMPORT_DIR)
            {
                storage.parent().unwrap_or(&storage).to_path_buf()
            } else {
                storage
            }
        } else {
            self.dir.clone()
        }
    }
}
pub struct LibraryScan {
    pub courses: Vec<Course>,
    pub issues: Vec<String>,
    pub materials: Vec<PathBuf>,
}

fn version_course(dir: &Path) -> Result<Course> {
    let manifest = course2md::artifact::read_manifest(&dir.join("manifest.json"))?;
    let document: course2md::artifact::Document = serde_json::from_slice(&std::fs::read(
        course2md::artifact::safe_asset_path(dir, &manifest.document)?,
    )?)?;
    anyhow::ensure!(
        document.schema == 1 && course2md::artifact::has_readable_body(&document.sections),
        "笔记正文无法读取"
    );
    let warning = course2md::artifact::validate_version(dir, &manifest)
        .err()
        .map(|_| "已发布文件有外部修改或资源缺失；正在读取现有正文，原文件已保留。".into());
    let thumbnail = manifest
        .frames
        .iter()
        .find_map(|f| course2md::artifact::safe_asset_path(dir, &f.image).ok());
    Ok(Course {
        dir: dir.to_path_buf(),
        title: manifest.title.clone(),
        modified: SystemTime::UNIX_EPOCH + std::time::Duration::from_millis(manifest.created_at_ms),
        slides: manifest
            .frames
            .iter()
            .filter(|f| course2md::artifact::safe_asset_path(dir, &f.image).is_ok())
            .count(),
        segments: document.sections.iter().map(|s| s.speech.len()).sum(),
        thumbnail,
        manifest: Some(manifest),
        warning,
    })
}

fn current_course(dir: &Path) -> Result<Course> {
    let load = || -> Result<Course> {
        let current: course2md::artifact::CurrentVersion =
            serde_json::from_slice(&std::fs::read(dir.join("current.json"))?)?;
        anyhow::ensure!(current.schema == 1, "笔记版本暂不受支持");
        let manifest_path = course2md::artifact::safe_asset_path(dir, &current.manifest)?;
        let course = version_course(manifest_path.parent().context("版本位置无效")?)?;
        let manifest = course.manifest.as_ref().unwrap();
        anyhow::ensure!(
            manifest.course_id == current.course_id && manifest.version_id == current.version_id,
            "笔记版本记录不一致"
        );
        Ok(course)
    };
    match load() {
        Ok(course) => Ok(course),
        Err(error) => {
            let mut previous = std::fs::read_dir(dir.join("versions"))
                .into_iter()
                .flatten()
                .filter_map(Result::ok)
                .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()))
                .filter_map(|e| version_course(&e.path()).ok())
                .filter(|c| c.warning.is_none())
                .collect::<Vec<_>>();
            previous.sort_by_key(|c| c.manifest.as_ref().map(|m| m.revision));
            if let Some(mut course) = previous.pop() {
                course.warning = Some(
                    "当前版本暂时无法读取；正在显示此前保存的完好版本，所有原文件已保留。".into(),
                );
                Ok(course)
            } else {
                Err(error)
            }
        }
    }
}

fn legacy_image(
    dir: &Path,
    resource: &course2md::legacy::Resource,
    normalized: bool,
) -> Option<PathBuf> {
    let relative = if normalized {
        resource.path.as_str()
    } else {
        resource
            .original_path
            .as_deref()?
            .strip_prefix("original/")?
    };
    course2md::artifact::safe_asset_path(dir, relative).ok()
}
fn legacy_course(
    dir: &Path,
    note: &course2md::legacy::Note,
    manifest: Option<course2md::artifact::Manifest>,
) -> Course {
    let images = note
        .provenance
        .resources
        .iter()
        .filter_map(|r| legacy_image(dir, r, false))
        .collect::<Vec<_>>();
    Course {
        dir: dir.to_path_buf(),
        title: note.document.meta.title.clone(),
        modified: note.modified,
        slides: images.len(),
        segments: note.document.sections.iter().map(|s| s.speech.len()).sum(),
        thumbnail: images.first().cloned(),
        manifest,
        warning: note.provenance.warnings.first().cloned(),
    }
}
fn legacy_preview(
    mut course: Course,
    document: course2md::artifact::Document,
    markdown: String,
    provenance: course2md::legacy::Provenance,
    normalized: bool,
) -> Preview {
    let mut blocks = Vec::new();
    let mut plain_text = String::new();
    let mut frames = Vec::new();
    for (index, block) in provenance.blocks.iter().enumerate() {
        match block {
            course2md::legacy::Block::Heading { text, seconds, .. } => {
                blocks.push(PreviewBlock::Heading {
                    text: text.clone(),
                    anchor: format!("legacy-heading-{index}"),
                    seconds: *seconds,
                });
                plain_text.push_str(text);
                plain_text.push_str("\n\n");
            }
            course2md::legacy::Block::Paragraph { text } => {
                blocks.push(PreviewBlock::Paragraph {
                    text: text.clone(),
                    anchor: format!("legacy-paragraph-{index}"),
                });
                plain_text.push_str(text);
                plain_text.push_str("\n\n");
            }
            course2md::legacy::Block::Image { reference, .. } => {
                if let Some(path) = provenance
                    .resources
                    .iter()
                    .find(|r| &r.reference == reference)
                    .and_then(|r| legacy_image(&course.dir, r, normalized))
                {
                    blocks.push(PreviewBlock::Image(path.clone()));
                    if !frames.contains(&path) {
                        frames.push(path);
                    }
                }
            }
        }
    }
    let mut metadata = vec![("原稿".into(), provenance.primary.clone())];
    if !document.meta.uploader.is_empty() {
        metadata.push(("作者".into(), document.meta.uploader.clone()));
    }
    if document.meta.duration > 0. {
        metadata.push((
            "时长".into(),
            course2md::render::fmt_ts(document.meta.duration),
        ));
    }
    if !document.meta.webpage_url.is_empty() {
        metadata.push(("来源".into(), document.meta.webpage_url.clone()));
    }
    let mut issues = provenance.warnings;
    if let Some(warning) = &course.warning {
        if !issues.contains(warning) {
            issues.push(warning.clone());
        }
    }
    course.title = document.meta.title.clone();
    course.slides = frames.len();
    course.segments = document.sections.iter().map(|s| s.speech.len()).sum();
    let processing_issues = course
        .manifest
        .as_ref()
        .map(processing_issues)
        .unwrap_or_default();
    Preview {
        markdown_text: without_image_references(&markdown),
        markdown,
        course,
        blocks,
        frames,
        has_markdown: true,
        outputs: vec![],
        plain_text,
        document: Some(document),
        metadata,
        issues,
        processing_issues,
    }
}

pub fn library(root: &Path) -> Result<Vec<Course>> {
    Ok(scan_library(root)?.courses)
}

pub fn scan_library(root: &Path) -> Result<LibraryScan> {
    let mut scan = LibraryScan {
        courses: Vec::new(),
        issues: Vec::new(),
        materials: Vec::new(),
    };
    if !root.exists() {
        return Ok(scan);
    }
    let mut queue = VecDeque::from([(root.to_path_buf(), 0)]);
    while let Some((dir, depth)) = queue.pop_front() {
        if dir.join("current.json").is_file() {
            match current_course(&dir) {
                Ok(course) => scan.courses.push(course),
                Err(error) => scan.issues.push(format!("{}：{error:#}", dir.display())),
            }
            continue;
        }
        if course2md::legacy::is_candidate(&dir)
            || dir
                .join(course2md::legacy::IMPORT_DIR)
                .join("current.json")
                .is_file()
        {
            match course2md::legacy::read(&dir) {
                Ok(Some(note)) => {
                    let mut fallback = legacy_course(&dir, &note, None);
                    match course2md::legacy::import_note(&dir, note) {
                        Ok(imported) => match version_course(&imported.version_dir) {
                            Ok(course) => scan.courses.push(course),
                            Err(error) => {
                                fallback.warning =
                                    Some(format!("内部副本暂时无法读取，正在读取原稿：{error:#}"));
                                scan.courses.push(fallback);
                            }
                        },
                        Err(error) => {
                            fallback.warning =
                                Some(format!("正在只读原稿；内部副本尚未保存：{error:#}"));
                            scan.courses.push(fallback);
                        }
                    }
                }
                Ok(None) => match current_course(&dir.join(course2md::legacy::IMPORT_DIR)) {
                    Ok(mut course) => {
                        course.warning = Some(
                            "原稿目前没有可读正文；正在显示此前导入的版本，原文件已保留。".into(),
                        );
                        scan.courses.push(course);
                    }
                    Err(_) => scan.materials.push(dir),
                },
                Err(error) => match current_course(&dir.join(course2md::legacy::IMPORT_DIR)) {
                    Ok(mut course) => {
                        course.warning = Some(format!(
                            "原稿暂时无法读取；正在显示此前导入的版本：{error:#}"
                        ));
                        scan.courses.push(course);
                    }
                    Err(_) => scan.issues.push(format!("{}：{error:#}", dir.display())),
                },
            }
            continue;
        }
        if depth < 8 {
            match std::fs::read_dir(&dir) {
                Ok(entries) => {
                    for entry in entries {
                        match entry {
                            Ok(entry) => {
                                if entry.file_name().to_string_lossy().starts_with('.')
                                    || entry.file_name() == "versions"
                                {
                                    continue;
                                }
                                match entry.file_type() {
                                    Ok(kind) if kind.is_dir() => {
                                        queue.push_back((entry.path(), depth + 1))
                                    }
                                    Ok(_) => {}
                                    Err(error) => scan
                                        .issues
                                        .push(format!("{}：{error}", entry.path().display())),
                                }
                            }
                            Err(error) => scan.issues.push(format!("{}：{error}", dir.display())),
                        }
                    }
                }
                Err(error) => scan.issues.push(format!("{}：{error}", dir.display())),
            }
        }
    }
    scan.courses.sort_by_key(|c| std::cmp::Reverse(c.modified));
    Ok(scan)
}

pub fn default_output() -> PathBuf {
    let home = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"));
    PathBuf::from(home.unwrap_or_else(|| ".".into())).join("Documents/course2md")
}

#[derive(Clone)]
pub enum PreviewBlock {
    Markdown(String),
    Image(PathBuf),
    Heading {
        text: String,
        anchor: String,
        seconds: Option<f64>,
    },
    Paragraph {
        text: String,
        anchor: String,
    },
}
#[derive(Clone)]
pub enum ProcessingStage {
    Transcript,
    Screenshots,
    Proofreading,
    Summary,
}
impl ProcessingStage {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Transcript => "部分文字",
            Self::Screenshots => "截图",
            Self::Proofreading => "AI 校对",
            Self::Summary => "摘要",
        }
    }
}
#[derive(Clone)]
pub struct ProcessingIssue {
    pub stage: ProcessingStage,
    pub outcome: course2md::artifact::Outcome,
}
fn processing_issues(manifest: &course2md::artifact::Manifest) -> Vec<ProcessingIssue> {
    [
        (ProcessingStage::Transcript, &manifest.outcomes.transcript),
        (ProcessingStage::Screenshots, &manifest.outcomes.screenshots),
        (
            ProcessingStage::Proofreading,
            &manifest.outcomes.proofreading,
        ),
        (ProcessingStage::Summary, &manifest.outcomes.summary),
    ]
    .into_iter()
    .filter(|(_, outcome)| {
        matches!(
            outcome.status,
            course2md::artifact::Status::Failed | course2md::artifact::Status::Partial
        )
    })
    .map(|(stage, outcome)| ProcessingIssue {
        stage,
        outcome: outcome.clone(),
    })
    .collect()
}
#[derive(Clone)]
pub struct Preview {
    pub course: Course,
    pub markdown: String,
    pub blocks: Vec<PreviewBlock>,
    pub frames: Vec<PathBuf>,
    pub has_markdown: bool,
    pub outputs: Vec<String>,
    pub plain_text: String,
    pub markdown_text: String,
    pub document: Option<course2md::artifact::Document>,
    pub metadata: Vec<(String, String)>,
    pub issues: Vec<String>,
    /// Incomplete processing is recoverable through its task, never by re-reading files.
    pub processing_issues: Vec<ProcessingIssue>,
}

pub fn read_preview(mut course: Course) -> Result<Preview> {
    if course.dir.join("manifest.json").is_file() {
        let manifest = course2md::artifact::read_manifest(&course.dir.join("manifest.json"))?;
        let path = course2md::artifact::safe_asset_path(&course.dir, &manifest.document)?;
        let document: course2md::artifact::Document =
            serde_json::from_slice(&std::fs::read(path)?)?;
        anyhow::ensure!(
            course2md::artifact::has_readable_body(&document.sections),
            "笔记正文无法读取，原文件已保留"
        );
        let markdown = course2md::artifact::safe_asset_path(&course.dir, &manifest.markdown)
            .and_then(|path| Ok(std::fs::read_to_string(path)?))
            .unwrap_or_else(|_| {
                course2md::render::render_markdown(&document.meta, &document.sections)
            });
        if let Some(provenance) = course2md::legacy::provenance(&course.dir)? {
            course.manifest = Some(manifest);
            return Ok(legacy_preview(course, document, markdown, provenance, true));
        }
        let mut plain_text = format!("{}\n\n", manifest.title);
        let mut blocks = Vec::new();
        if let Some(summary) = &document.summary {
            blocks.push(PreviewBlock::Heading {
                text: "摘要".into(),
                anchor: "summary".into(),
                seconds: None,
            });
            blocks.push(PreviewBlock::Paragraph {
                text: summary.tldr.clone(),
                anchor: "summary-tldr".into(),
            });
            plain_text.push_str(&summary.tldr);
            plain_text.push_str("\n\n");
            for (i, point) in summary.key_points.iter().enumerate() {
                blocks.push(PreviewBlock::Paragraph {
                    text: format!("• {point}"),
                    anchor: format!("key-point-{i}"),
                });
                plain_text.push_str(point);
                plain_text.push('\n');
            }
        }
        let mut issues = course.warning.clone().into_iter().collect::<Vec<_>>();
        let processing_issues = processing_issues(&manifest);
        let mut frames = Vec::new();
        for frame in &manifest.frames {
            match course2md::artifact::safe_asset_path(&course.dir, &frame.image) {
                Ok(path) => frames.push(path),
                Err(_) => issues.push(format!(
                    "{} 的截图暂时无法读取",
                    course2md::render::fmt_ts(frame.t)
                )),
            }
        }
        for (section_index, section) in document.sections.iter().enumerate() {
            blocks.push(PreviewBlock::Heading {
                text: course2md::render::fmt_ts(section.t),
                anchor: format!("section-{section_index}"),
                seconds: Some(section.t),
            });
            if !section.image.is_empty()
                && manifest.frames.iter().any(|f| f.image == section.image)
                && let Ok(path) = course2md::artifact::safe_asset_path(&course.dir, &section.image)
            {
                blocks.push(PreviewBlock::Image(path));
            }
            for (paragraph_index, paragraph) in section.speech.iter().enumerate() {
                if !paragraph.text.trim().is_empty() {
                    let anchor = format!("paragraph-{section_index}-{paragraph_index}");
                    blocks.push(PreviewBlock::Paragraph {
                        text: paragraph.text.clone(),
                        anchor,
                    });
                    plain_text.push_str(&paragraph.text);
                    plain_text.push_str("\n\n");
                }
            }
        }
        let mut metadata = Vec::new();
        if !document.meta.uploader.is_empty() {
            metadata.push(("作者".into(), document.meta.uploader.clone()));
        }
        if document.meta.duration > 0. {
            metadata.push((
                "时长".into(),
                course2md::render::fmt_ts(document.meta.duration),
            ));
        }
        if !document.meta.webpage_url.is_empty() {
            metadata.push(("来源".into(), document.meta.webpage_url.clone()));
        }
        course.title = manifest.title.clone();
        course.slides = manifest.frames.len();
        course.segments = document.sections.iter().map(|s| s.speech.len()).sum();
        course.manifest = Some(manifest.clone());
        let markdown_text = without_image_references(&markdown);
        return Ok(Preview {
            course,
            markdown,
            blocks,
            frames,
            has_markdown: true,
            outputs: manifest.outputs,
            plain_text,
            markdown_text,
            document: Some(document),
            metadata,
            issues,
            processing_issues,
        });
    }
    let note =
        course2md::legacy::read(&course.dir)?.context("这份旧资料没有可读正文；原文件已保留")?;
    match course2md::legacy::import_note(&course.dir, note) {
        Ok(imported) => {
            course.dir = imported.version_dir;
            read_preview(course)
        }
        Err(error) => {
            let note = course2md::legacy::read(&course.dir)?.context("原稿暂时无法读取")?;
            let mut preview =
                legacy_preview(course, note.document, note.markdown, note.provenance, false);
            preview
                .issues
                .push(format!("正在只读原稿；内部副本尚未保存：{error:#}"));
            Ok(preview)
        }
    }
}

pub fn without_image_references(markdown: &str) -> String {
    course2md::legacy::without_images(markdown)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn library_discovers_json_body_without_run_and_keeps_failure_materials_separate() {
        let root = tempfile::tempdir().unwrap();
        let note = root.path().join("only-json");
        let failure = root.path().join("failed");
        std::fs::create_dir_all(&note).unwrap();
        std::fs::create_dir_all(&failure).unwrap();
        std::fs::write(
            note.join("structured.json"),
            include_str!("../../tests/fixtures/legacy/json/structured.json"),
        )
        .unwrap();
        std::fs::write(failure.join("run.json"), r#"{"success":false}"#).unwrap();
        let first = scan_library(root.path()).unwrap();
        assert_eq!(first.courses.len(), 1);
        assert_eq!(first.materials, vec![failure]);
        assert_eq!(
            first.courses[0].storage_dir().canonicalize().unwrap(),
            note.canonicalize().unwrap()
        );
        let version = first.courses[0].dir.clone();
        let preview = read_preview(first.courses[0].clone()).unwrap();
        assert!(preview.plain_text.contains("真实正文"));
        assert!(preview.document.is_some());
        assert!(
            preview
                .blocks
                .iter()
                .any(|b| matches!(b,PreviewBlock::Heading{seconds:Some(t),..} if *t==10.))
        );
        let second = scan_library(root.path()).unwrap();
        assert_eq!(second.courses[0].dir, version);
    }
    #[test]
    fn normalization_failure_does_not_hide_existing_html_and_still_allows_export() {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(
            root.path().join("course.html"),
            include_str!("../../tests/fixtures/legacy/html/course.html"),
        )
        .unwrap();
        let reserved = root.path().join(course2md::legacy::IMPORT_DIR);
        std::fs::create_dir_all(&reserved).unwrap();
        std::fs::write(reserved.join("unrelated"), "用户文件").unwrap();
        let scan = scan_library(root.path()).unwrap();
        assert_eq!(scan.courses.len(), 1);
        assert!(scan.courses[0].manifest.is_none());
        let preview = read_preview(scan.courses[0].clone()).unwrap();
        assert!(preview.document.is_some());
        assert!(preview.plain_text.contains("第一条手工备注"));
        assert!(!preview.plain_text.contains("do_not_execute"));
        assert!(
            preview.blocks.iter().any(
                |b| matches!(b,PreviewBlock::Heading{text,seconds:None,..} if text=="补充主题")
            )
        );
        let export = root.path().join("exported.html");
        course2md::portable::export(
            &preview.course.dir,
            course2md::config::OutputFormat::Html,
            &export,
        )
        .unwrap();
        assert!(
            std::fs::read_to_string(export)
                .unwrap()
                .contains("人工说明")
        );
        assert_eq!(
            std::fs::read_to_string(reserved.join("unrelated")).unwrap(),
            "用户文件"
        );
        assert!(!reserved.join("owner.json").exists());
    }
}
