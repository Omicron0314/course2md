use course2md::{artifact, config::OutputFormat, legacy, portable};
use std::{io::Read, path::Path};

fn fixture(kind: &str, root: &Path) {
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/legacy")
        .join(kind);
    std::fs::create_dir_all(root.join("frames")).unwrap();
    for entry in std::fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        std::fs::copy(entry.path(), root.join(entry.file_name())).unwrap();
    }
    image::RgbImage::from_pixel(2, 2, image::Rgb([60, 110, 170]))
        .save(root.join("frames/slide one.png"))
        .unwrap();
}
#[test]
fn markdown_import_preserves_manual_bytes_and_resources_and_is_idempotent() {
    let root = tempfile::tempdir().unwrap();
    fixture("markdown", root.path());
    let original = std::fs::read(root.path().join("course.md")).unwrap();
    let picture = std::fs::read(root.path().join("frames/slide one.png")).unwrap();
    let first = legacy::import(root.path()).unwrap().unwrap();
    assert_eq!(
        std::fs::read(root.path().join("course.md")).unwrap(),
        original
    );
    assert_eq!(
        std::fs::read(first.version_dir.join("original/course.md")).unwrap(),
        original
    );
    assert_eq!(
        std::fs::read(first.version_dir.join("original/frames/slide one.png")).unwrap(),
        picture
    );
    assert_eq!(first.manifest.title, "手工修订的金融课");
    assert_eq!(first.manifest.frames.len(), 1);
    let document: artifact::Document =
        serde_json::from_slice(&std::fs::read(first.version_dir.join("document.json")).unwrap())
            .unwrap();
    assert!(
        document
            .sections
            .iter()
            .flat_map(|s| &s.speech)
            .any(|e| e.text.contains("列表里的人工备注"))
    );
    assert_eq!(document.meta.uploader, "");
    assert!(document.meta.webpage_url.contains("p=2"));
    let markdown = std::fs::read_to_string(first.version_dir.join("course.md")).unwrap();
    assert!(markdown.contains("**风险说明**"));
    assert!(markdown.contains("![手写标记](assets/"));
    assert!(markdown.contains("后面的文字不能丢"));
    let second = legacy::import(root.path()).unwrap().unwrap();
    assert_eq!(first.version_dir, second.version_dir);
    assert_eq!(
        std::fs::read_dir(root.path().join(legacy::IMPORT_DIR).join("versions"))
            .unwrap()
            .count(),
        1
    );
    std::fs::write(
        root.path().join("course.md"),
        format!(
            "{}\n新加的人工段落。\n",
            String::from_utf8(original.clone()).unwrap()
        ),
    )
    .unwrap();
    let third = legacy::import(root.path()).unwrap().unwrap();
    assert_ne!(third.version_dir, first.version_dir);
    assert_eq!(
        std::fs::read(first.version_dir.join("original/course.md")).unwrap(),
        original
    );
    artifact::validate_version(&first.version_dir, &first.manifest).unwrap();
}
#[test]
fn html_only_and_json_only_are_readable_without_running_a_pipeline() {
    for kind in ["html", "json"] {
        let root = tempfile::tempdir().unwrap();
        fixture(kind, root.path());
        let imported = legacy::import(root.path()).unwrap().unwrap();
        assert_eq!(imported.manifest.frames.len(), 1);
        assert!(!root.path().join("audio.wav").exists());
        assert!(!root.path().join("requests").exists());
        let document: artifact::Document = serde_json::from_slice(
            &std::fs::read(imported.version_dir.join("document.json")).unwrap(),
        )
        .unwrap();
        assert!(artifact::has_readable_body(&document.sections));
        if kind == "html" {
            assert_eq!(imported.manifest.title, "手工修改的网页标题");
            assert!(imported.provenance.blocks.iter().any(
                |b| matches!(b,legacy::Block::Heading{seconds:None,text,..} if text=="补充主题")
            ));
            assert!(!format!("{:?}", imported.provenance.blocks).contains("do_not_execute"));
        } else {
            assert!(document.meta.webpage_url.is_empty());
            assert!(
                imported
                    .provenance
                    .warnings
                    .iter()
                    .any(|w| w.contains("没有记录视频来源"))
            );
        }
    }
}
#[test]
fn placeholders_and_failure_logs_are_materials_while_corrupt_json_is_reported() {
    let root = tempfile::tempdir().unwrap();
    fixture("empty", root.path());
    assert!(legacy::read(root.path()).unwrap().is_none());
    assert!(legacy::import(root.path()).unwrap().is_none());
    assert!(!root.path().join(legacy::IMPORT_DIR).exists());
    std::fs::write(root.path().join("structured.json"), b"{broken").unwrap();
    assert!(legacy::read(root.path()).is_err());
}
#[test]
fn legacy_exports_remain_portable_and_preserve_originals_in_the_markdown_package() {
    let root = tempfile::tempdir().unwrap();
    let source = root.path().join("source");
    fixture("markdown", &source);
    let original = std::fs::read(source.join("course.md")).unwrap();
    let imported = legacy::import(&source).unwrap().unwrap();
    for format in [OutputFormat::Md, OutputFormat::Html, OutputFormat::Json] {
        portable::export(
            &imported.version_dir,
            format,
            &root.path().join(portable::file_name(format)),
        )
        .unwrap();
    }
    std::fs::remove_dir_all(source).unwrap();
    let mut archive =
        zip::ZipArchive::new(std::fs::File::open(root.path().join("course.zip")).unwrap()).unwrap();
    let mut kept = Vec::new();
    archive
        .by_name("original/course.md")
        .unwrap()
        .read_to_end(&mut kept)
        .unwrap();
    assert_eq!(kept, original);
    assert!(archive.by_name("original/frames/slide one.png").is_ok());
    let html = std::fs::read_to_string(root.path().join("course.html")).unwrap();
    assert!(html.contains("data:image/png;base64,"));
    assert!(html.contains("风险说明"));
    let json = std::fs::read_to_string(root.path().join("structured.json")).unwrap();
    assert!(json.contains("image_id"));
    assert!(!json.contains("base64"));
    assert!(!json.contains("assets/"));
}
#[test]
fn outside_and_remote_images_are_not_read_and_unknown_import_store_is_not_overwritten() {
    let root = tempfile::tempdir().unwrap();
    let source = root.path().join("source");
    fixture("markdown", &source);
    std::fs::write(root.path().join("outside.png"), b"private bytes").unwrap();
    std::fs::write(source.join("course.md"),"# 笔记\n\n正文应当仍可阅读。\n\n![外部](../outside.png)\n\n![远程](https://127.0.0.1:1/private.png)").unwrap();
    let read = legacy::read(&source).unwrap().unwrap();
    assert_eq!(read.provenance.resources.len(), 0);
    assert_eq!(
        read.provenance
            .warnings
            .iter()
            .filter(|w| w.starts_with("旧图片未包含"))
            .count(),
        2
    );
    std::fs::create_dir_all(source.join(legacy::IMPORT_DIR)).unwrap();
    std::fs::write(source.join(legacy::IMPORT_DIR).join("keep"), b"user owned").unwrap();
    assert!(legacy::import(&source).is_err());
    let destination = root.path().join("readonly.html");
    portable::export(&source, OutputFormat::Html, &destination).unwrap();
    assert!(
        std::fs::read_to_string(destination)
            .unwrap()
            .contains("正文应当仍可阅读")
    );
    assert_eq!(
        std::fs::read(source.join(legacy::IMPORT_DIR).join("keep")).unwrap(),
        b"user owned"
    );
    assert!(!source.join(legacy::IMPORT_DIR).join("owner.json").exists());
}
#[test]
fn markdown_copy_removes_real_images_but_keeps_inline_code_and_surrounding_words() {
    let value = "before ![inline](frames/a.png) after\n\n![ref][pic]\n\n[pic]: frames/a.png\n\n`![example](literal)`";
    let copied = legacy::without_images(value);
    assert!(copied.contains("before  after"));
    assert!(!copied.contains("frames/a.png"));
    assert!(copied.contains("`![example](literal)`"));
}

#[cfg(unix)]
#[test]
fn read_only_directory_remains_readable_and_exportable_without_an_internal_write() {
    use std::os::unix::fs::PermissionsExt;
    if unsafe { libc::geteuid() } == 0 {
        return;
    }
    let root = tempfile::tempdir().unwrap();
    let source = root.path().join("read-only");
    fixture("html", &source);
    let original = std::fs::read(source.join("course.html")).unwrap();
    std::fs::set_permissions(&source, std::fs::Permissions::from_mode(0o555)).unwrap();
    let read = legacy::read(&source);
    let imported = legacy::import(&source);
    let exported = portable::export(
        &source,
        OutputFormat::Html,
        &root.path().join("output.html"),
    );
    std::fs::set_permissions(&source, std::fs::Permissions::from_mode(0o755)).unwrap();
    assert!(read.unwrap().is_some());
    assert!(imported.is_err());
    assert!(exported.is_ok());
    assert!(!source.join(legacy::IMPORT_DIR).exists());
    assert_eq!(std::fs::read(source.join("course.html")).unwrap(), original);
}
