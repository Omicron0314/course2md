//! Exports that retain their contents when moved away from the course library.

use crate::{
    artifact::{self, Document},
    config::OutputFormat,
    execution, render,
};
use anyhow::{Context, Result};
use base64::Engine as _;
use std::{
    collections::BTreeMap,
    io::Write,
    path::{Path, PathBuf},
};

pub fn file_name(format: OutputFormat) -> &'static str {
    match format {
        OutputFormat::Md => "course.zip",
        OutputFormat::Html => "course.html",
        OutputFormat::Json => "structured.json",
    }
}

/// Exact destination, with no overwrite. UI may explicitly confirm replacing a separate
/// exported file, but must never mutate the immutable note version itself.
pub fn export(version_dir: &Path, format: OutputFormat, destination: &Path) -> Result<PathBuf> {
    if !version_dir.join("manifest.json").is_file() && crate::legacy::is_candidate(version_dir) {
        let note = crate::legacy::read(version_dir)?.context("这份旧资料没有可读正文")?;
        let temporary = crate::runtime::TempWorkDir::new("legacy-export")?;
        let imported = crate::legacy::import_note(temporary.path(), note)?;
        return export(&imported.version_dir, format, destination);
    }
    let manifest = artifact::read_manifest(&version_dir.join("manifest.json"))?;
    artifact::validate_version(version_dir, &manifest)?;
    let document: Document = serde_json::from_slice(&std::fs::read(artifact::safe_asset_path(
        version_dir,
        &manifest.document,
    )?)?)?;
    if let Some(provenance) = crate::legacy::provenance(version_dir)?
        && format != OutputFormat::Json
    {
        return write_legacy(version_dir, &document, &provenance, format, destination);
    }
    write_document(version_dir, &document, format, destination)
}

struct ImageAsset {
    id: String,
    path: String,
    mime: &'static str,
    bytes: Vec<u8>,
    sha256: String,
}
fn images(root: &Path, document: &Document) -> Result<BTreeMap<String, ImageAsset>> {
    let mut images = BTreeMap::new();
    for section in document.sections.iter().filter(|s| !s.image.is_empty()) {
        if images.contains_key(&section.image) {
            continue;
        }
        let bytes = std::fs::read(artifact::safe_asset_path(root, &section.image)?)?;
        let sha256 = execution::digest(&bytes);
        let id = format!("image-{}", &sha256[..24]);
        let mime =
            crate::legacy::image_mime(&bytes).context("截图不是可识别的图片，原文件已保留")?;
        let ext = match mime {
            "image/png" => "png",
            "image/gif" => "gif",
            "image/webp" => "webp",
            _ => "jpg",
        };
        images.insert(
            section.image.clone(),
            ImageAsset {
                path: format!("images/{id}.{ext}"),
                id,
                mime,
                bytes,
                sha256,
            },
        );
    }
    Ok(images)
}

fn write_legacy(
    root: &Path,
    document: &Document,
    provenance: &crate::legacy::Provenance,
    format: OutputFormat,
    destination: &Path,
) -> Result<PathBuf> {
    anyhow::ensure!(
        !destination.exists(),
        "导出位置已有文件，未覆盖：{}",
        destination.display()
    );
    let parent = destination
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    std::fs::create_dir_all(parent)?;
    let mut file = tempfile::NamedTempFile::new_in(parent)?;
    if format == OutputFormat::Md {
        let mut zip = zip::ZipWriter::new(file.as_file_mut());
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        zip.start_file("course.md", options)?;
        zip.write_all(&std::fs::read(artifact::safe_asset_path(
            root,
            "course.md",
        )?)?)?;
        let manifest = artifact::read_manifest(&root.join("manifest.json"))?;
        for asset in &manifest.assets {
            if (asset.path.starts_with("original/")
                && !matches!(
                    asset.path.as_str(),
                    "original/run.json" | "original/meta.json"
                ))
                || asset.path.starts_with("assets/")
            {
                zip.start_file(&asset.path, options)?;
                zip.write_all(&std::fs::read(artifact::safe_asset_path(
                    root,
                    &asset.path,
                )?)?)?;
            }
        }
        zip.start_file("import-source.json", options)?;
        let originals = provenance
            .originals
            .iter()
            .filter(|file| !matches!(file.name.as_str(), "run.json" | "meta.json"))
            .collect::<Vec<_>>();
        zip.write_all(&serde_json::to_vec_pretty(&serde_json::json!({"schema":1,"primary":provenance.primary,"originals":originals,"warnings":provenance.warnings}))?)?;
        zip.finish()?;
    } else {
        let mut html = format!(
            "<!doctype html><html lang=\"zh\"><meta charset=\"utf-8\"><title>{}</title><style>body{{max-width:920px;margin:2rem auto;padding:1rem;font-family:system-ui;line-height:1.7}}img{{max-width:100%}}p{{white-space:pre-wrap}}</style><body>",
            render::esc(&document.meta.title)
        );
        for block in &provenance.blocks {
            match block {
                crate::legacy::Block::Heading { text, level, .. } => {
                    html.push_str(&format!("<h{level}>{}</h{level}>", render::esc(text)))
                }
                crate::legacy::Block::Paragraph { text } => {
                    html.push_str(&format!("<p>{}</p>", render::esc(text)))
                }
                crate::legacy::Block::Image { reference, alt } => {
                    if let Some(resource) = provenance
                        .resources
                        .iter()
                        .find(|r| &r.reference == reference)
                    {
                        let bytes =
                            std::fs::read(artifact::safe_asset_path(root, &resource.path)?)?;
                        html.push_str(&format!(
                            "<img alt=\"{}\" src=\"data:{};base64,{}\">",
                            render::esc(alt),
                            resource.mime,
                            base64::engine::general_purpose::STANDARD.encode(bytes)
                        ));
                    }
                }
            }
        }
        if url::Url::parse(&document.meta.webpage_url)
            .is_ok_and(|url| matches!(url.scheme(), "http" | "https"))
        {
            html.push_str(&format!(
                "<p>来源：<a href=\"{}\">{}</a></p>",
                render::esc(&document.meta.webpage_url),
                render::esc(&document.meta.webpage_url)
            ));
        }
        for warning in &provenance.warnings {
            html.push_str(&format!("<p>{}</p>", render::esc(warning)));
        }
        html.push_str("</body></html>");
        file.write_all(html.as_bytes())?;
    }
    file.as_file().sync_all()?;
    file.persist_noclobber(destination)?;
    artifact::sync_dir(parent)?;
    Ok(destination.to_path_buf())
}

pub(crate) fn write_document(
    root: &Path,
    document: &Document,
    format: OutputFormat,
    destination: &Path,
) -> Result<PathBuf> {
    anyhow::ensure!(
        !destination.exists(),
        "导出位置已有文件，未覆盖：{} / Export destination already exists",
        destination.display()
    );
    let parent = destination
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    std::fs::create_dir_all(parent)?;
    let mut file = tempfile::NamedTempFile::new_in(parent)?;
    let images = images(root, document)?;
    let mut portable = document.clone();
    let local_file = if portable.meta.extractor == "local" {
        let name = Path::new(&portable.meta.webpage_url)
            .file_name()
            .map(|s| s.to_string_lossy().into_owned());
        portable.meta.webpage_url.clear();
        name
    } else {
        None
    };
    for section in &mut portable.sections {
        if let Some(image) = images.get(&section.image) {
            section.image = image.path.clone();
        }
    }
    match format {
        OutputFormat::Md => {
            let mut text = render::render_markdown(&portable.meta, &portable.sections);
            if let Some(summary) = &portable.summary {
                text = crate::summarize::insert_into_md(&text, summary);
            }
            if let Some(name) = &local_file {
                text.push_str(&format!("\n源文件：{}\n", name.replace(['\r', '\n'], " ")));
            }
            let mut zip = zip::ZipWriter::new(file.as_file_mut());
            let options = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Stored);
            zip.start_file("course.md", options)?;
            zip.write_all(text.as_bytes())?;
            let mut written = std::collections::HashSet::new();
            for image in images.values() {
                if written.insert(&image.path) {
                    zip.start_file(&image.path, options)?;
                    zip.write_all(&image.bytes)?;
                }
            }
            zip.finish()?;
        }
        OutputFormat::Html => {
            let mut text = render::render_html(&portable.meta, &portable.sections);
            if let Some(summary) = &portable.summary {
                text = crate::summarize::insert_into_html(&text, summary);
            }
            for image in images.values() {
                text = text.replace(
                    &format!("src=\"{}\"", render::esc(&image.path)),
                    &format!(
                        "src=\"data:{};base64,{}\"",
                        image.mime,
                        base64::engine::general_purpose::STANDARD.encode(&image.bytes)
                    ),
                );
            }
            file.write_all(text.as_bytes())?;
        }
        OutputFormat::Json => {
            let sections = document.sections.iter().map(|section| serde_json::json!({"t":section.t,"end":section.end,"image_id":images.get(&section.image).map(|image|&image.id),"speech":section.speech})).collect::<Vec<_>>();
            let image_data = images.values().map(|image| serde_json::json!({"id":image.id,"mime_type":image.mime,"sha256":image.sha256})).collect::<Vec<_>>();
            file.write_all(&serde_json::to_vec_pretty(&serde_json::json!({"schema":1,"title":document.meta.title,"author":document.meta.uploader,"duration":if document.meta.duration > 0. { Some(document.meta.duration) } else { None },"source":if let Some(name) = local_file {serde_json::json!({"kind":"local","file_name":name})} else if document.meta.webpage_url.is_empty() {serde_json::json!({"kind":"unknown"})}else{serde_json::json!({"kind":"online","url":document.meta.webpage_url})},"summary":document.summary,"sections":sections,"images":image_data}))?)?;
        }
    }
    file.as_file().sync_all()?;
    file.persist_noclobber(destination).with_context(|| {
        format!(
            "无法保存导出文件；原文件未覆盖 / Could not save export: {}",
            destination.display()
        )
    })?;
    artifact::sync_dir(parent)?;
    Ok(destination.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_exports_survive_without_the_original_library_and_never_overwrite() {
        use crate::{
            fetch::VideoMeta,
            timeline::{Section, TranscriptEvent},
        };
        let root = tempfile::tempdir().unwrap();
        let source = root.path().join("source");
        std::fs::create_dir_all(source.join("frames")).unwrap();
        image::RgbImage::from_pixel(2, 2, image::Rgb([20, 80, 180]))
            .save(source.join("frames/a.png"))
            .unwrap();
        let document = Document {
            schema: 1,
            meta: VideoMeta {
                title: "title".into(),
                uploader: String::new(),
                duration: 0.,
                webpage_url: "/private/machine/movie.mp4".into(),
                extractor: "local".into(),
                id: "source".into(),
            },
            sections: vec![Section {
                t: 0.,
                end: 1.,
                image: "frames/a.png".into(),
                speech: vec![TranscriptEvent {
                    start: 0.,
                    end: 1.,
                    text: "Actual note".into(),
                    raw: None,
                }],
            }],
            summary: None,
        };
        let destination = root.path().join("moved");
        for format in [OutputFormat::Md, OutputFormat::Html, OutputFormat::Json] {
            write_document(
                &source,
                &document,
                format,
                &destination.join(file_name(format)),
            )
            .unwrap();
        }
        std::fs::remove_dir_all(source).unwrap();
        let mut zip =
            zip::ZipArchive::new(std::fs::File::open(destination.join("course.zip")).unwrap())
                .unwrap();
        assert_eq!(zip.len(), 2);
        assert!(zip.by_name("course.md").is_ok());
        let html = std::fs::read_to_string(destination.join("course.html")).unwrap();
        assert!(html.contains("data:image/png;base64,"));
        assert!(!html.contains("src=\"images/"));
        assert!(!html.contains("时长 00:00"));
        let json = std::fs::read_to_string(destination.join("structured.json")).unwrap();
        assert!(json.contains("image_id"));
        assert!(!json.contains("/private/machine"));
        assert!(!json.contains("frames/a.png"));
        let existing = destination.join("keep.html");
        std::fs::write(&existing, "user content").unwrap();
        assert!(write_document(root.path(), &document, OutputFormat::Html, &existing).is_err());
        assert_eq!(std::fs::read_to_string(existing).unwrap(), "user content");
    }
}
