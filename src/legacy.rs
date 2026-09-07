//! Local, non-destructive import of older note exports. Never runs a model or fetches URLs.
use crate::{
    artifact::{self, Asset, Document, Manifest, Outcome, Outcomes, Status},
    execution,
    fetch::VideoMeta,
    timeline::{Section, TranscriptEvent},
};
use anyhow::{Context, Result};
use html5ever::tokenizer::{
    BufferQueue, Token, TokenSink, TokenSinkResult, Tokenizer, TokenizerOpts,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    time::SystemTime,
};

pub const IMPORT_DIR: &str = ".course2md-import";
const TEXT_LIMIT: u64 = 32 * 1024 * 1024;
const RESOURCE_LIMIT: u64 = 64 * 1024 * 1024;
const TOTAL_LIMIT: usize = 256 * 1024 * 1024;
const CANDIDATES: [&str; 4] = [
    "course.md",
    "course.html",
    "structured.json",
    "document.json",
];

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Block {
    Heading {
        text: String,
        level: u8,
        seconds: Option<f64>,
    },
    Paragraph {
        text: String,
    },
    Image {
        reference: String,
        alt: String,
    },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Original {
    pub name: String,
    pub path: String,
    pub sha256: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Resource {
    pub reference: String,
    pub path: String,
    pub original_path: Option<String>,
    pub mime: String,
    pub sha256: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Provenance {
    pub schema: u32,
    pub primary: String,
    pub originals: Vec<Original>,
    pub resources: Vec<Resource>,
    pub blocks: Vec<Block>,
    pub warnings: Vec<String>,
}
pub struct Note {
    pub document: Document,
    pub markdown: String,
    pub provenance: Provenance,
    pub modified: SystemTime,
    fingerprint: String,
    files: BTreeMap<String, Vec<u8>>,
}
pub struct Imported {
    pub version_dir: PathBuf,
    pub manifest: Manifest,
    pub provenance: Provenance,
}
#[derive(Default)]
struct Parsed {
    blocks: Vec<Block>,
    title: Option<String>,
    source: Option<String>,
    author: Option<String>,
    duration: Option<f64>,
    meta: Option<VideoMeta>,
    sections: Option<Vec<Section>>,
    summary: Option<crate::summarize::Summary>,
}

fn known_meta(value: &serde_json::Value) -> Option<VideoMeta> {
    value.as_object()?;
    let string = |key: &str| value[key].as_str().unwrap_or_default().to_owned();
    Some(VideoMeta {
        title: string("title"),
        uploader: string("uploader"),
        duration: value["duration"]
            .as_f64()
            .filter(|v| v.is_finite() && *v >= 0.)
            .unwrap_or(0.),
        webpage_url: string("webpage_url"),
        extractor: string("extractor"),
        id: string("id"),
    })
}

pub fn is_candidate(dir: &Path) -> bool {
    CANDIDATES.iter().any(|name| dir.join(name).is_file()) || dir.join("run.json").is_file()
}
fn read_bounded(path: &Path, limit: u64) -> Result<Vec<u8>> {
    use std::io::Read as _;
    anyhow::ensure!(
        std::fs::metadata(path)?.is_file(),
        "引用不是普通文件，未读取：{}",
        path.display()
    );
    anyhow::ensure!(
        std::fs::metadata(path)?.len() <= limit,
        "旧文件过大，未改动原文件：{}",
        path.display()
    );
    let mut bytes = Vec::new();
    std::fs::File::open(path)?
        .take(limit + 1)
        .read_to_end(&mut bytes)?;
    anyhow::ensure!(
        bytes.len() as u64 <= limit,
        "旧文件在读取时发生变化，原文件已保留"
    );
    Ok(bytes)
}
fn text(bytes: &[u8]) -> Result<String> {
    if bytes.starts_with(&[0xff, 0xfe]) || bytes.starts_with(&[0xfe, 0xff]) {
        anyhow::ensure!(bytes.len() % 2 == 0, "旧文件的 UTF-16 编码不完整");
        let little = bytes[0] == 0xff;
        let units = bytes[2..]
            .chunks_exact(2)
            .map(|b| {
                if little {
                    u16::from_le_bytes([b[0], b[1]])
                } else {
                    u16::from_be_bytes([b[0], b[1]])
                }
            })
            .collect::<Vec<_>>();
        return String::from_utf16(&units).context("旧文件包含无效 UTF-16 文字");
    }
    Ok(
        std::str::from_utf8(bytes.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(bytes))
            .context("旧文件不是可识别的 UTF-8 或 UTF-16 文字；原文件已保留")?
            .to_owned(),
    )
}
fn meaningful(value: &str) -> bool {
    let value = value.trim();
    !value.is_empty() && !matches!(value, "(本段无语音)" | "（本段无语音）" | "本段无语音")
}
pub fn parse_seconds(value: &str) -> Option<f64> {
    let value = value.trim().trim_matches(['[', ']']);
    let parts = value.split(':').collect::<Vec<_>>();
    if !(2..=3).contains(&parts.len()) {
        return None;
    }
    let mut result = 0.;
    for (i, part) in parts.iter().enumerate() {
        let n: f64 = part.parse().ok()?;
        if !n.is_finite() || n < 0. || (i > 0 && n >= 60.) {
            return None;
        }
        result = result * 60. + n;
    }
    Some(result)
}

#[derive(Default)]
struct HtmlReader {
    parsed: Parsed,
    buffer: String,
    heading: Option<u8>,
    hidden: usize,
    in_header: usize,
    title_tag: bool,
    link: Option<String>,
    list_item: bool,
    seen_h1: bool,
}
impl HtmlReader {
    fn flush(&mut self) {
        let value = std::mem::take(&mut self.buffer).trim().to_owned();
        if value.is_empty() {
            return;
        }
        if self.title_tag {
            if self.parsed.title.is_none() {
                self.parsed.title = Some(value);
            }
        } else if let Some(level) = self.heading {
            if level == 1 && !self.seen_h1 {
                self.parsed.title = Some(value.clone());
                self.seen_h1 = true;
            }
            self.parsed.blocks.push(Block::Heading {
                seconds: parse_seconds(&value),
                text: value,
                level,
            });
        } else {
            if self.in_header > 0
                && (value.contains("张截图")
                    || value.starts_with("作者 ")
                    || value.starts_with("时长 "))
            {
                for part in value.split('·').map(str::trim) {
                    if let Some(author) = part
                        .strip_prefix("作者 ")
                        .filter(|a| !matches!(*a, "未知" | "unknown"))
                    {
                        self.parsed.author = Some(author.into());
                    }
                    if let Some(duration) = part.strip_prefix("时长 ").and_then(parse_seconds) {
                        self.parsed.duration = Some(duration);
                    }
                }
                return;
            }
            if meaningful(&value) {
                self.parsed.blocks.push(Block::Paragraph {
                    text: if self.list_item {
                        format!("• {value}")
                    } else {
                        value
                    },
                });
            }
        }
    }
}
impl TokenSink for HtmlReader {
    type Handle = ();
    fn process_token(&mut self, token: Token, _: u64) -> TokenSinkResult<()> {
        use html5ever::tokenizer::{EndTag, StartTag};
        match token {
            Token::TagToken(tag) => {
                let name = tag.name.as_ref();
                let start = tag.kind == StartTag;
                if matches!(name, "script" | "style" | "template" | "noscript") {
                    if start {
                        self.hidden += 1;
                    } else {
                        self.hidden = self.hidden.saturating_sub(1);
                    }
                    return TokenSinkResult::Continue;
                }
                if self.hidden > 0 {
                    return TokenSinkResult::Continue;
                }
                let attr = |wanted: &str| {
                    tag.attrs
                        .iter()
                        .find(|a| a.name.local.as_ref() == wanted)
                        .map(|a| a.value.to_string())
                };
                match name {
                    "title" => {
                        self.flush();
                        self.title_tag = start;
                    }
                    "header" => {
                        self.flush();
                        if start {
                            self.in_header += 1;
                        } else {
                            self.in_header = self.in_header.saturating_sub(1);
                        }
                    }
                    "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                        self.flush();
                        self.heading = start.then(|| name.as_bytes()[1] - b'0');
                    }
                    "a" => {
                        if start {
                            self.link = attr("href");
                        } else if let Some(href) = self.link.take() {
                            let label = self.buffer.trim();
                            if self.parsed.source.is_none()
                                && (label == "源视频"
                                    || label.contains("来源")
                                    || parse_seconds(label).is_some())
                            {
                                self.parsed.source = Some(href);
                            }
                        }
                    }
                    "img" if start => {
                        self.flush();
                        if let Some(reference) = attr("src") {
                            self.parsed.blocks.push(Block::Image {
                                reference,
                                alt: attr("alt").unwrap_or_default(),
                            });
                        }
                    }
                    "li" => {
                        self.flush();
                        self.list_item = start;
                    }
                    "p" | "div" | "section" | "article" | "main" | "blockquote" | "pre" | "tr"
                    | "ul" | "ol" => self.flush(),
                    "br" if start => self.buffer.push('\n'),
                    "td" | "th" if tag.kind == EndTag => self.buffer.push_str(" | "),
                    _ => {}
                }
            }
            Token::CharacterTokens(value) if self.hidden == 0 => self.buffer.push_str(&value),
            _ => {}
        }
        TokenSinkResult::Continue
    }
}
fn parse_html(value: &str) -> Parsed {
    let mut queue = BufferQueue::default();
    queue.push_back(value.into());
    let mut tokenizer = Tokenizer::new(HtmlReader::default(), TokenizerOpts::default());
    let _ = tokenizer.feed(&mut queue);
    tokenizer.end();
    tokenizer.sink.flush();
    tokenizer.sink.parsed
}
fn parse_markdown(value: &str) -> Parsed {
    let mut html = String::new();
    pulldown_cmark::html::push_html(
        &mut html,
        pulldown_cmark::Parser::new_ext(value, pulldown_cmark::Options::all()),
    );
    let mut parsed = parse_html(&html);
    // These exact lines are generated metadata, not proof that a transcript exists.
    parsed.blocks.retain(|block| match block {
        Block::Paragraph { text } => {
            let value = text.trim_start_matches("• ").trim();
            if let Some(author) = value.strip_prefix("作者：") {
                if !matches!(author, "未知" | "unknown" | "") {
                    parsed.author = Some(author.to_owned());
                }
                return false;
            }
            if let Some(duration) = value.strip_prefix("时长：").and_then(parse_seconds) {
                parsed.duration = Some(duration);
                return false;
            }
            !value.starts_with("来源：") && !value.starts_with("由 course2md 生成（")
        }
        _ => true,
    });
    parsed
}
fn parse_json(value: &str) -> Result<Parsed> {
    let value: serde_json::Value = serde_json::from_str(value)?;
    let schema = value
        .get("schema")
        .or_else(|| value.get("schema_version"))
        .and_then(|v| v.as_u64());
    anyhow::ensure!(
        schema.is_none_or(|v| v == 1),
        "这份 JSON 的结构版本暂不受支持；原文件已保留"
    );
    let mut parsed = Parsed::default();
    parsed.meta = value.get("meta").and_then(known_meta);
    parsed.summary = value
        .get("summary")
        .and_then(|s| serde_json::from_value(s.clone()).ok());
    if let Some(summary) = &parsed.summary {
        parsed.blocks.push(Block::Heading {
            text: "摘要".into(),
            level: 2,
            seconds: None,
        });
        parsed.blocks.push(Block::Paragraph {
            text: summary.tldr.clone(),
        });
        for point in &summary.key_points {
            parsed.blocks.push(Block::Paragraph {
                text: format!("• {point}"),
            });
        }
    }
    parsed.title = value["title"].as_str().map(str::to_owned);
    parsed.author = value["author"].as_str().map(str::to_owned);
    parsed.duration = value["duration"].as_f64();
    parsed.source = value["source"]["url"].as_str().map(str::to_owned);
    let sections = value["sections"]
        .as_array()
        .context("JSON 缺少可识别的笔记段落")?;
    parsed.sections = serde_json::from_value::<Vec<Section>>(value["sections"].clone()).ok();
    if let Some(sections) = &parsed.sections {
        for section in sections {
            anyhow::ensure!(
                section.t.is_finite()
                    && section.t >= 0.
                    && section.end.is_finite()
                    && section.end >= 0.,
                "JSON 中的视频时间无效，原文件已保留"
            );
            if !section.speech.is_empty() {
                execution::validate_events(&section.speech)?;
            }
        }
    }
    for section in sections {
        if let Some(t) = section["t"].as_f64().filter(|t| t.is_finite() && *t >= 0.) {
            parsed.blocks.push(Block::Heading {
                text: crate::render::fmt_ts(t),
                level: 2,
                seconds: Some(t),
            });
        }
        if let Some(reference) = section["image"].as_str().filter(|s| !s.is_empty()) {
            parsed.blocks.push(Block::Image {
                reference: reference.into(),
                alt: String::new(),
            });
        } else if let Some(id) = section["image_id"].as_str() {
            parsed.blocks.push(Block::Image {
                reference: format!("image-id:{id}"),
                alt: String::new(),
            });
        }
        let speech = section["speech"]
            .as_array()
            .context("JSON 段落缺少文字列表")?;
        for event in speech {
            if let Some(value) = event["text"].as_str().filter(|s| meaningful(s)) {
                parsed.blocks.push(Block::Paragraph { text: value.into() });
            }
        }
    }
    Ok(parsed)
}
fn readable(parsed: &Parsed) -> bool {
    parsed
        .blocks
        .iter()
        .any(|b| matches!(b, Block::Paragraph { text } if meaningful(text)))
}
fn read_resource(root: &Path, reference: &str) -> Result<(Vec<u8>, Option<String>, String)> {
    use base64::Engine as _;
    if let Some(data) = reference.strip_prefix("data:") {
        let (header, encoded) = data.split_once(',').context("图片数据地址不完整")?;
        anyhow::ensure!(
            header.ends_with(";base64") && header.starts_with("image/"),
            "图片数据地址不受支持"
        );
        anyhow::ensure!(
            encoded.len() as u64 <= RESOURCE_LIMIT * 2,
            "图片数据超过读取上限"
        );
        let bytes = base64::engine::general_purpose::STANDARD.decode(encoded)?;
        let mime = image_mime(&bytes).context("图片格式不受支持")?;
        return Ok((bytes, None, mime.into()));
    }
    let base = url::Url::from_directory_path(root).map_err(|_| anyhow::anyhow!("资料目录无效"))?;
    let url = base.join(reference)?;
    anyhow::ensure!(url.scheme() == "file", "远程图片未下载");
    let path = url
        .to_file_path()
        .map_err(|_| anyhow::anyhow!("图片文件地址无效"))?;
    let canonical = path.canonicalize()?;
    anyhow::ensure!(
        canonical.starts_with(root),
        "图片位于此资料目录之外，未读取"
    );
    let relative = canonical
        .strip_prefix(root)?
        .to_string_lossy()
        .replace('\\', "/");
    let bytes = read_bounded(&canonical, RESOURCE_LIMIT)?;
    let mime = image_mime(&bytes).context("图片格式不受支持")?;
    Ok((bytes, Some(relative), mime.into()))
}
pub fn image_mime(bytes: &[u8]) -> Option<&'static str> {
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        Some("image/png")
    } else if bytes.starts_with(&[0xff, 0xd8, 0xff]) {
        Some("image/jpeg")
    } else if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        Some("image/gif")
    } else if bytes.starts_with(b"RIFF") && bytes.get(8..12) == Some(b"WEBP") {
        Some("image/webp")
    } else {
        None
    }
}
fn extension(mime: &str) -> &str {
    match mime {
        "image/png" => "png",
        "image/gif" => "gif",
        "image/webp" => "webp",
        _ => "jpg",
    }
}
fn render_blocks(blocks: &[Block], resources: &BTreeMap<String, Resource>) -> String {
    let mut result = String::new();
    for block in blocks {
        match block {
            Block::Heading { text, level, .. } => {
                result.push_str(&format!("{} {text}\n\n", "#".repeat(*level as usize)))
            }
            Block::Paragraph { text } => result.push_str(&format!("{text}\n\n")),
            Block::Image { reference, alt } => {
                if let Some(image) = resources.get(reference) {
                    result.push_str(&format!(
                        "![{}]({})\n\n",
                        alt.replace(['[', ']'], ""),
                        image.path
                    ));
                }
            }
        }
    }
    result
}
/// Rewrites only parsed image spans; authored prose, code, lists, tables and links remain.
fn rewrite_markdown(value: &str, resources: &BTreeMap<String, Resource>) -> String {
    use pulldown_cmark::{Event, Tag, TagEnd};
    let mut edits = Vec::new();
    let mut image: Option<(std::ops::Range<usize>, String, String)> = None;
    for (event, range) in
        pulldown_cmark::Parser::new_ext(value, pulldown_cmark::Options::all()).into_offset_iter()
    {
        match event {
            Event::Start(Tag::Image { dest_url, .. }) => {
                image = Some((range, dest_url.into_string(), String::new()))
            }
            Event::Text(text) | Event::Code(text) if image.is_some() => {
                image.as_mut().unwrap().2.push_str(&text)
            }
            Event::End(TagEnd::Image) => {
                if let Some((range, reference, alt)) = image.take() {
                    let replacement = resources
                        .get(&reference)
                        .map(|r| format!("![{}]({})", alt.replace(['[', ']'], ""), r.path))
                        .unwrap_or_else(|| {
                            if alt.is_empty() {
                                String::new()
                            } else {
                                format!("（图片：{alt}，文件未包含）")
                            }
                        });
                    edits.push((range, replacement));
                }
            }
            _ => {}
        }
    }
    let mut result = value.to_owned();
    for (range, replacement) in edits.into_iter().rev() {
        result.replace_range(range, &replacement);
    }
    result
}
pub fn without_images(value: &str) -> String {
    use pulldown_cmark::{Event, Tag};
    let mut ranges = Vec::new();
    let mut definitions = Vec::new();
    for (event, range) in
        pulldown_cmark::Parser::new_ext(value, pulldown_cmark::Options::all()).into_offset_iter()
    {
        if let Event::Start(Tag::Image { id, .. }) = event {
            ranges.push(range);
            if !id.is_empty() {
                definitions.push(id.into_string());
            }
        }
    }
    let mut result = value.to_owned();
    for range in ranges.into_iter().rev() {
        result.replace_range(range, "");
    }
    result
        .lines()
        .filter(|line| {
            !definitions
                .iter()
                .any(|id| line.trim_start().starts_with(&format!("[{id}]:")))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Read only recognized exports and their in-directory image references; no writes/network.
pub fn read(dir: &Path) -> Result<Option<Note>> {
    let root = dir.canonicalize()?;
    let mut warnings = Vec::new();
    let mut files = BTreeMap::new();
    let mut originals = Vec::new();
    let mut candidates = Vec::new();
    let mut all_images = Vec::new();
    for (priority, name) in CANDIDATES.iter().enumerate() {
        let path = root.join(name);
        if !path.is_file() {
            continue;
        }
        let result = (|| -> Result<(Parsed, String)> {
            let bytes = read_bounded(&artifact::safe_asset_path(&root, name)?, TEXT_LIMIT)?;
            let content = text(&bytes);
            originals.push(Original {
                name: name.to_string(),
                path: format!("original/{name}"),
                sha256: execution::digest(&bytes),
            });
            files.insert(format!("original/{name}"), bytes);
            let content = content?;
            let parsed = match *name {
                "course.md" => parse_markdown(&content),
                "course.html" => parse_html(&content),
                _ => parse_json(&content)?,
            };
            Ok((parsed, content))
        })();
        match result {
            Ok((parsed, content)) => {
                all_images.extend(parsed.blocks.iter().filter_map(|b| {
                    if let Block::Image { reference, .. } = b {
                        Some(reference.clone())
                    } else {
                        None
                    }
                }));
                if readable(&parsed) {
                    let modified = path
                        .metadata()?
                        .modified()
                        .unwrap_or(SystemTime::UNIX_EPOCH);
                    candidates.push((
                        modified,
                        std::cmp::Reverse(priority),
                        name.to_string(),
                        parsed,
                        content,
                    ));
                }
            }
            Err(error) => warnings.push(format!("{name} 无法读取：{error:#}")),
        }
    }
    candidates.sort_by_key(|(modified, priority, ..)| (*modified, *priority));
    let Some((modified, _, primary, mut parsed, content)) = candidates.pop() else {
        anyhow::ensure!(warnings.is_empty(), "{}", warnings.join("；"));
        return Ok(None);
    };
    if !candidates.is_empty() {
        warnings.push(format!(
            "正在读取最近修改的 {primary}；其他旧文稿已原样保留。"
        ));
    }
    let mut meta = parsed
        .meta
        .take()
        .or_else(|| {
            let path = root.join("meta.json");
            read_bounded(&path, TEXT_LIMIT)
                .ok()
                .and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok())
                .and_then(|v| known_meta(&v))
        })
        .unwrap_or(VideoMeta {
            title: String::new(),
            uploader: String::new(),
            duration: 0.,
            webpage_url: String::new(),
            extractor: String::new(),
            id: String::new(),
        });
    if let Some(title) = parsed.title.filter(|v| !v.trim().is_empty()) {
        meta.title = title;
    }
    if meta.title.trim().is_empty() {
        meta.title = dir
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
    }
    if let Some(source) = parsed.source.filter(|v| !v.trim().is_empty()) {
        meta.webpage_url = source;
    }
    if let Some(author) = parsed.author {
        meta.uploader = author;
    }
    if let Some(duration) = parsed.duration.filter(|v| v.is_finite() && *v > 0.) {
        meta.duration = duration;
    }
    if matches!(meta.uploader.as_str(), "未知" | "unknown") {
        meta.uploader.clear();
    }
    if !meta.duration.is_finite() || meta.duration < 0. {
        meta.duration = 0.;
    }
    if meta.webpage_url.starts_with("file:") || Path::new(&meta.webpage_url).is_absolute() {
        meta.extractor = "local".into();
    }
    if meta.webpage_url.is_empty() {
        warnings.push("旧资料没有记录视频来源；可以阅读和导出，不会自动重新处理。".into());
    }
    for name in ["meta.json", "run.json"] {
        let path = root.join(name);
        if path.is_file() {
            if let Ok(bytes) = read_bounded(&artifact::safe_asset_path(&root, name)?, TEXT_LIMIT) {
                originals.push(Original {
                    name: name.into(),
                    path: format!("original/{name}"),
                    sha256: execution::digest(&bytes),
                });
                files.insert(format!("original/{name}"), bytes);
            }
        }
    }
    let run = files
        .get("original/run.json")
        .and_then(|b| serde_json::from_slice::<serde_json::Value>(b).ok());
    if !run.is_some_and(|v| v["success"].as_bool() == Some(true)) {
        warnings.push("旧处理记录缺失或未完成；本次仅导入已存在的正文。".into());
    }
    let mut resources = BTreeMap::new();
    all_images.sort();
    all_images.dedup();
    for reference in all_images {
        match read_resource(&root, &reference) {
            Ok((bytes, original, mime)) => {
                let sha256 = execution::digest(&bytes);
                let path = format!("assets/{sha256}.{}", extension(&mime));
                if let Some(relative) = &original {
                    files.insert(format!("original/{relative}"), bytes.clone());
                }
                files.insert(path.clone(), bytes);
                resources.insert(
                    reference.clone(),
                    Resource {
                        reference,
                        path,
                        original_path: original.map(|p| format!("original/{p}")),
                        mime,
                        sha256,
                    },
                );
            }
            Err(error) => warnings.push(format!(
                "旧图片未包含（{}）：{error:#}",
                if reference.starts_with("data:") {
                    "内嵌图片"
                } else {
                    &reference
                }
            )),
        }
        anyhow::ensure!(
            files.values().map(Vec::len).sum::<usize>() <= TOTAL_LIMIT,
            "旧资料及图片超过本次导入上限；原文件已保留"
        );
    }
    let has_html = primary == "course.md"
        && pulldown_cmark::Parser::new_ext(&content, pulldown_cmark::Options::all()).any(|event| {
            matches!(
                event,
                pulldown_cmark::Event::Html(_) | pulldown_cmark::Event::InlineHtml(_)
            )
        });
    if has_html {
        warnings
            .push("原稿包含 HTML 标记；阅读和新导出采用文字与图片结构，原稿已原样保留。".into());
    }
    let markdown = if primary == "course.md" && !has_html {
        rewrite_markdown(&content, &resources)
    } else {
        render_blocks(&parsed.blocks, &resources)
    };
    let mut sections = Vec::<Section>::new();
    let mut current = Section {
        t: 0.,
        end: 0.,
        image: String::new(),
        speech: vec![],
    };
    for block in &parsed.blocks {
        match block {
            Block::Heading {
                text,
                level,
                seconds,
            } => {
                if let Some(t) = seconds {
                    if !current.speech.is_empty() || !current.image.is_empty() {
                        current.end = *t;
                        sections.push(current);
                    }
                    current = Section {
                        t: *t,
                        end: *t,
                        image: String::new(),
                        speech: vec![],
                    };
                } else if !(*level == 1 && text == &meta.title) {
                    current.speech.push(TranscriptEvent {
                        start: current.t,
                        end: current.t,
                        text: text.clone(),
                        raw: None,
                    });
                }
            }
            Block::Paragraph { text } => current.speech.push(TranscriptEvent {
                start: current.t,
                end: current.t,
                text: text.clone(),
                raw: None,
            }),
            Block::Image { reference, .. } => {
                if let Some(resource) = resources.get(reference) {
                    if !current.image.is_empty() {
                        let t = current.t;
                        sections.push(current);
                        current = Section {
                            t,
                            end: t,
                            image: String::new(),
                            speech: vec![],
                        };
                    }
                    current.image = resource.path.clone();
                }
            }
        }
    }
    if !current.speech.is_empty() || !current.image.is_empty() {
        current.end = current.t.max(meta.duration);
        sections.push(current);
    }
    if let Some(mut original_sections) = parsed.sections.take() {
        for section in &mut original_sections {
            section.image = resources
                .get(&section.image)
                .map(|r| r.path.clone())
                .unwrap_or_default();
        }
        if artifact::has_readable_body(&original_sections) {
            sections = original_sections;
        }
    }
    let fingerprint = execution::digest(&serde_json::to_vec(&serde_json::json!({
        "importer":1,"primary":primary,"files":files.iter().map(|(path,bytes)|(path,execution::digest(bytes))).collect::<BTreeMap<_,_>>()
    }))?);
    Ok(Some(Note {
        document: Document {
            schema: 1,
            meta,
            sections,
            summary: parsed.summary,
        },
        markdown,
        modified,
        fingerprint,
        files,
        provenance: Provenance {
            schema: 1,
            primary,
            originals,
            resources: resources.into_values().collect(),
            blocks: parsed.blocks,
            warnings,
        },
    }))
}

pub fn provenance(version: &Path) -> Result<Option<Provenance>> {
    let path = version.join("legacy.json");
    if !path.is_file() {
        return Ok(None);
    }
    let value: Provenance = serde_json::from_slice(&std::fs::read(artifact::safe_asset_path(
        version,
        "legacy.json",
    )?)?)?;
    anyhow::ensure!(value.schema == 1, "旧资料导入记录版本不受支持");
    Ok(Some(value))
}
pub fn import(dir: &Path) -> Result<Option<Imported>> {
    let Some(note) = read(dir)? else {
        return Ok(None);
    };
    import_note(dir, note).map(Some)
}
pub fn import_note(dir: &Path, note: Note) -> Result<Imported> {
    let root = dir.canonicalize()?;
    let store = root.join(IMPORT_DIR);
    let owner_path = store.join("owner.json");
    let course_id = if store.exists() {
        anyhow::ensure!(
            !std::fs::symlink_metadata(&store)?.file_type().is_symlink(),
            "旧资料导入位置是符号链接，未写入"
        );
        let owner: serde_json::Value = serde_json::from_slice(
            &std::fs::read(&owner_path).context("导入位置已有其他文件，未覆盖")?,
        )?;
        anyhow::ensure!(
            owner["schema"] == 1 && owner["kind"] == "legacy_import",
            "导入位置无法验证归属，未写入"
        );
        let id = owner["course_id"].as_str().context("旧资料导入身份缺失")?;
        anyhow::ensure!(execution::valid_id(id), "旧资料导入身份无效");
        id.to_owned()
    } else {
        let id = format!(
            "legacy-{}",
            &execution::digest(root.to_string_lossy().as_bytes())[..32]
        );
        let staging = tempfile::Builder::new()
            .prefix(".legacy-init-")
            .tempdir_in(&root)?;
        crate::checkpoint::atomic_write(
            &staging.path().join("owner.json"),
            &serde_json::to_vec(
                &serde_json::json!({"schema":1,"kind":"legacy_import","course_id":id}),
            )?,
        )?;
        std::fs::rename(staging.path(), &store)?;
        artifact::sync_dir(&root)?;
        id
    };
    let _lock = crate::runtime::lock_file(&store.join(".import.lock"))?;
    let version_id = format!("import-{}", &note.fingerprint[..40]);
    let versions = store.join("versions");
    std::fs::create_dir_all(&versions)?;
    let version = versions.join(&version_id);
    let manifest = if version.exists() {
        let manifest = artifact::read_manifest(&version.join("manifest.json"))?;
        artifact::validate_version(&version, &manifest)?;
        manifest
    } else {
        let staging = tempfile::Builder::new()
            .prefix(".importing-")
            .tempdir_in(&versions)?;
        let mut files = note.files;
        files.insert(
            "document.json".into(),
            serde_json::to_vec_pretty(&note.document)?,
        );
        files.insert("course.md".into(), note.markdown.as_bytes().to_vec());
        files.insert(
            "meta.json".into(),
            serde_json::to_vec_pretty(&note.document.meta)?,
        );
        files.insert(
            "legacy.json".into(),
            serde_json::to_vec_pretty(&note.provenance)?,
        );
        let mut assets = Vec::new();
        for (path, bytes) in files {
            let target = staging.path().join(&path);
            crate::checkpoint::atomic_write(&target, &bytes)?;
            assets.push(Asset {
                path,
                bytes: bytes.len() as u64,
                sha256: execution::digest(&bytes),
            });
        }
        let frames = note
            .document
            .sections
            .iter()
            .filter(|s| !s.image.is_empty())
            .map(|s| crate::timeline::FrameEvent {
                t: s.t,
                image: s.image.clone(),
            })
            .collect::<Vec<_>>();
        let mut outcomes = Outcomes::default();
        outcomes.transcript = Outcome::succeeded();
        if !frames.is_empty() {
            outcomes.screenshots = Outcome::succeeded();
        }
        if note
            .provenance
            .warnings
            .iter()
            .any(|w| w.starts_with("旧图片未包含"))
        {
            outcomes.screenshots = Outcome {
                status: if frames.is_empty() {
                    Status::Failed
                } else {
                    Status::Partial
                },
                message: Some("部分旧图片缺失，已保留正文和原始引用。".into()),
                completed: Some(frames.len()),
                total: None,
            };
        }
        let previous = std::fs::read(store.join("current.json"))
            .ok()
            .and_then(|bytes| serde_json::from_slice::<artifact::CurrentVersion>(&bytes).ok())
            .and_then(|p| artifact::safe_asset_path(&store, &p.manifest).ok())
            .and_then(|p| artifact::read_manifest(&p).ok());
        let manifest = Manifest {
            schema: 1,
            task_id: version_id.clone(),
            course_id: course_id.clone(),
            source_id: if note.document.meta.webpage_url.is_empty() {
                format!("legacy:{course_id}")
            } else {
                format!(
                    "legacy:{}",
                    execution::digest(note.document.meta.webpage_url.as_bytes())
                )
            },
            version_id: version_id.clone(),
            title: note.document.meta.title.clone(),
            created_at_ms: note
                .modified
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            revision: previous.as_ref().map_or(1, |m| m.revision + 1),
            document: "document.json".into(),
            markdown: "course.md".into(),
            frames,
            assets,
            outputs: vec![],
            partial: matches!(
                outcomes.screenshots.status,
                Status::Failed | Status::Partial
            ),
            outcomes,
        };
        crate::checkpoint::atomic_write(
            &staging.path().join("manifest.json"),
            &serde_json::to_vec_pretty(&manifest)?,
        )?;
        artifact::validate_version(staging.path(), &manifest)?;
        artifact::sync_dir(staging.path())?;
        std::fs::rename(staging.path(), &version)?;
        artifact::sync_dir(&versions)?;
        manifest
    };
    let pointer = artifact::CurrentVersion {
        schema: 1,
        course_id,
        version_id: manifest.version_id.clone(),
        manifest: format!("versions/{}/manifest.json", manifest.version_id),
    };
    let bytes = serde_json::to_vec_pretty(&pointer)?;
    if std::fs::read(store.join("current.json")).ok().as_deref() != Some(bytes.as_slice()) {
        crate::checkpoint::atomic_write(&store.join("current.json"), &bytes)?;
        artifact::sync_dir(&store)?;
    }
    Ok(Imported {
        version_dir: version,
        manifest,
        provenance: note.provenance,
    })
}
