//! Reading navigation is bound to one immutable note version, independently of exports.
use super::*;
use crate::{notes::PreviewBlock, reader_navigation as nav, theme::*};
use gpui_component::{
    button::*,
    menu::{DropdownMenu, PopupMenuItem},
};
use std::{
    cell::RefCell,
    ops::Range,
    rc::Rc,
    sync::{Arc, atomic::AtomicBool},
};

const READER_MEASURE: Rems = rems(52.);

actions!(
    course2md_reader,
    [
        FindInNote,
        NextMatch,
        PreviousMatch,
        CloseFind,
        PreviousImage,
        NextImage,
        ZoomIn,
        ZoomOut,
        FitImage,
        CloseImage,
        NextReaderView,
        PreviousReaderView,
        ReaderPageUp,
        ReaderPageDown,
        ReaderStart,
        ReaderEnd
    ]
);

#[derive(Clone)]
struct Match {
    block: usize,
    range: Range<usize>,
}
#[derive(Clone)]
struct Frame {
    anchor: String,
    path: Option<PathBuf>,
    seconds: Option<f64>,
    caption: Option<String>,
    transcript: String,
    body_anchor: Option<String>,
    width: u32,
    height: u32,
}
#[derive(Clone)]
struct Version {
    course: Course,
    label: String,
}
#[derive(Default)]
struct ReaderData {
    frames: Vec<Frame>,
    versions: Vec<Version>,
    issues: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
enum OfflineVideo {
    #[default]
    NotRequested,
    Missing,
    Available(PathBuf),
}

#[derive(Clone)]
struct OfflineVideoRequest {
    library_root: PathBuf,
    task_id: String,
    work_dir: PathBuf,
    version_dir: PathBuf,
}

impl OfflineVideoRequest {
    fn for_version(
        course: &Course,
        task: &workspace::TaskRecord,
        library: &workspace::LibraryLocation,
    ) -> Option<Self> {
        let manifest = course.manifest.as_ref()?;
        if !task.plan.source.online
            || !task.plan.options.keep_video
            || task.id != manifest.task_id
            || task.plan.source_id != manifest.source_id
            || task.artifact.as_ref() != Some(&course.dir)
            || task.plan.library_id != library.id
        {
            return None;
        }
        Some(Self {
            library_root: library.root.clone(),
            task_id: task.id.clone(),
            work_dir: task.work_dir.clone(),
            version_dir: course.dir.clone(),
        })
    }

    /// Called only by reader workers, including the final check before opening.
    fn inspect(&self) -> OfflineVideo {
        self.checked_path()
            .map(OfflineVideo::Available)
            .unwrap_or(OfflineVideo::Missing)
    }

    fn checked_path(&self) -> Option<PathBuf> {
        let mut components = std::path::Path::new(&self.task_id).components();
        if !matches!(components.next(), Some(std::path::Component::Normal(_)))
            || components.next().is_some()
        {
            return None;
        }
        let expected_work = self
            .library_root
            .join(".course2md/work")
            .join(&self.task_id);
        if self.work_dir != expected_work {
            return None;
        }
        let root = self.library_root.canonicalize().ok()?;
        let work = self.work_dir.canonicalize().ok()?;
        if work != root.join(".course2md/work").join(&self.task_id)
            || !self.version_dir.canonicalize().ok()?.starts_with(&root)
        {
            return None;
        }
        let media = work.join("media.mp4").canonicalize().ok()?;
        let metadata = media.metadata().ok()?;
        (media.parent() == Some(work.as_path()) && metadata.is_file() && metadata.len() > 0)
            .then_some(media)
    }
}

struct ImageViewer {
    scroll: ScrollHandle,
    viewport_scroll: ScrollHandle,
    viewport_size: gpui::Size<Pixels>,
    frames: Vec<Frame>,
    index: usize,
    title: String,
    source: Option<nav::SourceTarget>,
    source_available: bool,
    version: PathBuf,
    zoom: Option<f32>,
    return_focus: Option<FocusHandle>,
    focus: FocusHandle,
}
pub(crate) struct State {
    controls_scroll: ScrollHandle,
    toc_scroll: ScrollHandle,
    view_focus: [FocusHandle; 2],
    find: Entity<InputState>,
    focus: FocusHandle,
    find_open: bool,
    info_open: bool,
    processing_details_open: bool,
    toc_open: bool,
    find_return_focus: Option<FocusHandle>,
    matches: Vec<Match>,
    match_index: usize,
    clear_find: bool,
    loaded: Option<PathBuf>,
    generation: u64,
    data_loading: bool,
    frames: Vec<Frame>,
    versions: Vec<Version>,
    issues: Vec<String>,
    pending_restore: Option<workspace::ReadingPosition>,
    restoring: bool,
    restore_generation: u64,
    last_position: Option<(String, workspace::ReadingPosition)>,
    layout: Option<(f32, f32, f32)>,
    item_layout: Rc<RefCell<nav::ReadingLayout>>,
    viewer: Option<ImageViewer>,
    exports: BTreeMap<PathBuf, PathBuf>,
    source_loading: bool,
    source: Option<nav::SourceTarget>,
    source_available: bool,
    offline_video: OfflineVideo,
    offline_opening: bool,
    _subscriptions: Vec<Subscription>,
}
impl State {
    pub(crate) fn new(window: &mut Window, cx: &mut Context<Desktop>) -> Self {
        let find = cx.new(|cx| InputState::new(window, cx).placeholder("在这份笔记中查找"));
        let subscription = cx.subscribe_in(&find, window, |this, _, event, window, cx| {
            match event {
                InputEvent::Change => {
                    this.update_reader_matches(cx);
                    this.move_reader_match(0, cx);
                }
                InputEvent::PressEnter { shift, .. } => {
                    this.move_reader_match(if *shift { -1 } else { 1 }, cx)
                }
                _ => {}
            }
            let _ = window;
        });
        cx.bind_keys([
            KeyBinding::new("cmd-f", FindInNote, Some("CourseReader")),
            KeyBinding::new("cmd-g", NextMatch, Some("CourseReader")),
            KeyBinding::new("cmd-shift-g", PreviousMatch, Some("CourseReader")),
            KeyBinding::new("escape", CloseFind, Some("CourseReader")),
            KeyBinding::new("left", PreviousImage, Some("ReaderImage")),
            KeyBinding::new("right", NextImage, Some("ReaderImage")),
            KeyBinding::new("=", ZoomIn, Some("ReaderImage")),
            KeyBinding::new("+", ZoomIn, Some("ReaderImage")),
            KeyBinding::new("-", ZoomOut, Some("ReaderImage")),
            KeyBinding::new("0", FitImage, Some("ReaderImage")),
            KeyBinding::new("escape", CloseImage, Some("ReaderImage")),
            KeyBinding::new("right", NextReaderView, Some("ReaderViews")),
            KeyBinding::new("left", PreviousReaderView, Some("ReaderViews")),
            KeyBinding::new("pageup", ReaderPageUp, Some("CourseReader && !Input")),
            KeyBinding::new("pagedown", ReaderPageDown, Some("CourseReader && !Input")),
            KeyBinding::new("home", ReaderStart, Some("CourseReader && !Input")),
            KeyBinding::new("end", ReaderEnd, Some("CourseReader && !Input")),
        ]);
        Self {
            controls_scroll: ScrollHandle::new(),
            toc_scroll: ScrollHandle::new(),
            view_focus: std::array::from_fn(|_| cx.focus_handle()),
            find,
            focus: cx.focus_handle(),
            find_open: false,
            info_open: false,
            processing_details_open: false,
            toc_open: false,
            find_return_focus: None,
            matches: Vec::new(),
            match_index: 0,
            clear_find: false,
            loaded: None,
            generation: 0,
            data_loading: false,
            frames: Vec::new(),
            versions: Vec::new(),
            issues: Vec::new(),
            pending_restore: None,
            restoring: false,
            restore_generation: 0,
            last_position: None,
            layout: None,
            item_layout: Rc::default(),
            viewer: None,
            exports: BTreeMap::new(),
            source_loading: false,
            source: None,
            source_available: false,
            offline_video: OfflineVideo::NotRequested,
            offline_opening: false,
            _subscriptions: vec![subscription],
        }
    }
}
struct ImageDialog {
    desktop: Entity<Desktop>,
    _observation: Subscription,
}
impl Render for ImageDialog {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.desktop
            .update(cx, |desktop, cx| desktop.reader_image_content(window, cx))
    }
}
fn block_text(block: &PreviewBlock) -> Option<&str> {
    match block {
        PreviewBlock::Heading { text, .. } | PreviewBlock::Paragraph { text, .. } => Some(text),
        _ => None,
    }
}
fn block_anchor(block: &PreviewBlock, index: usize) -> String {
    match block {
        PreviewBlock::Heading { anchor, .. } | PreviewBlock::Paragraph { anchor, .. } => {
            anchor.clone()
        }
        PreviewBlock::Image(path) => format!(
            "image:{index}:{}",
            path.file_name().unwrap_or_default().to_string_lossy()
        ),
    }
}
fn block_time(blocks: &[PreviewBlock], index: usize) -> Option<f64> {
    for block in blocks.iter().take(index + 1).rev() {
        // An explicitly untimed heading ends the previous time's scope.
        if let PreviewBlock::Heading { seconds, .. } = block {
            return *seconds;
        }
    }
    None
}
fn frame_label(title: &str, frame: &Frame, index: usize) -> String {
    match frame.seconds {
        Some(time) => format!("{title}，{} 的截图", course2md::render::fmt_ts(time)),
        None => format!("{title}，第 {} 张图片", index + 1),
    }
}
/// Selectable paragraph text with word-level find highlights. Selection plumbing
/// mirrors gpui_base::SelectableText (which accepts no highlight runs).
struct ReaderText {
    id: ElementId,
    handle: Option<gpui_base::TextSelectionHandle>,
    text: SharedString,
    styled_text: StyledText,
    document_order: u64,
}
impl ReaderText {
    fn new(
        id: impl Into<ElementId>,
        text: impl Into<SharedString>,
        document_order: u64,
        highlights: Vec<(Range<usize>, HighlightStyle)>,
    ) -> Self {
        let text = text.into();
        let styled = StyledText::new(text.clone());
        ReaderText {
            id: id.into(),
            handle: None,
            styled_text: if highlights.is_empty() {
                styled
            } else {
                styled.with_highlights(highlights)
            },
            text,
            document_order,
        }
    }
}
impl IntoElement for ReaderText {
    type Element = Self;
    fn into_element(self) -> Self::Element {
        self
    }
}
impl Element for ReaderText {
    type RequestLayoutState = gpui_base::TextSelectionHandle;
    type PrepaintState = Hitbox;
    fn id(&self) -> Option<ElementId> {
        Some(self.id.clone())
    }
    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }
    fn request_layout(
        &mut self,
        global_id: Option<&GlobalElementId>,
        inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let handle = self.handle.clone().unwrap_or_else(|| {
            window.with_element_state(
                global_id.expect("ReaderText must have a stable element id"),
                |retained: Option<gpui_base::TextSelectionHandle>, _| {
                    let handle = retained.unwrap_or_else(|| {
                        gpui_base::TextSelectionHandle::new(self.text.clone(), cx)
                    });
                    (handle.clone(), handle)
                },
            )
        });
        let (layout_id, ()) = self
            .styled_text
            .request_layout(global_id, inspector_id, window, cx);
        (layout_id, handle)
    }
    fn prepaint(
        &mut self,
        global_id: Option<&GlobalElementId>,
        inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        handle: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        self.styled_text
            .prepaint(global_id, inspector_id, bounds, &mut (), window, cx);
        let hitbox = window.insert_hitbox(bounds, HitboxBehavior::Normal);
        handle.register(
            gpui_base::TextSelectionRegistration::new(hitbox.clone(), bounds)
                .with_document_order(self.document_order)
                .with_text_bounds(vec![bounds]),
            window,
            cx,
        );
        hitbox
    }
    fn paint(
        &mut self,
        global_id: Option<&GlobalElementId>,
        inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        handle: &mut Self::RequestLayoutState,
        _: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let layout = self.styled_text.layout().clone();
        let selected_text_before = gpui_base::TextSelection::selected_text(window, cx);
        let projection = handle.update_runs(
            &[
                gpui_base::TextSelectionRun::new(self.text.clone(), layout.clone(), bounds)
                    .with_document_order(self.document_order),
            ],
            cx,
        );
        if selected_text_before != gpui_base::TextSelection::selected_text(window, cx) {
            window.refresh();
        }
        for range in projection.ranges().iter().flatten().cloned() {
            paint_text_selection(&layout, range, window);
        }
        self.styled_text.paint(
            global_id,
            inspector_id,
            bounds,
            &mut (),
            &mut (),
            window,
            cx,
        );
    }
}
/// Quad painting identical to gpui_base::SelectableText's selection pass.
fn paint_text_selection(layout: &gpui::TextLayout, range: Range<usize>, window: &mut Window) {
    let (Some(start), Some(end)) = (
        layout.position_for_index(range.start),
        layout.position_for_index(range.end),
    ) else {
        return;
    };
    let line_height = layout.line_height();
    let bounds = layout.bounds();
    let quads = if start.y == end.y {
        vec![Bounds::from_corners(
            start,
            Point::new(end.x, end.y + line_height),
        )]
    } else {
        let mut quads = vec![Bounds::from_corners(
            start,
            Point::new(bounds.right(), start.y + line_height),
        )];
        if end.y > start.y + line_height {
            quads.push(Bounds::from_corners(
                Point::new(bounds.left(), start.y + line_height),
                Point::new(bounds.right(), end.y),
            ));
        }
        quads.push(Bounds::from_corners(
            Point::new(bounds.left(), end.y),
            Point::new(end.x, end.y + line_height),
        ));
        quads
    };
    for quad in quads {
        window.paint_quad(PaintQuad {
            bounds: quad,
            background: color(SELECTION).into(),
            corner_radii: gpui::Corners::default(),
            border_widths: gpui::Edges::default(),
            border_color: transparent_black(),
            border_style: gpui::BorderStyle::default(),
        });
    }
}
fn paragraph(
    id: impl Into<ElementId>,
    text: String,
    order: usize,
    highlights: Vec<(Range<usize>, HighlightStyle)>,
) -> Stateful<Div> {
    let id = id.into();
    div()
        .id(id.clone())
        .role(Role::Paragraph)
        .aria_label(text.clone())
        .text_size(TEXT_READER)
        .line_height(relative(1.75))
        .child(ReaderText::new(id, text, order as u64, highlights))
}
impl Desktop {
    fn scroll_reader_page(&mut self, direction: f32, cx: &mut Context<Self>) {
        let step = self.reader_scroll.bounds().size.height * 0.85 * direction;
        let offset = self.reader_scroll.offset() + point(px(0.), step);
        self.reader_scroll.set_offset(offset);
        cx.notify();
    }
    pub fn save_library_presentation(&mut self, cx: &mut Context<Self>) {
        let mut preferences = self.application_edit_base();
        preferences.desktop.library_cards = self.desktop_settings.library_cards;
        preferences.desktop.library_group_folders = self.desktop_settings.library_group_folders;
        self.commit_application(preferences, cx);
        cx.notify();
    }
    fn reading_key(&self) -> Option<String> {
        let preview = self.preview.as_ref()?;
        Some(match &preview.course.manifest {
            Some(manifest) => format!(
                "{}:{}:{}",
                manifest.course_id, manifest.version_id, self.result_tab
            ),
            None => {
                let identity = self
                    .course_location(&preview.course)
                    .and_then(|library| {
                        preview
                            .course
                            .dir
                            .strip_prefix(&library.root)
                            .ok()
                            .map(|relative| format!("{}:{}", library.id, relative.display()))
                    })
                    .unwrap_or_else(|| preview.course.dir.display().to_string());
                format!("{identity}:{}", self.result_tab)
            }
        })
    }
    fn capture_reading_position(&self) -> Option<workspace::ReadingPosition> {
        let preview = self.preview.as_ref()?;
        let (index, within, height) = self
            .reader_ui
            .item_layout
            .borrow()
            .top_item(f32::from(self.reader_scroll.offset().y))?;
        let (paragraph, seconds) = if self.result_tab == 0 {
            (
                preview
                    .blocks
                    .get(index)
                    .map(|block| block_anchor(block, index)),
                block_time(&preview.blocks, index),
            )
        } else {
            self.reader_ui
                .frames
                .get(index)
                .map(|frame| (Some(frame.anchor.clone()), frame.seconds))
                .unwrap_or((None, None))
        };
        let fraction = Some(nav::within_fraction(within, height));
        Some(workspace::ReadingPosition {
            paragraph,
            seconds,
            offset: f32::from(self.reader_scroll.offset().y),
            within,
            fraction,
        })
    }
    pub fn save_reading_position(&mut self, cx: &mut Context<Self>) {
        if self.page != Page::Result
            || self.reading
            || self.reader_ui.restoring
            || self.reader_ui.pending_restore.is_some()
        {
            return;
        }
        let (Some(key), Some(position)) = (self.reading_key(), self.capture_reading_position())
        else {
            return;
        };
        if self.reader_ui.last_position.as_ref() == Some(&(key.clone(), position.clone())) {
            return;
        }
        if let Some(workspace) = &mut self.workspace {
            if let Err(error) = workspace.transaction(|state| {
                state.positions.insert(key.clone(), position.clone());
                Ok(())
            }) {
                self.workspace_error = Some(format!("阅读位置尚未保存：{error:#}"));
                cx.notify();
                return;
            }
        }
        self.reader_saved_offset = position.offset;
        self.reader_ui.last_position = Some((key, position));
    }
    pub fn restore_reading_position(&mut self, cx: &mut Context<Self>) {
        self.ensure_reader_data(cx);
        self.reader_ui.pending_restore = Some(
            self.reading_key()
                .and_then(|key| self.workspace.as_ref()?.state.positions.get(&key))
                .cloned()
                .unwrap_or_default(),
        );
        self.reader_ui.restore_generation += 1;
        self.reader_ui.restoring = false;
        self.reader_ui.last_position = None;
        cx.notify();
    }
    fn position_index(&self, position: &workspace::ReadingPosition) -> Option<usize> {
        if self.result_tab == 0 {
            let blocks = &self.preview.as_ref()?.blocks;
            position
                .paragraph
                .as_ref()
                .and_then(|anchor| {
                    blocks
                        .iter()
                        .enumerate()
                        .position(|(index, block)| block_anchor(block, index) == *anchor)
                        .map(|index| {
                            if anchor == "summary" {
                                blocks
                                    .iter()
                                    .position(|block| {
                                        matches!(block, PreviewBlock::Paragraph { anchor, .. }
                                if anchor == "summary-tldr" || anchor.starts_with("key-point-"))
                                    })
                                    .unwrap_or(index)
                            } else {
                                index
                            }
                        })
                })
                .or_else(|| {
                    position.seconds.and_then(|seconds| {
                        nav::nearest_time(
                            blocks.iter().enumerate().map(|(i, block)| {
                                (
                                    i,
                                    match block {
                                        PreviewBlock::Heading { seconds, .. } => *seconds,
                                        _ => None,
                                    },
                                )
                            }),
                            seconds,
                        )
                        .map(|(i, _)| i)
                    })
                })
        } else {
            position
                .paragraph
                .as_ref()
                .and_then(|anchor| {
                    self.reader_ui
                        .frames
                        .iter()
                        .position(|frame| frame.anchor == *anchor)
                })
                .or_else(|| {
                    position.seconds.and_then(|seconds| {
                        nav::nearest_time(
                            self.reader_ui
                                .frames
                                .iter()
                                .enumerate()
                                .map(|(i, frame)| (i, frame.seconds)),
                            seconds,
                        )
                        .map(|(i, _)| i)
                    })
                })
        }
    }
    fn schedule_reader_restore(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.result_tab == 1 && self.reader_ui.data_loading {
            return;
        }
        let Some(position) = self.reader_ui.pending_restore.take() else {
            return;
        };
        self.reader_ui.restoring = true;
        let generation = self.reader_ui.restore_generation;
        let key = self.reading_key();
        cx.on_next_frame(window, move |this, _, cx| {
            if this.reader_ui.restore_generation != generation || this.reading_key() != key {
                return;
            }
            let target = this
                .position_index(&position)
                .and_then(|index| {
                    this.reader_ui.item_layout.borrow().restore(
                        index,
                        position.fraction,
                        position.within,
                    )
                })
                .map(px)
                .unwrap_or_else(|| px(position.offset.min(0.)));
            let maximum = this.reader_scroll.max_offset().y;
            this.reader_scroll
                .set_offset(point(px(0.), target.max(-maximum).min(px(0.))));
            this.reader_ui.restoring = false;
            this.reader_ui.last_position = None;
            cx.notify();
        });
    }
    fn ensure_reader_data(&mut self, cx: &mut Context<Self>) {
        let Some(preview) = &self.preview else {
            return;
        };
        if self.reader_ui.loaded.as_ref() == Some(&preview.course.dir) {
            return;
        }
        self.reader_ui.loaded = Some(preview.course.dir.clone());
        self.reader_ui
            .controls_scroll
            .set_offset(point(px(0.), px(0.)));
        self.reader_ui.frames.clear();
        self.reader_ui.versions.clear();
        self.reader_ui.issues.clear();
        self.reader_ui.source = None;
        self.reader_ui.source_available = false;
        self.reader_ui.offline_video = OfflineVideo::NotRequested;
        self.reader_ui.offline_opening = false;
        self.reader_ui.matches.clear();
        self.reader_ui.find_open = false;
        self.reader_ui.info_open = false;
        self.reader_ui.processing_details_open = false;
        self.reader_ui.toc_open = false;
        self.reader_ui.clear_find = true;
        self.reader_ui.layout = None;
        self.refresh_reader_data(cx);
    }
    fn refresh_reader_data(&mut self, cx: &mut Context<Self>) {
        let Some(preview) = self.preview.clone() else {
            return;
        };
        let path = preview.course.dir.clone();
        let source = self.unmapped_reader_source();
        let offline_request = self.reader_offline_video_request();
        let locations = self
            .workspace
            .as_ref()
            .map(|workspace| {
                workspace
                    .state
                    .libraries
                    .iter()
                    .map(|library| (library.root.clone(), library.previous_roots.clone()))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        self.reader_ui.generation += 1;
        let generation = self.reader_ui.generation;
        self.reader_ui.data_loading = true;
        self.reader_ui.offline_opening = false;
        cx.spawn(async move |this, cx| {
            let (data, source, source_available, offline_video) = cx
                .background_executor()
                .spawn(async move {
                    let source = source.map(|source| match source {
                        nav::SourceTarget::Local(path) => {
                            nav::SourceTarget::Local(nav::relocated_source(&path, &locations))
                        }
                        other => other,
                    });
                    let available = source.as_ref().is_some_and(|source| match source {
                        nav::SourceTarget::Web(_) => true,
                        nav::SourceTarget::Local(path) => path.is_file(),
                    });
                    let offline_video = offline_request
                        .map(|request| request.inspect())
                        .unwrap_or_default();
                    (load_reader_data(&preview), source, available, offline_video)
                })
                .await;
            let _ = this.update(cx, |this, cx| {
                if this.reader_ui.generation != generation
                    || this
                        .preview
                        .as_ref()
                        .is_none_or(|preview| preview.course.dir != path)
                {
                    return;
                }
                if this.reader_ui.pending_restore.is_none() && !this.reader_ui.restoring {
                    this.reader_ui.pending_restore = this.capture_reading_position();
                }
                this.reader_ui.frames = data.frames;
                this.reader_ui.versions = data.versions;
                this.reader_ui.issues = data.issues;
                this.reader_ui.source = source;
                this.reader_ui.source_available = source_available;
                this.reader_ui.offline_video = offline_video;
                this.reader_ui.data_loading = false;
                this.reader_ui.layout = None;
                this.reader_ui.restore_generation += 1;
                this.reader_ui.restoring = false;
                cx.notify();
            });
        })
        .detach();
    }
    fn reader_source_key(&self) -> Option<String> {
        let preview = self.preview.as_ref()?;
        Some(
            preview
                .course
                .manifest
                .as_ref()
                .map(|m| {
                    if m.source_id.is_empty() {
                        m.course_id.clone()
                    } else {
                        m.source_id.clone()
                    }
                })
                .unwrap_or_else(|| preview.course.storage_dir().display().to_string()),
        )
    }
    fn reader_source(&self) -> Option<nav::SourceTarget> {
        self.reader_ui.source.clone()
    }
    fn reader_offline_video_request(&self) -> Option<OfflineVideoRequest> {
        let course = &self.preview.as_ref()?.course;
        let manifest = course.manifest.as_ref()?;
        let state = &self.workspace.as_ref()?.state;
        let task = state.task(&manifest.task_id)?;
        let library = state.library(&task.plan.library_id)?;
        OfflineVideoRequest::for_version(course, task, library)
    }
    fn play_reader_offline_video(&mut self, cx: &mut Context<Self>) {
        if self.reader_ui.offline_opening {
            return;
        }
        let Some(request) = self.reader_offline_video_request() else {
            return;
        };
        let version = request.version_dir.clone();
        let generation = self.reader_ui.generation;
        self.reader_ui.offline_opening = true;
        cx.spawn(async move |this, cx| {
            let status = cx
                .background_executor()
                .spawn(async move { request.inspect() })
                .await;
            let _ = this.update(cx, |this, cx| {
                if this.reader_ui.generation != generation
                    || this
                        .preview
                        .as_ref()
                        .is_none_or(|preview| preview.course.dir != version)
                {
                    return;
                }
                this.reader_ui.offline_opening = false;
                if this.page == Page::Result {
                    if let OfflineVideo::Available(path) = &status {
                        cx.open_with_system(path);
                    } else {
                        this.message =
                            Some("此版本的离线视频暂不可用，仍可打开原视频网页。".into());
                    }
                }
                this.reader_ui.offline_video = status;
                cx.notify();
            });
        })
        .detach();
        cx.notify();
    }
    fn unmapped_reader_source(&self) -> Option<nav::SourceTarget> {
        if let Some(path) = self
            .reader_source_key()
            .and_then(|key| self.workspace.as_ref()?.state.reader_sources.get(&key))
        {
            return Some(nav::SourceTarget::Local(path.clone()));
        }
        let preview = self.preview.as_ref()?;
        let raw = preview
            .document
            .as_ref()
            .map(|doc| (doc.meta.webpage_url.as_str(), doc.meta.extractor == "local"))
            .or_else(|| {
                preview
                    .metadata
                    .iter()
                    .find(|(key, _)| key == "来源")
                    .map(|(_, value)| (value.as_str(), false))
            })?;
        nav::source_target(raw.0, raw.1)
    }
    fn open_reader_source(&mut self, seconds: Option<f64>, cx: &mut Context<Self>) {
        let Some(source) = self.reader_source() else {
            return;
        };
        if let Some(url) = seconds.and_then(|time| nav::seek_url(&source, time)) {
            cx.open_url(&url);
            return;
        }
        match source {
            nav::SourceTarget::Web(url) => cx.open_url(&url),
            nav::SourceTarget::Local(path) if path.is_file() => cx.open_with_system(&path),
            nav::SourceTarget::Local(path) => {
                self.message = Some(format!(
                    "原视频不在 {}。请使用“重新定位原视频”选择它的新位置。",
                    path.display()
                ));
                cx.notify();
            }
        }
    }
    fn reader_screenshot_retry(&self) -> Option<String> {
        let manifest = self.preview.as_ref()?.course.manifest.as_ref()?;
        if !matches!(
            manifest.outcomes.screenshots.status,
            course2md::artifact::Status::Failed | course2md::artifact::Status::Partial
        ) {
            return None;
        }
        self.workspace
            .as_ref()?
            .state
            .tasks
            .iter()
            .find(|task| {
                task.id == manifest.task_id
                    && task.handled_by.is_none()
                    && !matches!(
                        task.state,
                        workspace::TaskState::Running
                            | workspace::TaskState::Pausing
                            | workspace::TaskState::Queued
                    )
                    && !task
                        .blocked
                        .iter()
                        .any(|blocked| blocked.reason == "uncertain")
                    && task.artifact.as_ref()
                        == self.preview.as_ref().map(|preview| &preview.course.dir)
            })
            .map(|task| task.id.clone())
    }
    fn relocate_reader_source(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.reader_ui.source_loading {
            return;
        }
        let Some(key) = self.reader_source_key() else {
            return;
        };
        let receiver = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: Some("选择原视频".into()),
        });
        cx.spawn_in(window, async move |this, cx| {
            let path = match receiver.await {
                Ok(Ok(Some(paths))) => paths.into_iter().next(),
                _ => None,
            };
            let Some(path) = path else {
                return;
            };
            let _ = this.update(cx, |this, cx| {
                this.reader_ui.source_loading = true;
                cx.notify();
            });
            let input = path.to_string_lossy().into_owned();
            let identity = key.clone();
            let checked = cx
                .background_executor()
                .spawn(async move {
                    let source::SourceProbe::Single(source) =
                        source::probe(input, false, Arc::new(AtomicBool::new(false)))?
                    else {
                        anyhow::bail!("请选择可读取的视频文件");
                    };
                    anyhow::ensure!(
                        !identity.starts_with("local:sha256:") || identity == source.identity,
                        "这个文件与生成笔记时的视频内容不同，请选择同一个原视频"
                    );
                    Ok::<_, anyhow::Error>(())
                })
                .await;
            let _ = this.update(cx, |this, cx| {
                this.reader_ui.source_loading = false;
                let result = checked.and_then(|_| {
                    this.workspace
                        .as_mut()
                        .ok_or_else(|| anyhow::anyhow!("课程库记录暂时不可写"))?
                        .transaction(|state| {
                            state.reader_sources.insert(key, path.clone());
                            Ok(())
                        })
                });
                this.message = Some(match result {
                    Ok(()) => {
                        this.refresh_reader_data(cx);
                        format!("已定位原视频：{}", path.display())
                    }
                    Err(error) => format!("原视频位置没有更改：{error:#}"),
                });
                cx.notify();
            });
        })
        .detach();
    }
    pub fn open_reader_find(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.preview.is_none() {
            return;
        }
        if !self.reader_ui.find_open {
            self.reader_ui.find_return_focus = window.focused(cx);
        }
        self.reader_ui.find_open = true;
        self.reader_ui
            .find
            .update(cx, |input, cx| input.focus(window, cx));
        self.update_reader_matches(cx);
        cx.notify();
    }
    fn close_reader_find(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.reader_ui.find_open = false;
        if let Some(focus) = self.reader_ui.find_return_focus.take() {
            focus.focus(window, cx);
        }
        cx.notify();
    }
    fn update_reader_matches(&mut self, cx: &mut Context<Self>) {
        let query = self.reader_ui.find.read(cx).value().to_string();
        self.reader_ui.matches = self
            .preview
            .as_ref()
            .map(|preview| {
                preview
                    .blocks
                    .iter()
                    .enumerate()
                    .flat_map(|(block, value)| {
                        block_text(value)
                            .map(|text| nav::text_matches(text, &query))
                            .unwrap_or_default()
                            .into_iter()
                            .map(move |range| Match { block, range })
                    })
                    .collect()
            })
            .unwrap_or_default();
        self.reader_ui.match_index = 0;
        cx.notify();
    }
    fn move_reader_match(&mut self, delta: isize, cx: &mut Context<Self>) {
        let count = self.reader_ui.matches.len();
        if count == 0 {
            return;
        }
        self.reader_ui.match_index =
            (self.reader_ui.match_index as isize + delta).rem_euclid(count as isize) as usize;
        let found = self.reader_ui.matches[self.reader_ui.match_index].clone();
        let fraction = self
            .preview
            .as_ref()
            .and_then(|preview| preview.blocks.get(found.block))
            .and_then(block_text)
            .map(|text| {
                text[..found.range.start].chars().count() as f32
                    / text.chars().count().max(1) as f32
            })
            .unwrap_or(0.);
        self.jump_reader_block(found.block, fraction, cx);
    }
    fn jump_reader_block(&mut self, index: usize, fraction: f32, cx: &mut Context<Self>) {
        if self.result_tab != 0 {
            self.save_reading_position(cx);
            self.result_tab = 0;
        }
        let Some(preview) = &self.preview else {
            return;
        };
        let Some(block) = preview.blocks.get(index) else {
            return;
        };
        self.reader_ui.pending_restore = Some(workspace::ReadingPosition {
            paragraph: Some(block_anchor(block, index)),
            seconds: block_time(&preview.blocks, index),
            fraction: Some(fraction),
            ..Default::default()
        });
        self.reader_ui.restore_generation += 1;
        self.reader_ui.restoring = false;
        cx.notify();
    }
    fn open_reader_version(&mut self, course: Course, cx: &mut Context<Self>) {
        self.load_reader_version(course, false, cx);
    }
    fn load_reader_version(&mut self, course: Course, refresh: bool, cx: &mut Context<Self>) {
        if self.reading
            || (!refresh
                && self
                    .preview
                    .as_ref()
                    .is_some_and(|preview| preview.course.dir == course.dir))
        {
            return;
        }
        self.save_reading_position(cx);
        let position = self.capture_reading_position();
        let tab = self.result_tab;
        self.read_generation += 1;
        let generation = self.read_generation;
        self.reading = true;
        cx.notify();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_executor()
                .spawn(async move { notes::read_preview(course) })
                .await;
            let _ = this.update(cx, |this, cx| {
                if this.read_generation != generation {
                    return;
                }
                this.reading = false;
                match result {
                    Ok(mut preview) => {
                        preview.course.title = this.course_display_title(&preview.course);
                        this.preview = Some(preview);
                        this.result_tab = tab;
                        if refresh {
                            this.reader_ui.loaded = None;
                        }
                        let has_saved_position = this.reading_key().is_some_and(|key| {
                            this.workspace.as_ref().is_some_and(|workspace| {
                                workspace.state.positions.contains_key(&key)
                            })
                        });
                        this.restore_reading_position(cx);
                        if refresh {
                            this.reader_ui.pending_restore = position.clone();
                            this.message = Some("已重新读取这份笔记。".into());
                        }
                        if !refresh
                            && !has_saved_position
                            && let Some(seconds) = position.and_then(|position| position.seconds)
                        {
                            let blocks = &this.preview.as_ref().unwrap().blocks;
                            let nearest = nav::nearest_time(
                                blocks.iter().enumerate().map(|(index, block)| {
                                    (
                                        index,
                                        match block {
                                            PreviewBlock::Heading { seconds, .. } => *seconds,
                                            _ => None,
                                        },
                                    )
                                }),
                                seconds,
                            );
                            if let Some((index, exact)) = nearest {
                                this.reader_ui.pending_restore = Some(workspace::ReadingPosition {
                                    seconds: Some(seconds),
                                    fraction: Some(0.),
                                    ..Default::default()
                                });
                                if !exact {
                                    this.message = Some(format!(
                                        "本版没有原来的 {}，已定位到最近的 {}。",
                                        course2md::render::fmt_ts(seconds),
                                        course2md::render::fmt_ts(
                                            block_time(blocks, index).unwrap()
                                        )
                                    ));
                                }
                            } else {
                                this.reader_ui.pending_restore = Some(Default::default());
                                this.message = Some("本版没有可对应的时间，已从开头显示。".into());
                            }
                        }
                    }
                    Err(error) => {
                        this.message = Some(format!(
                            "这个版本暂时无法打开：{error:#}。当前笔记仍可阅读。"
                        ))
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }
    pub fn export_note(
        &mut self,
        format: course2md::config::OutputFormat,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(preview) = &self.preview {
            self.export_course(preview.course.clone(), format, window, cx);
        }
    }
    pub fn export_course(
        &mut self,
        course: Course,
        format: course2md::config::OutputFormat,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.exporting {
            return;
        }
        let version = course.dir.clone();
        let title = self.course_display_title(&course);
        let stem = title
            .chars()
            .map(|c| if "/\\:*?\"<>|".contains(c) { '_' } else { c })
            .collect::<String>();
        let extension = match format {
            course2md::config::OutputFormat::Md => "zip",
            course2md::config::OutputFormat::Html => "html",
            course2md::config::OutputFormat::Json => "json",
        };
        let suggested = format!("{stem}.{extension}");
        let receiver = cx.prompt_for_new_path(&self.output(cx), Some(&suggested));
        cx.spawn_in(window, async move |this, cx| {
            let path = match receiver.await {
                Ok(Ok(Some(path))) => path,
                Ok(Ok(None)) => return,
                _ => {
                    let _ = this.update(cx, |this, cx| {
                        this.message = Some("无法打开导出文件选择器，请重试。".into());
                        cx.notify();
                    });
                    return;
                }
            };
            let _ = this.update(cx, |this, cx| {
                this.exporting = true;
                cx.notify();
            });
            let target = path.clone();
            let source = version.clone();
            let exported = cx
                .background_executor()
                .spawn(async move {
                    let protected = source.canonicalize()?;
                    let parent = target
                        .parent()
                        .ok_or_else(|| anyhow::anyhow!("导出位置无效"))?
                        .canonicalize()?;
                    anyhow::ensure!(
                        !parent.starts_with(&protected)
                            && !parent.ancestors().any(|ancestor| ancestor
                                .join("manifest.json")
                                .is_file()
                                && ancestor.parent().is_some_and(|parent| parent
                                    .file_name()
                                    .is_some_and(|name| name == "versions"))),
                        "请将导出文件保存到笔记版本目录以外，避免覆盖已保存的正文和图片。"
                    );
                    // The native save panel owns the explicit overwrite decision; the old file
                    // remains intact until a complete export is ready beside it.
                    if target.exists() {
                        let temp = parent.join(format!(
                            ".course2md-export-{}.{}",
                            workspace::new_id("file"),
                            extension
                        ));
                        let result =
                            course2md::portable::export(&source, format, &temp).and_then(|_| {
                                std::fs::rename(&temp, &target).map_err(anyhow::Error::from)
                            });
                        if result.is_err() {
                            let _ = std::fs::remove_file(&temp);
                        }
                        result
                    } else {
                        course2md::portable::export(&source, format, &target).map(|_| ())
                    }
                })
                .await;
            let _ = this.update(cx, |this, cx| {
                this.exporting = false;
                this.message = Some(match exported {
                    Ok(()) => {
                        this.reader_ui.exports.insert(version, path.clone());
                        format!("已导出“{title}”到 {}", path.display())
                    }
                    Err(error) => format!("导出未完成：{error:#}。笔记正文仍可阅读。"),
                });
                cx.notify();
            });
        })
        .detach();
    }
    fn select_reader_view(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        if self.result_tab != index {
            self.save_reading_position(cx);
            self.result_tab = index;
            self.restore_reading_position(cx);
        }
        self.reader_ui.view_focus[index].focus(window, cx);
        cx.notify();
    }
    pub fn reader_page(&mut self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let Some(preview) = self.preview.clone() else {
            return crate::motion::enter(
                "reader-opening",
                h_flex()
                    .items_center()
                    .gap_2()
                    .py_6()
                    .child(crate::motion::spinner("reader-opening-spinner", cx))
                    .child(theme::accessible_text("opening-note", "正在打开笔记…")),
                cx,
            );
        };
        self.ensure_reader_data(cx);
        for (index, focus) in self.reader_ui.view_focus.iter_mut().enumerate() {
            *focus = focus.clone().tab_stop(index == self.result_tab);
        }
        if self.reader_ui.clear_find {
            self.reader_ui.clear_find = false;
            self.reader_ui
                .find
                .update(cx, |input, cx| input.set_value("", window, cx));
        }
        let layout = (
            f32::from(window.bounds().size.width),
            f32::from(window.bounds().size.height),
            f32::from(window.rem_size()),
        );
        if self.reader_ui.layout.is_some_and(|old| old != layout)
            && self.reader_ui.pending_restore.is_none()
            && !self.reader_ui.restoring
        {
            self.reader_ui.pending_restore = self.capture_reading_position();
            self.reader_ui.restore_generation += 1;
        }
        self.reader_ui.layout = Some(layout);
        self.schedule_reader_restore(window, cx);
        let rem_size = f32::from(window.rem_size());
        let content_width = crate::views::shell_content_width(Page::Result, window);
        let available_height = (layout.1 - rem_size * (40. / 14.) - 64.).max(0.);
        let compact = content_width < rem_size * 60. || available_height < rem_size * 36.;
        let root = v_flex()
            .id("reader-page")
            .track_focus(&self.reader_ui.focus)
            .key_context("CourseReader")
            .on_action(
                cx.listener(|this, _: &FindInNote, window, cx| this.open_reader_find(window, cx)),
            )
            .on_action(cx.listener(|this, _: &NextMatch, _, cx| this.move_reader_match(1, cx)))
            .on_action(cx.listener(|this, _: &PreviousMatch, _, cx| this.move_reader_match(-1, cx)))
            .on_action(cx.listener(|this, _: &ReaderPageUp, _, cx| this.scroll_reader_page(1., cx)))
            .on_action(
                cx.listener(|this, _: &ReaderPageDown, _, cx| this.scroll_reader_page(-1., cx)),
            )
            .on_action(cx.listener(|this, _: &ReaderStart, _, cx| {
                this.reader_scroll.set_offset(point(px(0.), px(0.)));
                cx.notify();
            }))
            .on_action(cx.listener(|this, _: &ReaderEnd, _, cx| {
                this.reader_scroll.scroll_to_bottom();
                cx.notify();
            }))
            .on_action(cx.listener(|this, _: &CloseFind, window, cx| {
                if this.reader_ui.find_open {
                    this.close_reader_find(window, cx);
                } else {
                    cx.propagate();
                }
            }))
            .h_full()
            .min_h_0()
            .w_full()
            .gap_3()
            .when(compact, |view| view.gap_2());
        // Ordinary windows retain the full header. In a compact window the
        // title and tools stay visible; only explicitly opened details scroll.
        let controls_scroll = self.reader_ui.controls_scroll.clone();
        let reveal = |id: &'static str, child: AnyElement| {
            if compact {
                child
            } else {
                crate::focus_scroll::RevealFocus::new(id, child, controls_scroll.clone())
                    .inline()
                    .into_any_element()
            }
        };
        let mut page = v_flex()
            .id("reader-controls")
            .w_full()
            .min_w_0()
            .gap_3()
            .flex_shrink_0()
            .when(compact, |view| view.gap_1())
            .when(!compact, |view| {
                view.max_h(relative(0.45))
                    .overflow_y_scroll()
                    .track_scroll(&controls_scroll)
            });
        // Keep navigation with the title and group provenance on one row.
        // Generation details stay available without a second reading header.
        let outline: Vec<(f64, String)> = preview
            .document
            .as_ref()
            .and_then(|document| document.summary.as_ref())
            .map(|summary| {
                summary
                    .outline
                    .iter()
                    .map(|item| (item.t, item.title.clone()))
                    .collect()
            })
            .unwrap_or_default();
        let chapter_title = |seconds: Option<f64>| -> Option<String> {
            let seconds = seconds?;
            outline
                .iter()
                .find(|(time, _)| (time - seconds).abs() < 1.5)
                .map(|(_, title)| title.clone())
        };
        let headings: Vec<(usize, String, Option<f64>)> = preview
            .blocks
            .iter()
            .enumerate()
            .filter_map(|(index, block)| {
                if let PreviewBlock::Heading { text, seconds, .. } = block {
                    Some((
                        index,
                        chapter_title(*seconds).unwrap_or_else(|| text.clone()),
                        *seconds,
                    ))
                } else {
                    None
                }
            })
            .collect();
        let return_label = match self.result_origin {
            Page::New => "工作台",
            Page::Task => "任务",
            Page::Settings => "设置",
            _ => "我的笔记",
        };
        page = page.child(
            h_flex()
                .gap_3()
                .items_start()
                .when(compact, |row| row.items_center())
                .child(reveal(
                    "reveal-reader-back",
                    (quiet("reader-back")
                        .icon(IconName::ArrowLeft)
                        .accessibility_label(format!("返回{return_label}"))
                        .tooltip(format!("返回{return_label}"))
                        .when(!compact, |button| button.label(return_label))
                        .when(compact, |button| {
                            button.w(CONTROL_HEIGHT).px_0()
                        })
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.navigate(this.result_origin, cx)
                        })))
                    .into_any_element(),
                ))
                .child(
                    theme::accessible_text("reader-title", preview.course.title.clone())
                        .role(Role::Heading)
                        .flex_1()
                        .min_w_0()
                        .when(!compact, |title| {
                            title.whitespace_normal().text_size(TEXT_DISPLAY)
                        })
                        .when(compact, |title| {
                            title.whitespace_nowrap().text_ellipsis().text_size(TEXT_TITLE)
                        })
                        .font_weight(FontWeight::SEMIBOLD),
                ),
        );
        // Meta line 1: 作者 · 时长 · 平台 BV 号 · 打开原视频；无数据的字段省略。
        let mut facts: Vec<String> = Vec::new();
        for (label, value) in &preview.metadata {
            if label == "来源" || value.trim().is_empty() {
                continue;
            }
            facts.push(if label == "原稿" {
                format!("{label}：{value}")
            } else {
                value.clone()
            });
        }
        if let Some(document) = &preview.document {
            let platform = match document.meta.extractor.as_str() {
                "local" => "本地视频".to_owned(),
                "bilibili" => "Bilibili".to_owned(),
                "youtube" => "YouTube".to_owned(),
                "" => String::new(),
                other => {
                    let mut chars = other.chars();
                    let name = chars
                        .next()
                        .map(|first| first.to_uppercase().collect::<String>())
                        .unwrap_or_default()
                        + chars.as_str();
                    name
                }
            };
            if !platform.is_empty() {
                facts.push(platform);
            }
        }
        let mut meta_facts = h_flex().gap_2().flex_wrap().items_baseline();
        if !facts.is_empty() {
            meta_facts = meta_facts.child(
                theme::accessible_text("reader-facts", facts.join(" · "))
                    .text_size(TEXT_AUX)
                    .text_color(color(GRAY)),
            );
        }
        if let Some(source) = self.reader_source() {
            match &source {
                nav::SourceTarget::Web(url) => {
                    meta_facts = meta_facts.child(reveal(
                        "reveal-reader-source",
                        (quiet("reader-source")
                            .icon(icons::external_link())
                            .label("打开原视频")
                            .min_h(rems(1.6))
                            .accessibility_label(format!("打开原视频：{url}"))
                            .on_click(
                                cx.listener(|this, _, _, cx| this.open_reader_source(None, cx)),
                            ))
                        .into_any_element(),
                    ));
                }
                nav::SourceTarget::Local(path) => {
                    let exists = self.reader_ui.source_available;
                    meta_facts = meta_facts
                        .child(
                            theme::accessible_text(
                                "local-video-path",
                                format!(
                                    "原视频：{}",
                                    path.file_name().unwrap_or_default().to_string_lossy()
                                ),
                            )
                            .text_size(TEXT_AUX)
                            .text_color(color(GRAY))
                            .min_w_0(),
                        )
                        .when(exists, |row| {
                            row.child(reveal(
                                "reveal-reader-source",
                                (quiet("reader-source")
                                    .icon(icons::movie())
                                    .label("打开原视频")
                                    .tooltip(path.display().to_string())
                                    .min_h(rems(1.6))
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.open_reader_source(None, cx)
                                    })))
                                .into_any_element(),
                            ))
                        })
                        .when(!exists, |row| {
                            row.child(
                                theme::accessible_text(
                                    "missing-original-video",
                                    "原视频已移动或暂时不可访问；笔记仍可阅读。",
                                )
                                .text_size(TEXT_AUX)
                                .text_color(color(GRAY)),
                            )
                        })
                        .child(reveal(
                            "reveal-relocate-reader-source",
                            (quiet("relocate-reader-source")
                                .icon(IconName::FolderOpen)
                                .label(if self.reader_ui.source_loading {
                                    "正在核对视频…"
                                } else {
                                    "重新定位原视频"
                                })
                                .min_h(rems(1.6))
                                .loading(self.reader_ui.source_loading)
                                .disabled(self.reader_ui.source_loading)
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.relocate_reader_source(window, cx)
                                })))
                            .into_any_element(),
                        ));
                }
            }
        }
        match &self.reader_ui.offline_video {
            OfflineVideo::Available(_) => {
                meta_facts = meta_facts.child(reveal(
                    "reveal-reader-offline-video",
                    quiet("reader-offline-video")
                        .icon(icons::movie())
                        .label("播放离线视频")
                        .loading(self.reader_ui.offline_opening)
                        .disabled(self.reader_ui.offline_opening)
                        .on_click(cx.listener(|this, _, _, cx| this.play_reader_offline_video(cx)))
                        .into_any_element(),
                ));
            }
            OfflineVideo::Missing => {
                meta_facts = meta_facts
                    .child(
                        theme::accessible_text(
                            "reader-offline-video-missing",
                            "此版本的离线视频暂不可用，仍可打开原视频网页。",
                        )
                        .text_size(TEXT_AUX)
                        .text_color(color(MUTED)),
                    )
                    .child(reveal(
                        "reveal-retry-reader-offline-video",
                        quiet("retry-reader-offline-video")
                            .icon(icons::refresh())
                            .label("重试播放离线视频")
                            .loading(self.reader_ui.offline_opening)
                            .disabled(self.reader_ui.offline_opening)
                            .on_click(
                                cx.listener(|this, _, _, cx| this.play_reader_offline_video(cx)),
                            )
                            .into_any_element(),
                    ));
            }
            OfflineVideo::NotRequested => {}
        }
        let mut information_action = None;
        // A single version needs no selector. Keep its revision and date in
        // generation information; multiple versions retain a visible choice.
        if let Some(manifest) = &preview.course.manifest {
            meta_facts = meta_facts.child(div().flex_1());
            if self.reader_ui.versions.len() > 1 {
                let weak = cx.weak_entity();
                let versions = self.reader_ui.versions.clone();
                let current = preview.course.dir.clone();
                meta_facts = meta_facts.child(reveal(
                    "reveal-choose-note-version",
                    (quiet("choose-note-version")
                        .icon(icons::history())
                        .label(format!("版本 {}", manifest.revision))
                        .tooltip("选择笔记版本")
                        .child(Icon::new(IconName::ChevronDown).size_4().flex_shrink_0())
                        .min_h(rems(1.6))
                        .disabled(self.reading)
                        .dropdown_menu(move |menu, _, _| {
                            let mut menu = menu;
                            for version in &versions {
                                let weak = weak.clone();
                                let course = version.course.clone();
                                let label = if course.dir == current {
                                    format!("{}（当前）", version.label)
                                } else {
                                    version.label.clone()
                                };
                                menu = menu.item(
                                    PopupMenuItem::new(label)
                                        .icon(icons::history())
                                        .checked(course.dir == current)
                                        .on_click(move |_, _, cx| {
                                            let _ = weak.update(cx, |this, cx| {
                                                this.open_reader_version(course.clone(), cx)
                                            });
                                        }),
                                );
                            }
                            menu
                        }))
                    .into_any_element(),
                ));
            }
        }
        if preview.course.manifest.is_some() || compact {
            let information = reveal(
                "reveal-reader-information",
                (quiet("reader-information")
                    .icon(IconName::Info)
                    .accessibility_label(if self.reader_ui.info_open {
                        "收起生成信息"
                    } else {
                        "生成信息"
                    })
                    .tooltip("生成信息")
                    .when(!compact, |button| {
                        button.label(if self.reader_ui.info_open {
                            "收起生成信息"
                        } else {
                            "生成信息"
                        })
                    })
                    .when(compact, |button| button.w(CONTROL_HEIGHT).px_0())
                    .min_h(rems(1.6))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.reader_ui.info_open = !this.reader_ui.info_open;
                        cx.notify();
                    })))
                .into_any_element(),
            );
            if compact {
                information_action = Some(information);
            } else {
                meta_facts = meta_facts.child(information);
            }
        }
        let mut compact_metadata = None;
        if !facts.is_empty()
            || self.reader_source().is_some()
            || self.reader_ui.offline_video != OfflineVideo::NotRequested
            || preview.course.manifest.is_some()
        {
            if compact {
                compact_metadata = Some(meta_facts.into_any_element());
            } else {
                page = page.child(meta_facts);
            }
        }
        let mut toolbar = h_flex().gap_2().flex_wrap().items_center().child(
            SingleChoiceGroup::new("reader-tabs", "阅读视图")
                .tabs()
                .options([("note", "笔记"), ("images", "截图")])
                .when(!compact, |choices| {
                    choices.icon("note", icons::article()).icon("images", icons::image())
                })
                .focus_handles(self.reader_ui.view_focus.iter().cloned())
                .selected(if self.result_tab == 0 {
                    "note"
                } else {
                    "images"
                })
                .on_change(cx.listener(|this, selected: &SharedString, window, cx| {
                    this.select_reader_view(usize::from(selected.as_ref() == "images"), window, cx);
                })),
        );
        toolbar = toolbar.child(div().flex_1());
        toolbar = toolbar.child(reveal(
            "reveal-find-note",
            (quiet("find-note")
                .icon(icons::search())
                .when(!compact, |button| button.label("查找"))
                .when(compact, |button| button.w(CONTROL_HEIGHT).px_0())
                .tooltip("查找 · Command F")
                .accessibility_label("在笔记中查找，Command F")
                .on_click(cx.listener(|this, _, window, cx| this.open_reader_find(window, cx))))
            .into_any_element(),
        ));
        if !headings.is_empty() {
            let toc_on = self.reader_ui.toc_open;
            toolbar = toolbar.child(reveal(
                "reveal-note-contents",
                (control("note-contents")
                    .px(px(12.))
                    .bg(color(if toc_on { BADGE_PROGRESS_BG } else { SURFACE }))
                    .border_color(color(CONTROL))
                    .when(toc_on, |button| button.font_weight(FontWeight::SEMIBOLD))
                    .accessibility_label(if toc_on {
                        "目录：开"
                    } else {
                        "目录：关"
                    })
                    .icon(icons::toc())
                    .when(!compact, |button| button.label("目录"))
                    .when(compact, |button| button.w(CONTROL_HEIGHT).px_0())
                    .tooltip("目录")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.reader_ui.pending_restore = this.capture_reading_position();
                        this.reader_ui.restore_generation += 1;
                        this.reader_ui.toc_open = !this.reader_ui.toc_open;
                        cx.notify();
                    })))
                .into_any_element(),
            ));
        }
        let copy_weak = cx.entity().downgrade();
        toolbar = toolbar.child(reveal(
            "reveal-copy-note",
            (outline_pill("copy-note")
                .icon(icons::content_copy())
                .accessibility_label("复制笔记")
                .tooltip("复制笔记")
                .when(!compact, |button| {
                    button.label("复制纯文本")
                        .child(Icon::new(IconName::ChevronDown).size_4().flex_shrink_0())
                })
                .when(compact, |button| button.w(CONTROL_HEIGHT).px_0())
                .dropdown_menu(move |menu, _, _| {
                    let plain = copy_weak.clone();
                    let markdown = copy_weak.clone();
                    menu.item(
                        PopupMenuItem::new("复制纯文本")
                            .icon(icons::content_copy())
                            .on_click(move |_, _, cx| {
                                let _ = plain.update(cx, |this, cx| {
                                    if let Some(preview) = &this.preview {
                                        cx.write_to_clipboard(ClipboardItem::new_string(
                                            preview.plain_text.clone(),
                                        ));
                                        this.message = Some("已复制纯文本，不含图片。".into());
                                        cx.notify();
                                    }
                                });
                            }),
                    )
                    .item(
                        PopupMenuItem::new("复制 Markdown 文本")
                            .icon(icons::article())
                            .on_click(move |_, _, cx| {
                                let _ = markdown.update(cx, |this, cx| {
                                    if let Some(preview) = &this.preview {
                                        cx.write_to_clipboard(ClipboardItem::new_string(
                                            preview.markdown_text.clone(),
                                        ));
                                        this.message =
                                            Some("已复制 Markdown 文本，不含图片引用。".into());
                                        cx.notify();
                                    }
                                });
                            }),
                    )
                }))
            .into_any_element(),
        ));
        let export_weak = cx.entity().downgrade();
        toolbar = toolbar.child(reveal(
            "reveal-export-note",
            (outline_pill("export-note")
                .icon(icons::download())
                .accessibility_label(if self.exporting {
                    "正在导出…"
                } else {
                    "导出"
                })
                .tooltip("导出")
                .when(!compact, |button| {
                    button.label(if self.exporting { "正在导出…" } else { "导出" })
                        .child(Icon::new(IconName::ChevronDown).size_4().flex_shrink_0())
                })
                .when(compact, |button| button.w(CONTROL_HEIGHT).px_0())
                .loading(self.exporting)
                .disabled(self.exporting || self.preview.is_none())
                .dropdown_menu(move |menu, _, _| {
                    let mut menu = menu;
                    for (format, label) in [
                        (course2md::config::OutputFormat::Md, "Markdown 包（含图片）"),
                        (
                            course2md::config::OutputFormat::Html,
                            "网页文件（可独立阅读）",
                        ),
                        (
                            course2md::config::OutputFormat::Json,
                            "JSON 数据（不含图片文件）",
                        ),
                    ] {
                        let weak = export_weak.clone();
                        menu =
                            menu.item(PopupMenuItem::new(label).icon(icons::download()).on_click(
                                move |_, window, cx| {
                                    let _ = weak.update(cx, |this, cx| {
                                        this.export_note(format, window, cx)
                                    });
                                },
                            ));
                    }
                    menu
                }))
            .into_any_element(),
        ));
        let files_action = reveal(
            "reveal-note-files",
            (control("note-files")
                .ghost()
                .icon(IconName::FolderOpen)
                .accessibility_label("在文件夹中显示这份笔记")
                .tooltip("显示笔记文件")
                .on_click(cx.listener(|this, _, _, cx| {
                    if let Some(preview) = &this.preview {
                        cx.reveal_path(&preview.course.dir);
                    }
                })))
            .into_any_element(),
        );
        if compact {
            compact_metadata = Some(
                v_flex().gap_2().children(compact_metadata).child(files_action).into_any_element(),
            );
        } else {
            toolbar = toolbar.child(files_action);
        }
        toolbar = toolbar.children(information_action);
        page = page.child(
            toolbar.when(!compact, |toolbar| toolbar.py_2())
                .border_b_1().border_color(color(HAIRLINE)),
        );
        let (fixed_header, mut page) = if compact {
            (
                Some(page.into_any_element()),
                v_flex()
                    .id("reader-extra-controls")
                    .w_full()
                    .min_w_0()
                    .min_h_0()
                    .flex_shrink_0()
                    .gap_2()
                    .max_h(px(available_height * 0.28))
                    .overflow_y_scroll()
                    .track_scroll(&controls_scroll),
            )
        } else {
            (None, page)
        };
        let reveal = |id: &'static str, child: AnyElement| {
            crate::focus_scroll::RevealFocus::new(id, child, controls_scroll.clone()).inline()
        };
        if self.reading {
            page = page.child(crate::motion::enter(
                "reader-version-loading",
                h_flex()
                    .items_center()
                    .gap_2()
                    .child(crate::motion::spinner("reader-version-spinner", cx))
                    .child(theme::accessible_text(
                        "reader-version-loading-label",
                        "正在打开笔记…",
                    )),
                cx,
            ));
        }
        let mut details = Vec::new();
        if let Some(manifest) = &preview.course.manifest {
            let stamp = nav::timestamp_local(manifest.created_at_ms);
            let date = stamp.get(..10).unwrap_or(&stamp);
            details.push(format!("版本 {} · {date} 生成", manifest.revision));
            for (name, outcome) in [
                ("文字", &manifest.outcomes.transcript),
                ("截图", &manifest.outcomes.screenshots),
                ("AI 校对", &manifest.outcomes.proofreading),
                ("摘要", &manifest.outcomes.summary),
            ] {
                use course2md::artifact::Status;
                details.push(format!(
                    "{name}：{}",
                    match outcome.status {
                        Status::NotRequested => "未使用",
                        Status::Succeeded => "已完成",
                        Status::Partial => "部分完成",
                        Status::Failed => "未完成",
                    }
                ));
            }
            if !manifest.source_id.is_empty() {
                details.push(format!("来源身份：{}", manifest.source_id));
            }
            if let Some(task) = self.workspace.as_ref().and_then(|workspace| {
                workspace
                    .state
                    .tasks
                    .iter()
                    .find(|task| task.id == manifest.task_id)
            }) {
                if let Some(subtitle) = &task.plan.source.selected_subtitle {
                    details.push(format!("提交时选择的字幕：{}", subtitle.label));
                }
                if task.plan.config.defaults.transcript_source
                    == Some(course2md::config::TranscriptSource::Asr)
                {
                    if let Some(model) = &task.plan.config.defaults.asr_model {
                        details.push(format!("语音识别模型：{model}"));
                    }
                    if let Some(provider) = task.plan.config.defaults.provider {
                        details.push(format!("识别设备：{}", provider.as_str()));
                    }
                }
                if task.plan.options.llm || task.plan.options.summarize {
                    details.push(format!("AI 模型：{}", task.plan.config.llm.model));
                }
            }
        }
        if !details.is_empty() || compact_metadata.is_some() {
            page = page.child(disclosure(
                "reader-information-panel",
                self.reader_ui.info_open,
                v_flex()
                    .gap_2()
                    .p_4()
                    .bg(color(INSET))
                    .border_1()
                    .border_color(color(HAIRLINE))
                    .rounded(RADIUS_CARD)
                    .children(compact_metadata)
                    .children(details.into_iter().enumerate().map(|(index, value)| {
                        theme::accessible_text(("reader-info", index), value).text_sm()
                    })),
                window,
                cx,
            ));
        }
        if let Some(current) = &preview.course.manifest {
            if let Some(newer) = self
                .courses
                .iter()
                .find(|course| {
                    course.manifest.as_ref().is_some_and(|m| {
                        m.course_id == current.course_id && m.revision > current.revision
                    })
                })
                .cloned()
            {
                page = page.child(crate::motion::enter(
                    "reader-new-version-notice",
                    h_flex()
                        .gap_2()
                        .items_center()
                        .flex_wrap()
                        .p_3()
                        .bg(color(ACCENT_SOFT))
                        .rounded(RADIUS_CARD)
                        .child(
                            icons::history()
                                .size(px(18.))
                                .text_color(color(ACCENT_STRONG)),
                        )
                        .child(theme::accessible_text(
                            "newer-note-version",
                            "这份笔记已有新版。",
                        ))
                        .child(reveal(
                            "reveal-open-newer-version",
                            (outline_pill("open-newer-version")
                                .icon(icons::history())
                                .label("查看新版")
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.open_reader_version(newer.clone(), cx)
                                })))
                            .into_any_element(),
                        )),
                    cx,
                ));
            }
        }
        if self.reader_ui.find_open {
            let query = self.reader_ui.find.read(cx).value().to_string();
            let count = self.reader_ui.matches.len();
            let status = if query.trim().is_empty() {
                None
            } else if count == 0 {
                Some("未找到匹配内容".into())
            } else {
                Some(format!(
                    "第 {} 处，共 {count} 处",
                    self.reader_ui.match_index + 1
                ))
            };
            let mut find_panel = v_flex()
                .gap_2()
                .p_3()
                .bg(color(INSET))
                .rounded(RADIUS_CARD)
                .child(
                    h_flex()
                        .items_center()
                        .gap_2()
                        .bg(color(SURFACE))
                        .border_1()
                        .border_color(color(CONTROL))
                        .rounded(RADIUS_SMALL)
                        .px(px(12.))
                        .py(px(6.))
                        .min_h(rems(2.571))
                        .child(
                            icons::search()
                                .size(rems(18. / 14.))
                                .text_color(color(GRAY)),
                        )
                        .child(
                            div().min_w(rems(6.)).flex_1().child(reveal(
                                "reveal-reader-find",
                                (Input::new(&self.reader_ui.find)
                                    .aria_label("在这份笔记中查找")
                                    .appearance(false)
                                    .w_full()
                                    .h_auto()
                                    .min_h(rems(1.6)))
                                .into_any_element(),
                            )),
                        )
                        .when_some(status, |row, status: String| {
                            row.child(
                                theme::accessible_text("find-status", status)
                                    .text_size(TEXT_AUX)
                                    .text_color(color(GRAY))
                                    .flex_shrink_0(),
                            )
                        })
                        .child(reveal(
                            "reveal-previous-match",
                            (control("previous-match")
                                .ghost()
                                .rounded_full()
                                .icon(IconName::ArrowUp)
                                .accessibility_label("上一处")
                                .disabled(count == 0)
                                .on_click(
                                    cx.listener(|this, _, _, cx| this.move_reader_match(-1, cx)),
                                ))
                            .into_any_element(),
                        ))
                        .child(reveal(
                            "reveal-next-match",
                            (control("next-match")
                                .ghost()
                                .rounded_full()
                                .icon(IconName::ArrowDown)
                                .accessibility_label("下一处")
                                .disabled(count == 0)
                                .on_click(
                                    cx.listener(|this, _, _, cx| this.move_reader_match(1, cx)),
                                ))
                            .into_any_element(),
                        ))
                        .child(reveal(
                            "reveal-close-find",
                            (control("close-find")
                                .ghost()
                                .rounded_full()
                                .icon(IconName::Close)
                                .accessibility_label("关闭查找")
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.close_reader_find(window, cx)
                                })))
                            .into_any_element(),
                        )),
                );
            if let Some(found) = self
                .reader_ui
                .matches
                .get(self.reader_ui.match_index)
                .filter(|_| !query.trim().is_empty())
            {
                if let Some(text) = preview.blocks.get(found.block).and_then(block_text) {
                    let before = text[..found.range.start]
                        .chars()
                        .rev()
                        .take(32)
                        .collect::<Vec<_>>()
                        .into_iter()
                        .rev()
                        .collect::<String>();
                    let after = text[found.range.end..].chars().take(64).collect::<String>();
                    find_panel = find_panel.child(
                        theme::accessible_text(
                            "find-excerpt",
                            format!(
                                "匹配内容：{before}【{}】{after}",
                                &text[found.range.clone()]
                            ),
                        )
                        .text_sm(),
                    );
                }
            }
            page = page.child(crate::motion::enter("reader-find-panel", find_panel, cx));
        }
        if let Some(notice) = processing_notice(&preview.processing_issues) {
            let mut problem = v_flex()
                .gap_3()
                .p_4()
                .bg(color(WARNING_BG))
                .border_1()
                .border_color(color(WARNING_BG))
                .rounded(RADIUS_CARD)
                .child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .flex_wrap()
                        .child(badge(BadgeKind::Warning).child("需要处理"))
                        .child(
                            theme::accessible_text("reader-incomplete-processing", notice)
                                .text_sm(),
                        ),
                );
            let task_id = preview
                .course
                .manifest
                .as_ref()
                .map(|manifest| manifest.task_id.clone())
                .filter(|id| {
                    self.workspace.as_ref().is_some_and(|workspace| {
                        workspace.state.tasks.iter().any(|task| &task.id == id)
                    })
                });
            let mut actions = h_flex().gap_2().flex_wrap();
            if let Some(id) = task_id {
                actions = actions.child(reveal(
                    "reveal-reader-view-incomplete-task",
                    (outline_pill("reader-view-incomplete-task")
                        .icon(icons::task())
                        .label("查看任务")
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.save_reading_position(cx);
                            this.select_task(&id, cx);
                            this.page = Page::Task;
                            cx.notify();
                        })))
                    .into_any_element(),
                ));
            }
            let has_details = preview.processing_issues.iter().any(|issue| {
                issue
                    .outcome
                    .message
                    .as_ref()
                    .is_some_and(|detail| !detail.trim().is_empty())
            });
            if has_details {
                actions = actions.child(reveal(
                    "reveal-reader-processing-details",
                    (quiet("reader-processing-details")
                        .icon(IconName::Info)
                        .label(if self.reader_ui.processing_details_open {
                            "收起处理详情"
                        } else {
                            "查看处理详情"
                        })
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.reader_ui.processing_details_open =
                                !this.reader_ui.processing_details_open;
                            cx.notify();
                        })))
                    .into_any_element(),
                ));
            }
            problem = problem.child(actions);
            let mut processing_details = v_flex().gap_2();
            for (index, issue) in preview.processing_issues.iter().enumerate() {
                if let Some(detail) = issue
                    .outcome
                    .message
                    .as_ref()
                    .filter(|detail| !detail.trim().is_empty())
                {
                    processing_details = processing_details
                        .child(
                            theme::accessible_text(
                                ("reader-processing-detail-label", index),
                                format!("{}的技术详情", issue.stage.label()),
                            )
                            .text_sm()
                            .font_weight(FontWeight::MEDIUM),
                        )
                        .child(
                            theme::accessible_text(
                                ("reader-processing-detail", index),
                                detail.clone(),
                            )
                            .text_sm()
                            .text_color(color(MUTED)),
                        );
                }
            }
            problem = problem.child(disclosure(
                "reader-processing-detail-panel",
                self.reader_ui.processing_details_open,
                processing_details,
                window,
                cx,
            ));
            page = page.child(crate::motion::enter(
                "reader-processing-notice",
                problem,
                cx,
            ));
        }
        if files_need_reload(&preview, &self.reader_ui.issues, &self.reader_ui.frames) {
            let course = preview.course.clone();
            let issues = preview
                .issues
                .iter()
                .chain(self.reader_ui.issues.iter())
                .collect::<Vec<_>>();
            let messages = v_flex()
                .flex_1()
                .min_w(px(240.))
                .gap_2()
                .when(issues.is_empty(), |view| {
                    view.child(
                        theme::accessible_text(
                            "reader-unreadable-images",
                            "部分截图暂时无法读取，笔记正文仍可阅读。",
                        )
                        .text_sm()
                        .text_color(color(WARNING)),
                    )
                })
                .children(issues.into_iter().enumerate().map(|(index, issue)| {
                    theme::accessible_text(("reader-issue", index), issue.clone())
                        .text_sm()
                        .text_color(color(WARNING))
                }));
            page = page.child(crate::motion::enter(
                "reader-file-notice",
                h_flex()
                    .w_full()
                    .min_w_0()
                    .items_center()
                    .flex_wrap()
                    .gap_3()
                    .p_3()
                    .bg(color(WARNING_BG))
                    .rounded(RADIUS_CARD)
                    .child(
                        icons::warning()
                            .size(px(20.))
                            .flex_shrink_0()
                            .text_color(color(WARNING)),
                    )
                    .child(messages)
                    .child(reveal(
                        "reveal-reload-reader-files",
                        (outline_pill("reload-reader-files")
                            .icon(icons::refresh())
                            .label("重新读取本版文件")
                            .loading(self.reading)
                            .disabled(self.reading)
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.load_reader_version(course.clone(), true, cx)
                            })))
                        .into_any_element(),
                    )),
                cx,
            ));
        }
        if let Some(task) = self.reader_screenshot_retry() {
            page = page.child(reveal(
                "reveal-reader-retry-screenshots",
                (outline_pill("reader-retry-screenshots")
                    .icon(icons::image())
                    .label("仅补截图")
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.save_reading_position(cx);
                        this.reprocess_task(
                            task.clone(),
                            vec!["screenshots".into()],
                            Vec::new(),
                            cx,
                        );
                    })))
                .into_any_element(),
            ));
        }
        if let Some(path) = self.reader_ui.exports.get(&preview.course.dir).cloned() {
            let label = format!(
                "已导出：{}",
                path.file_name().unwrap_or_default().to_string_lossy()
            );
            page = page.child(reveal(
                "reveal-reveal-reader-export",
                (control("reveal-reader-export")
                    .icon(IconName::FolderOpen)
                    .ghost()
                    .label(label)
                    .accessibility_label(format!("在访达中显示导出文件：{}", path.display()))
                    .on_click(move |_, _, cx| cx.reveal_path(&path)))
                .into_any_element(),
            ));
        }
        // Find highlights group per block; the current match reads stronger.
        let mut find_marks: BTreeMap<usize, Vec<(Range<usize>, bool)>> = BTreeMap::new();
        if self.reader_ui.find_open {
            for (position, found) in self.reader_ui.matches.iter().enumerate() {
                find_marks
                    .entry(found.block)
                    .or_default()
                    .push((found.range.clone(), position == self.reader_ui.match_index));
            }
        }
        let highlight_runs = |index: usize| -> Vec<(Range<usize>, HighlightStyle)> {
            find_marks
                .get(&index)
                .map(|marks| {
                    marks
                        .iter()
                        .map(|(range, current)| {
                            (
                                range.clone(),
                                if *current {
                                    HighlightStyle {
                                        background_color: Some(color(FIND_CURRENT).into()),
                                        underline: Some(UnderlineStyle {
                                            thickness: px(1.),
                                            color: Some(color(FIND_CURRENT_LINE).into()),
                                            wavy: false,
                                        }),
                                        ..Default::default()
                                    }
                                } else {
                                    HighlightStyle {
                                        background_color: Some(color(FIND_HIGHLIGHT).into()),
                                        ..Default::default()
                                    }
                                },
                            )
                        })
                        .collect()
                })
                .unwrap_or_default()
        };
        let top_index = self
            .reader_ui
            .item_layout
            .borrow()
            .top_item(f32::from(self.reader_scroll.offset().y))
            .map(|(index, _, _)| index)
            .unwrap_or(0);
        self.reader_ui.item_layout = Rc::default();
        let item_layout = self.reader_ui.item_layout.clone();
        let measured_scroll = self.reader_scroll.clone();
        let reading_note = self.result_tab == 0;
        let measured = |index: usize, child: AnyElement| {
            let positions = item_layout.clone();
            let scroll = measured_scroll.clone();
            div()
                .w_full()
                .min_w_0()
                .flex_shrink_0()
                .when(reading_note, |view| view.max_w(READER_MEASURE).mx_auto())
                .on_children_prepainted(move |bounds, _, _| {
                    if let Some(bounds) = bounds.first() {
                        positions.borrow_mut().record(
                            index,
                            f32::from(bounds.top() - scroll.bounds().top() - scroll.offset().y),
                            f32::from(bounds.size.height),
                        );
                    }
                })
                .child(child)
        };
        let current_chapter = headings
            .iter()
            .filter(|(index, _, _)| *index <= top_index)
            .last()
            .map(|(index, _, _)| *index);
        // Article renders directly on the canvas (no white sheet); the summary is
        // the one white card in the reading flow.
        let mut article = v_flex()
            .id("note-reader")
            .role(Role::Document)
            .aria_label(preview.course.title.clone())
            .flex_1()
            .min_h_0()
            .min_w_0()
            .w_full()
            .when(reading_note, |view| view.items_center())
            .overflow_y_scroll()
            .track_scroll(&self.reader_scroll)
            .gap_4()
            .py_2()
            .when(compact, |view| view.gap_2().py_1());
        let article_scroll = self.reader_scroll.clone();
        let reveal_article = |id: ElementId, child: AnyElement, full_width: bool| {
            let view = crate::focus_scroll::RevealFocus::new(id, child, article_scroll.clone());
            if full_width { view } else { view.inline() }
        };
        let source = self.reader_source();
        // Inline figures support reading. Size their preview from the current
        // viewport, leaving room for the section text; the viewer keeps full size.
        let inline_image_max_height = if compact {
            px((available_height * 0.22).min(120.))
        } else {
            px((f32::from(window.bounds().size.height) * 0.28).min(280.))
        };
        if self.result_tab == 0 {
            let summary_paragraphs: Vec<(usize, String)> = preview
                .blocks
                .iter()
                .enumerate()
                .filter_map(|(index, block)| match block {
                    PreviewBlock::Paragraph { text, anchor }
                        if anchor == "summary-tldr" || anchor.starts_with("key-point-") =>
                    {
                        Some((index, text.clone()))
                    }
                    _ => None,
                })
                .collect();
            if !summary_paragraphs.is_empty() {
                article = article.child(
                    v_flex()
                        .id("reader-summary")
                        .flex_shrink_0()
                        .w_full()
                        .max_w(READER_MEASURE)
                        .gap_2()
                        .p_4()
                        .bg(color(SURFACE))
                        .border_1()
                        .border_color(color(CARD_LINE))
                        .rounded(RADIUS_CARD)
                        .child(
                            theme::accessible_text("reader-summary-label", "摘要")
                                .text_size(TEXT_AUX)
                                .text_color(color(GRAY))
                                .font_weight(FontWeight::SEMIBOLD),
                        )
                        .children(summary_paragraphs.iter().map(|(index, text)| {
                            measured(
                                *index,
                                div()
                                    .text_size(TEXT_BODY)
                                    .line_height(relative(1.6))
                                    .child(ReaderText::new(
                                        SharedString::from(format!("summary-text-{index}")),
                                        text.clone(),
                                        *index as u64,
                                        highlight_runs(*index),
                                    ))
                                    .into_any_element(),
                            )
                        })),
                );
            }
            for (index, block) in preview.blocks.iter().enumerate() {
                let consumed_by_summary = match block {
                    PreviewBlock::Heading { anchor, .. } => anchor == "summary",
                    PreviewBlock::Paragraph { anchor, .. } => {
                        anchor == "summary-tldr" || anchor.starts_with("key-point-")
                    }
                    _ => false,
                };
                if consumed_by_summary {
                    continue;
                }
                let view = match block {
                    PreviewBlock::Heading {
                        text,
                        anchor,
                        seconds,
                    } => {
                        let url = source.as_ref().and_then(|source| {
                            seconds.and_then(|seconds| nav::seek_url(source, seconds))
                        });
                        let marks = highlight_runs(index);
                        let outlined = chapter_title(*seconds).filter(|_| text != "摘要");
                        // 无大纲且标题文本就是时间戳时，chip 独自承担章节标题。
                        let bare_timestamp = outlined.is_none()
                            && seconds.is_some_and(|s| course2md::render::fmt_ts(s) == *text);
                        let display = outlined.unwrap_or_else(|| text.clone());
                        let heading: Option<AnyElement> = if bare_timestamp && marks.is_empty() {
                            None
                        } else if marks.is_empty() {
                            Some(
                                theme::accessible_text(("reader-heading", index), display)
                                    .role(Role::Heading)
                                    .text_size(TEXT_READER)
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .into_any_element(),
                            )
                        } else {
                            Some(
                                div()
                                    .id(("reader-heading-wrap", index))
                                    .role(Role::Heading)
                                    .aria_label(text.clone())
                                    .text_size(TEXT_READER)
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(ReaderText::new(
                                        ("reader-heading-marks", index),
                                        text.clone(),
                                        index as u64,
                                        marks,
                                    ))
                                    .into_any_element(),
                            )
                        };
                        v_flex()
                            .id(SharedString::from(anchor.clone()))
                            .gap_2()
                            .pt_3()
                            .when(compact, |view| view.pt_0())
                            .child(
                                h_flex()
                                    .flex_wrap()
                                    .gap_2()
                                    .items_baseline()
                                    .when_some(*seconds, |row, seconds| {
                                        let chip = div()
                                            .min_h(rems(1.571))
                                            .px(px(6.))
                                            .rounded(RADIUS_SMALL)
                                            .border_1()
                                            .border_color(color(HAIRLINE))
                                            .bg(color(SURFACE))
                                            .text_size(TEXT_AUX)
                                            .text_color(color(GRAY))
                                            .flex_shrink_0()
                                            .whitespace_nowrap()
                                            .child(course2md::render::fmt_ts(seconds));
                                        row.child(if bare_timestamp {
                                            chip.id(("reader-heading", index))
                                                .role(Role::Heading)
                                                .aria_label(text.clone())
                                        } else {
                                            chip.id(("reader-heading-chip", index))
                                        })
                                    })
                                    .children(heading)
                                    .when_some(url, |row, url| {
                                        row.child(reveal_article(
                                            ("reveal-seek", index).into(),
                                            (quiet(("seek", index))
                                                .icon(icons::play_arrow())
                                                .label("从此处观看")
                                                .min_h(rems(1.6))
                                                .accessibility_label(format!("在原视频打开 {text}"))
                                                .on_click(move |_, _, cx| cx.open_url(&url)))
                                            .into_any_element(),
                                            false,
                                        ))
                                    }),
                            )
                            .children(
                                self.reader_ui
                                    .frames
                                    .iter()
                                    .enumerate()
                                    .filter(|(_, frame)| {
                                        frame.path.is_none()
                                            && frame.body_anchor.as_ref() == Some(anchor)
                                    })
                                    .map(|(frame_index, frame)| {
                                        theme::accessible_text(
                                            ("missing-body-image", frame_index),
                                            format!(
                                                "{}无法读取。对应正文保留在下方。",
                                                frame_label(
                                                    &preview.course.title,
                                                    frame,
                                                    frame_index
                                                )
                                            ),
                                        )
                                        .text_sm()
                                        .text_color(color(WARNING))
                                    }),
                            )
                            .into_any_element()
                    }
                    PreviewBlock::Paragraph { text, anchor } => paragraph(
                        SharedString::from(anchor.clone()),
                        text.clone(),
                        index,
                        highlight_runs(index),
                    )
                    .into_any_element(),
                    PreviewBlock::Image(path) => {
                        let frame_index = self
                            .reader_ui
                            .frames
                            .iter()
                            .position(|frame| frame.path.as_ref() == Some(path));
                        let frame = frame_index.and_then(|i| self.reader_ui.frames.get(i));
                        if frame_index.is_none() && !self.reader_ui.data_loading {
                            article = article.child(measured(
                                index,
                                theme::accessible_text(
                                    ("unreadable-inline-image", index),
                                    "这张截图无法读取；对应正文仍可阅读。",
                                )
                                .text_sm()
                                .text_color(color(WARNING))
                                .into_any_element(),
                            ));
                            continue;
                        }
                        let label = frame
                            .map(|frame| {
                                frame_label(&preview.course.title, frame, frame_index.unwrap())
                            })
                            .unwrap_or_else(|| format!("{}，正文图片", preview.course.title));
                        let seconds = frame.and_then(|frame| frame.seconds);
                        v_flex()
                            .w_full()
                            .bg(color(SURFACE))
                            .border_1()
                            .border_color(color(CARD_LINE))
                            .rounded(RADIUS_CARD)
                            .overflow_hidden()
                            .when(compact, |view| view.flex_row().items_center())
                            .child(reveal_article(
                                ("reveal-note-image", index).into(),
                                h_flex()
                                    .w_full()
                                    .when(compact, |view| {
                                        view.w(inline_image_max_height * (16. / 9.)).flex_shrink_0()
                                    })
                                    .justify_center()
                                    .bg(color(INSET))
                                    .rounded_t(RADIUS_CARD)
                                    .child(
                                        control(("note-image", index))
                                            .ghost()
                                            .p_0()
                                            .w_full()
                                            .h_auto()
                                            .min_h(px(0.))
                                            .rounded_t(RADIUS_CARD)
                                            .rounded_b(px(0.))
                                            .aspect_ratio(16. / 9.)
                                            .max_h(inline_image_max_height)
                                            .bg(color(INSET))
                                            .accessibility_label(format!("放大{label}"))
                                            .disabled(frame_index.is_none())
                                            .child(
                                                img(path.clone())
                                                    .size_full()
                                                    .rounded_t(RADIUS_CARD)
                                                    .object_fit(ObjectFit::Contain)
                                                    .with_fallback(|| {
                                                        theme::accessible_text(
                                                            "failed-reader-image",
                                                            "这张截图无法读取；对应正文仍可阅读。",
                                                        )
                                                        .into_any_element()
                                                    }),
                                            )
                                            .on_click(cx.listener(move |this, _, window, cx| {
                                                if let Some(index) = frame_index {
                                                    this.open_reader_image(index, window, cx);
                                                }
                                            })),
                                    )
                                    .into_any_element(),
                                !compact,
                            ))
                            .child(
                                h_flex()
                                    .gap_2()
                                    .items_center()
                                    .flex_wrap()
                                    .px(px(12.))
                                    .py(px(8.))
                                    .border_t_1()
                                    .border_color(color(CARD_LINE))
                                    .when(compact, |row| {
                                        row.flex_1().min_w_0().border_t_0().border_l_1().py_0()
                                    })
                                    .text_size(TEXT_AUX)
                                    .text_color(color(GRAY))
                                    .child(theme::accessible_text(
                                        ("figure-caption", index),
                                        seconds
                                            .map(|seconds| {
                                                format!(
                                                    "视频截图 · {}",
                                                    course2md::render::fmt_ts(seconds)
                                                )
                                            })
                                            .unwrap_or_else(|| "视频截图".into()),
                                    ))
                                    .child(div().flex_1())
                                    .when_some(frame_index, |row, frame_index| {
                                        row.child(reveal_article(
                                            ("reveal-figure-enlarge", index).into(),
                                            control(("figure-enlarge", index))
                                                .ghost()
                                                .icon(icons::zoom_in())
                                                .w(rems(2.571))
                                                .px_0()
                                                .accessibility_label("放大截图")
                                                .tooltip("放大截图")
                                                .on_click(cx.listener(
                                                    move |this, _, window, cx| {
                                                        this.open_reader_image(
                                                            frame_index,
                                                            window,
                                                            cx,
                                                        )
                                                    },
                                                ))
                                                .into_any_element(),
                                            false,
                                        ))
                                    }),
                            )
                            .into_any_element()
                    }
                };
                article = article.child(measured(index, view));
            }
        } else {
            if self.reader_ui.data_loading && self.reader_ui.frames.is_empty() {
                article = article.child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .py_6()
                        .child(crate::motion::spinner("reader-images-spinner", cx))
                        .child(theme::accessible_text("loading-images", "正在读取截图…")),
                );
            } else if self.reader_ui.frames.is_empty() {
                article = article.child(crate::motion::enter(
                    "reader-images-empty",
                    v_flex()
                        .w_full()
                        .py_8()
                        .px_6()
                        .gap_4()
                        .items_center()
                        .bg(color(INSET))
                        .rounded(RADIUS_CARD)
                        .child(icons::image().size(px(32.)).text_color(color(MUTED)))
                        .child(
                            theme::accessible_text("no-note-images", "这份笔记没有截图")
                                .text_size(TEXT_TITLE)
                                .font_weight(FontWeight::SEMIBOLD),
                        )
                        .child(
                            outline_pill("empty-images-to-note")
                                .icon(icons::article())
                                .label("阅读笔记")
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.select_reader_view(0, window, cx)
                                })),
                        ),
                    cx,
                ));
            }
            // Shot grid follows the mock's flex-wrap recipe: 12rem minimum cards
            // wrap to two columns when the column narrows.
            let mut grid = h_flex()
                .w_full()
                .min_w_0()
                .flex_shrink_0()
                .flex_wrap()
                .gap_4();
            for (index, frame) in self.reader_ui.frames.iter().enumerate() {
                let label = frame_label(&preview.course.title, frame, index);
                let mut card = v_flex()
                    .id(SharedString::from(frame.anchor.clone()))
                    .w_full()
                    .bg(color(SURFACE))
                    .border_1()
                    .border_color(color(CARD_LINE))
                    .rounded(RADIUS_CARD)
                    .overflow_hidden();
                if let Some(path) = &frame.path {
                    card = card.child(reveal_article(
                        ("reveal-screenshot", index).into(),
                        (control(("screenshot", index))
                            .ghost()
                            .p_0()
                            .w_full()
                            .h_auto()
                            .min_h(px(0.))
                            .rounded_t(RADIUS_CARD)
                            .rounded_b(px(0.))
                            .aspect_ratio(16. / 9.)
                            .accessibility_label(format!("放大{label}"))
                            .child(
                                img(path.clone())
                                    .size_full()
                                    .rounded_t(RADIUS_CARD)
                                    .object_fit(ObjectFit::Cover)
                                    .with_fallback(|| {
                                        theme::accessible_text(
                                            "failed-reader-image",
                                            "这张截图无法读取；对应正文仍可阅读。",
                                        )
                                        .into_any_element()
                                    }),
                            )
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.open_reader_image(index, window, cx)
                            })))
                        .into_any_element(),
                        true,
                    ));
                } else {
                    card = card.child(
                        div().p_3().child(
                            theme::accessible_text(
                                ("missing-frame", index),
                                "这张图片无法读取；可继续阅读对应正文。",
                            )
                            .text_size(TEXT_AUX)
                            .text_color(color(WARNING)),
                        ),
                    );
                }
                let mut card_body = v_flex().p_4().gap_2().child(
                    h_flex()
                        .items_center()
                        .gap_2()
                        .child(
                            theme::accessible_text(
                                ("frame-title", index),
                                frame
                                    .seconds
                                    .map(course2md::render::fmt_ts)
                                    .unwrap_or_else(|| format!("第 {} 张图片", index + 1)),
                            )
                            .role(Role::Heading)
                            .flex_1()
                            .font_weight(FontWeight::SEMIBOLD),
                        )
                        .when(frame.path.is_some(), |row| {
                            row.child(reveal_article(
                                ("reveal-frame-enlarge", index).into(),
                                control(("frame-enlarge", index))
                                    .ghost()
                                    .icon(icons::zoom_in())
                                    .w(rems(2.571))
                                    .px_0()
                                    .accessibility_label("放大截图")
                                    .tooltip("放大截图")
                                    .on_click(cx.listener(move |this, _, window, cx| {
                                        this.open_reader_image(index, window, cx)
                                    }))
                                    .into_any_element(),
                                false,
                            ))
                        }),
                );
                if let Some(caption) = &frame.caption {
                    card_body = card_body.child(
                        div()
                            .w_full()
                            .whitespace_normal()
                            .text_ellipsis()
                            .line_clamp(2)
                            .text_size(TEXT_AUX)
                            .text_color(color(GRAY))
                            .child(format!("原图说明：{caption}")),
                    );
                }
                if !frame.transcript.is_empty() {
                    card_body = card_body.child(
                        div()
                            .w_full()
                            .whitespace_normal()
                            .text_ellipsis()
                            .line_clamp(3)
                            .text_size(TEXT_AUX)
                            .text_color(color(GRAY))
                            .child(format!(
                                "{}：{}",
                                if frame.seconds.is_some() {
                                    "同期转录"
                                } else {
                                    "相邻正文"
                                },
                                frame.transcript
                            )),
                    );
                }
                let body_index = frame.body_anchor.as_ref().and_then(|anchor| {
                    preview
                        .blocks
                        .iter()
                        .enumerate()
                        .position(|(i, block)| block_anchor(block, i) == *anchor)
                });
                let mut actions = h_flex().gap_2().flex_wrap();
                if let Some(body_index) = body_index {
                    actions = actions.child(reveal_article(
                        ("reveal-frame-to-body", index).into(),
                        (outline_pill(("frame-to-body", index))
                            .icon(icons::article())
                            .label("定位正文")
                            .min_h(rems(2.286))
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.jump_reader_block(body_index, 0., cx)
                            })))
                        .into_any_element(),
                        false,
                    ));
                }
                if let Some(url) = source.as_ref().and_then(|source| {
                    frame
                        .seconds
                        .and_then(|seconds| nav::seek_url(source, seconds))
                }) {
                    actions = actions.child(reveal_article(
                        ("reveal-frame-to-source", index).into(),
                        (quiet(("frame-to-source", index))
                            .icon(icons::play_arrow())
                            .label("观看")
                            .min_h(rems(2.286))
                            .on_click(move |_, _, cx| cx.open_url(&url)))
                        .into_any_element(),
                        false,
                    ));
                } else if source.is_some() && self.reader_ui.source_available {
                    actions = actions.child(reveal_article(
                        ("reveal-frame-open-original", index).into(),
                        (quiet(("frame-open-original", index))
                            .icon(icons::movie())
                            .label("打开原视频")
                            .min_h(rems(2.286))
                            .on_click(
                                cx.listener(|this, _, _, cx| this.open_reader_source(None, cx)),
                            ))
                        .into_any_element(),
                        false,
                    ));
                }
                grid = grid.child(
                    measured(
                        index,
                        card.child(card_body.child(actions)).into_any_element(),
                    )
                    .flex_1()
                    .flex_basis(rems(12.))
                    .min_w(rems(12.))
                    .max_w(rems(17.)),
                );
            }
            article = article.child(grid);
        }
        // TOC panel: beside the article when the column can spare 16rem, below it
        // otherwise (mock flex-wrap semantics).
        let toc_open = self.reader_ui.toc_open && !headings.is_empty() && self.result_tab == 0;
        let rem = f32::from(window.rem_size());
        let content_width = crate::views::shell_content_width(Page::Result, window);
        let toc_side = toc_open && content_width >= 40. * rem + 24.;
        let body = if toc_side {
            h_flex()
                .flex_1()
                .min_h_0()
                .w_full()
                .gap(px(24.))
                .items_stretch()
                .child(article.h_full())
                .child(self.reader_toc_panel(&headings, current_chapter, true, cx))
                .into_any_element()
        } else if toc_open {
            v_flex()
                .flex_1()
                .min_h_0()
                .w_full()
                .gap_3()
                .child(article)
                .child(self.reader_toc_panel(&headings, current_chapter, false, cx))
                .into_any_element()
        } else {
            article.into_any_element()
        };
        let controls = if let Some(header) = fixed_header {
            v_flex()
                .w_full()
                .min_h_0()
                .flex_shrink_0()
                .child(header)
                .child(page)
                .into_any_element()
        } else {
            page.into_any_element()
        };
        root.child(controls)
            .child(crate::motion::enter(
                ("reader-view-content", self.result_tab),
                v_flex().flex_1().min_h_0().w_full().child(body),
                cx,
            ))
            .into_any_element()
    }
    /// Mock `tocPanel`: sticky-look white card listing chapters; the chapter at the
    /// current reading position reads coral on a soft wash.
    fn reader_toc_panel(
        &self,
        headings: &[(usize, String, Option<f64>)],
        current: Option<usize>,
        side: bool,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let mut panel = v_flex()
            .id("note-toc")
            .gap_1()
            .p_3()
            .bg(color(SURFACE))
            .border_1()
            .border_color(color(CARD_LINE))
            .rounded(RADIUS_CARD)
            .overflow_y_scroll()
            .track_scroll(&self.reader_ui.toc_scroll);
        panel = if side {
            panel.w(rems(16.)).flex_shrink_0().h_full().min_h_0()
        } else {
            panel.w_full().flex_shrink_0().max_h(relative(0.4))
        };
        panel = panel.child(
            theme::accessible_text("toc-title", "目录")
                .flex_shrink_0()
                .text_size(TEXT_AUX)
                .text_color(color(GRAY))
                .font_weight(FontWeight::SEMIBOLD),
        );
        for (index, label, seconds) in headings {
            let index = *index;
            let on = current == Some(index);
            panel = panel.child(crate::focus_scroll::RevealFocus::new(
                ("reveal-toc-item", index),
                control(("toc-item", index))
                    .ghost()
                    .w_full()
                    .h_auto()
                    .min_h(rems(2.))
                    .py(px(6.))
                    .px(px(8.))
                    .justify_start()
                    .rounded(px(8.))
                    .when(on, |button| {
                        button
                            .bg(color(ACCENT_SOFT))
                            .font_weight(FontWeight::SEMIBOLD)
                    })
                    .accessibility_label(format!("转到 {label}"))
                    .child(
                        h_flex()
                            .w_full()
                            .min_w_0()
                            .gap_2()
                            .items_baseline()
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .whitespace_nowrap()
                                    .text_ellipsis()
                                    .text_color(color(if on { ACCENT_STRONG } else { INK }))
                                    .child(label.clone()),
                            )
                            .when_some(*seconds, |row, seconds| {
                                row.child(
                                    div()
                                        .flex_shrink_0()
                                        .text_size(TEXT_AUX)
                                        .text_color(color(if on { ACCENT_STRONG } else { GRAY }))
                                        .child(course2md::render::fmt_ts(seconds)),
                                )
                            }),
                    )
                    .on_click(
                        cx.listener(move |this, _, _, cx| this.jump_reader_block(index, 0., cx)),
                    ),
                self.reader_ui.toc_scroll.clone(),
            ));
        }
        panel
    }
    fn open_reader_image(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        let Some(preview) = &self.preview else {
            return;
        };
        if self
            .reader_ui
            .frames
            .get(index)
            .is_none_or(|frame| frame.path.is_none())
        {
            return;
        }
        let focus = cx.focus_handle();
        self.reader_ui.viewer = Some(ImageViewer {
            scroll: ScrollHandle::new(),
            viewport_scroll: ScrollHandle::new(),
            viewport_size: size(px(0.), px(0.)),
            frames: self.reader_ui.frames.clone(),
            index,
            title: preview.course.title.clone(),
            source: self.reader_source(),
            source_available: self.reader_ui.source_available,
            version: preview.course.dir.clone(),
            zoom: None,
            return_focus: window.focused(cx),
            focus: focus.clone(),
        });
        let desktop = cx.entity();
        let content = cx.new(|cx| ImageDialog {
            _observation: cx.observe(&desktop, |_, _, cx| cx.notify()),
            desktop,
        });
        let weak = cx.weak_entity();
        window.open_dialog(cx, move |dialog, window, _| {
            let width = (f32::from(window.bounds().size.width) - 48.)
                .min(1180.)
                .max(280.);
            let closed = weak.clone();
            dialog
                .title("查看截图")
                .w(px(width))
                .margin_top(px(16.))
                .overlay_closable(false)
                .close_button(false)
                .child(content.clone())
                .on_close(move |_, window, cx| {
                    let _ =
                        closed.update(cx, |this, cx| this.restore_reader_image_focus(window, cx));
                })
        });
        focus.focus(window, cx);
        cx.notify();
    }
    fn restore_reader_image_focus(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(viewer) = self.reader_ui.viewer.take() {
            if let Some(focus) = viewer.return_focus {
                focus.focus(window, cx);
            }
        }
        cx.notify();
    }
    fn close_reader_image(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        window.close_dialog(cx);
        self.restore_reader_image_focus(window, cx);
    }
    fn move_reader_image(&mut self, delta: isize, cx: &mut Context<Self>) {
        if let Some(viewer) = &mut self.reader_ui.viewer {
            viewer.index = (viewer.index as isize + delta)
                .clamp(0, viewer.frames.len().saturating_sub(1) as isize)
                as usize;
            viewer.zoom = None;
            viewer.viewport_scroll.set_offset(point(px(0.), px(0.)));
            cx.notify();
        }
    }
    fn image_fit(viewer: &ImageViewer, window: &Window) -> f32 {
        let frame = &viewer.frames[viewer.index];
        let (width, height) =
            if viewer.viewport_size.width > px(0.) && viewer.viewport_size.height > px(0.) {
                (
                    f32::from(viewer.viewport_size.width),
                    f32::from(viewer.viewport_size.height),
                )
            } else {
                (
                    (f32::from(window.bounds().size.width) - 100.).clamp(180., 1120.),
                    (f32::from(window.bounds().size.height) - 400.).max(140.),
                )
            };
        (width / frame.width.max(1) as f32)
            .min(height / frame.height.max(1) as f32)
            .min(1.)
    }
    fn zoom_reader_image(&mut self, factor: Option<f32>, window: &Window, cx: &mut Context<Self>) {
        if let Some(viewer) = &mut self.reader_ui.viewer {
            let old_scale = viewer
                .zoom
                .unwrap_or_else(|| Self::image_fit(viewer, window));
            viewer.zoom = factor.map(|factor| (old_scale * factor).clamp(0.1, 4.));
            let new_scale = viewer
                .zoom
                .unwrap_or_else(|| Self::image_fit(viewer, window));
            let frame = &viewer.frames[viewer.index];
            let viewport = viewer.viewport_scroll.bounds().size;
            let offset = viewer.viewport_scroll.offset();
            viewer.viewport_scroll.set_offset(point(
                px(image_zoom_offset(
                    f32::from(viewport.width),
                    frame.width as f32 * old_scale,
                    frame.width as f32 * new_scale,
                    f32::from(offset.x),
                )),
                px(image_zoom_offset(
                    f32::from(viewport.height),
                    frame.height as f32 * old_scale,
                    frame.height as f32 * new_scale,
                    f32::from(offset.y),
                )),
            ));
            cx.notify();
        }
    }
    fn reader_image_content(&mut self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let Some(viewer) = &self.reader_ui.viewer else {
            return div().into_any_element();
        };
        let frame = viewer.frames[viewer.index].clone();
        let index = viewer.index;
        let count = viewer.frames.len();
        let label = frame_label(&viewer.title, &frame, index);
        let scale = viewer
            .zoom
            .unwrap_or_else(|| Self::image_fit(viewer, window));
        let version = viewer.version.clone();
        let source_link = viewer
            .source
            .as_ref()
            .and_then(|source| frame.seconds.and_then(|time| nav::seek_url(source, time)));
        let original_source = viewer.source.clone();
        let body_anchor = frame.body_anchor.clone();
        let scroll = viewer.scroll.clone();
        let reveal = |id: &'static str, child: AnyElement| {
            crate::focus_scroll::RevealFocus::new(id, child, scroll.clone()).inline()
        };
        let mut body = v_flex()
            .id("reader-image-dialog")
            .role(Role::Dialog)
            .aria_label(format!("查看{label}"))
            .track_focus(&viewer.focus)
            .key_context("ReaderImage")
            .on_action(cx.listener(|this, _: &PreviousImage, _, cx| this.move_reader_image(-1, cx)))
            .on_action(cx.listener(|this, _: &NextImage, _, cx| this.move_reader_image(1, cx)))
            .on_action(cx.listener(|this, _: &ZoomIn, window, cx| {
                this.zoom_reader_image(Some(1.25), window, cx)
            }))
            .on_action(cx.listener(|this, _: &ZoomOut, window, cx| {
                this.zoom_reader_image(Some(0.8), window, cx)
            }))
            .on_action(cx.listener(|this, _: &FitImage, window, cx| {
                this.zoom_reader_image(None, window, cx)
            }))
            .on_action(
                cx.listener(|this, _: &CloseImage, window, cx| this.close_reader_image(window, cx)),
            )
            .gap_3()
            .h(px((f32::from(window.bounds().size.height) - 125.).max(160.)))
            .min_h_0()
            .overflow_y_scroll()
            .track_scroll(&scroll)
            .child(
                theme::accessible_text("image-title", label.clone())
                    .role(Role::Heading)
                    .w_full()
                    .min_w_0()
                    .flex_shrink_0()
                    .whitespace_normal()
                    .text_ellipsis()
                    .line_clamp(2)
                    .text_lg(),
            )
            .child(
                h_flex()
                    .gap_2()
                    .flex_wrap()
                    .flex_shrink_0()
                    .child(reveal(
                        "reveal-image-previous",
                        (control("image-previous")
                            .icon(IconName::ChevronLeft)
                            .label("上一张")
                            .disabled(index == 0)
                            .on_click(
                                cx.listener(|this, _, _, cx| this.move_reader_image(-1, cx)),
                            ))
                        .into_any_element(),
                    ))
                    .child(theme::accessible_text(
                        "image-number",
                        format!("{} / {count}", index + 1),
                    ))
                    .child(reveal(
                        "reveal-image-next",
                        (control("image-next")
                            .icon(IconName::ChevronRight)
                            .label("下一张")
                            .disabled(index + 1 >= count)
                            .on_click(cx.listener(|this, _, _, cx| this.move_reader_image(1, cx))))
                        .into_any_element(),
                    ))
                    .child(reveal(
                        "reveal-image-zoom-out",
                        (control("image-zoom-out")
                            .icon(icons::zoom_out())
                            .label("缩小")
                            .disabled(scale <= 0.1)
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.zoom_reader_image(Some(0.8), window, cx)
                            })))
                        .into_any_element(),
                    ))
                    .child(theme::accessible_text(
                        "image-scale",
                        format!("{}%", (scale * 100.).round() as u32),
                    ))
                    .child(reveal(
                        "reveal-image-zoom-in",
                        (control("image-zoom-in")
                            .icon(icons::zoom_in())
                            .label("放大")
                            .disabled(scale >= 4.)
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.zoom_reader_image(Some(1.25), window, cx)
                            })))
                        .into_any_element(),
                    ))
                    .child(reveal(
                        "reveal-image-fit",
                        (control("image-fit")
                            .icon(icons::fit_screen())
                            .label("适合窗口")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.zoom_reader_image(None, window, cx)
                            })))
                        .into_any_element(),
                    ))
                    .child(reveal(
                        "reveal-image-close",
                        (control("image-close")
                            .icon(IconName::Close)
                            .label("关闭截图")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.close_reader_image(window, cx)
                            })))
                        .into_any_element(),
                    )),
            );
        if let Some(path) = frame.path {
            let viewport_scroll = viewer.viewport_scroll.clone();
            let viewport_owner = cx.weak_entity();
            let viewport_version = version.clone();
            body = body.child(
                div()
                    .on_children_prepainted(move |_, window, cx| {
                        let size = viewport_scroll.bounds().size;
                        let owner = viewport_owner.clone();
                        let version = viewport_version.clone();
                        window.defer(cx, move |_, cx| {
                            let _ = owner.update(cx, |this, cx| {
                                if let Some(viewer) = &mut this.reader_ui.viewer
                                    && viewer.version == version
                                    && viewer.viewport_size != size
                                {
                                    viewer.viewport_size = size;
                                    cx.notify();
                                }
                            });
                        });
                    })
                    .id("image-viewport")
                    .bg(color(INSET))
                    .border_1()
                    .border_color(color(HAIRLINE))
                    .rounded(RADIUS_CARD)
                    .flex_1()
                    .min_h(px(140.))
                    .w_full()
                    .overflow_x_scroll()
                    .overflow_y_scroll()
                    .track_scroll(&viewer.viewport_scroll)
                    .child(
                        div()
                            .id("reader-image-canvas")
                            .flex_none()
                            .w(px(frame.width as f32 * scale))
                            .h(px(frame.height as f32 * scale))
                            .min_w(relative(1.))
                            .min_h(relative(1.))
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(
                                div()
                                    .id("enlarged-reader-image")
                                    .flex_none()
                                    .w(px(frame.width as f32 * scale))
                                    .h(px(frame.height as f32 * scale))
                                    .role(Role::Image)
                                    .aria_label(
                                        frame
                                            .caption
                                            .as_ref()
                                            .map(|caption| format!("{label}。原图说明：{caption}"))
                                            .unwrap_or(label),
                                    )
                                    .child(
                                        img(path)
                                            .size_full()
                                            .object_fit(ObjectFit::Contain)
                                            .with_fallback(|| {
                                                theme::accessible_text(
                                                    "failed-reader-image",
                                                    "这张截图无法读取；对应正文仍可阅读。",
                                                )
                                                .into_any_element()
                                            }),
                                    ),
                            ),
                    ),
            );
        } else {
            body = body.child(theme::accessible_text(
                "image-unreadable",
                "这张截图无法读取。对应正文保留在下方。",
            ));
        }
        let has_details = frame.caption.is_some() || !frame.transcript.is_empty();
        let mut details = v_flex()
            .id("image-details")
            .w_full()
            .flex_shrink_0()
            .gap_2()
            .max_h(px(96.))
            .overflow_y_scroll();
        if let Some(caption) = frame.caption {
            details = details.child(paragraph(
                "image-caption",
                format!("原图说明：{caption}"),
                0,
                Vec::new(),
            ));
        }
        if !frame.transcript.is_empty() {
            details = details.child(paragraph(
                "image-transcript",
                format!(
                    "{}：{}",
                    if frame.seconds.is_some() {
                        "同期转录"
                    } else {
                        "相邻正文"
                    },
                    frame.transcript
                ),
                1,
                Vec::new(),
            ));
        }
        if has_details {
            body = body.child(details);
        }
        let mut actions = h_flex().gap_2().flex_wrap().flex_shrink_0();
        if let Some(anchor) = body_anchor {
            actions =
                actions.child(reveal(
                    "reveal-image-to-body",
                    (control("image-to-body")
                        .icon(icons::article())
                        .label("在笔记中查看")
                        .on_click(cx.listener(move |this, _, window, cx| {
                            let index =
                                this.preview
                                    .as_ref()
                                    .filter(|preview| preview.course.dir == version)
                                    .and_then(|preview| {
                                        preview.blocks.iter().enumerate().position(
                                            |(index, block)| block_anchor(block, index) == anchor,
                                        )
                                    });
                            this.close_reader_image(window, cx);
                            if let Some(index) = index {
                                this.jump_reader_block(index, 0., cx);
                            }
                        })))
                    .into_any_element(),
                ));
        }
        if let Some(url) = source_link {
            actions = actions.child(reveal(
                "reveal-image-to-source",
                (control("image-to-source")
                    .icon(icons::play_arrow())
                    .ghost()
                    .label("从此处观看")
                    .on_click(move |_, _, cx| cx.open_url(&url)))
                .into_any_element(),
            ));
        } else if let Some(source) = original_source.filter(|_| viewer.source_available) {
            actions = actions.child(reveal(
                "reveal-image-open-original",
                (control("image-open-original")
                    .icon(icons::movie())
                    .ghost()
                    .label("打开原视频")
                    .on_click(move |_, _, cx| match &source {
                        nav::SourceTarget::Web(url) => cx.open_url(url),
                        nav::SourceTarget::Local(path) => cx.open_with_system(path),
                    }))
                .into_any_element(),
            ));
        }
        body.child(actions).into_any_element()
    }
}

fn load_reader_data(preview: &notes::Preview) -> ReaderData {
    let mut data = ReaderData::default();
    let dir = &preview.course.dir;
    let provenance = if preview.course.manifest.is_some() {
        course2md::legacy::provenance(dir)
    } else {
        course2md::legacy::read(dir).map(|note| note.map(|note| note.provenance))
    };
    match provenance {
        Ok(Some(provenance)) => {
            let mut seconds = None;
            let mut anchor = None;
            for (index, block) in provenance.blocks.iter().enumerate() {
                match block {
                    course2md::legacy::Block::Heading { seconds: time, .. } => {
                        seconds = *time;
                        anchor = Some(format!("legacy-heading-{index}"));
                    }
                    course2md::legacy::Block::Paragraph { .. } => {
                        anchor = Some(format!("legacy-paragraph-{index}"));
                    }
                    course2md::legacy::Block::Image { reference, alt } => {
                        let resource = provenance
                            .resources
                            .iter()
                            .find(|resource| resource.reference == *reference);
                        let path = resource.and_then(|resource| {
                            let relative = if preview.course.manifest.is_some() {
                                Some(resource.path.as_str())
                            } else {
                                resource
                                    .original_path
                                    .as_deref()
                                    .and_then(|path| path.strip_prefix("original/"))
                            }?;
                            course2md::artifact::safe_asset_path(dir, relative).ok()
                        });
                        let transcript = provenance
                            .blocks
                            .iter()
                            .skip(index + 1)
                            .take_while(|block| {
                                matches!(block, course2md::legacy::Block::Paragraph { .. })
                            })
                            .filter_map(|block| {
                                if let course2md::legacy::Block::Paragraph { text } = block {
                                    Some(text.as_str())
                                } else {
                                    None
                                }
                            })
                            .collect::<Vec<_>>()
                            .join("\n\n");
                        let body_anchor = anchor.clone().or_else(|| {
                            provenance
                                .blocks
                                .iter()
                                .enumerate()
                                .skip(index + 1)
                                .find_map(|(i, block)| match block {
                                    course2md::legacy::Block::Heading { .. } => {
                                        Some(format!("legacy-heading-{i}"))
                                    }
                                    course2md::legacy::Block::Paragraph { .. } => {
                                        Some(format!("legacy-paragraph-{i}"))
                                    }
                                    _ => None,
                                })
                        });
                        data.frames.push(checked_frame(Frame {
                            anchor: format!("legacy-image-{index}"),
                            path,
                            seconds,
                            caption: (!alt.trim().is_empty()).then(|| alt.clone()),
                            transcript,
                            body_anchor,
                            width: 16,
                            height: 9,
                        }));
                    }
                }
            }
        }
        Ok(None) => {
            if let Some(manifest) = &preview.course.manifest {
                for (index, frame) in manifest.frames.iter().enumerate() {
                    let section = preview.document.as_ref().and_then(|document| {
                        document
                            .sections
                            .iter()
                            .enumerate()
                            .find(|(_, section)| section.image == frame.image)
                    });
                    data.frames.push(checked_frame(Frame {
                        anchor: format!("frame:{index}:{}", frame.image),
                        path: course2md::artifact::safe_asset_path(dir, &frame.image).ok(),
                        seconds: Some(frame.t),
                        caption: None,
                        transcript: section
                            .map(|(_, section)| {
                                section
                                    .speech
                                    .iter()
                                    .map(|speech| speech.text.as_str())
                                    .collect::<Vec<_>>()
                                    .join("\n\n")
                            })
                            .unwrap_or_default(),
                        body_anchor: section
                            .map(|(index, _)| format!("section-{index}"))
                            .or_else(|| {
                                nav::nearest_time(
                                    preview.blocks.iter().enumerate().map(|(index, block)| {
                                        (
                                            index,
                                            match block {
                                                PreviewBlock::Heading { seconds, .. } => *seconds,
                                                _ => None,
                                            },
                                        )
                                    }),
                                    frame.t,
                                )
                                .map(|(i, _)| block_anchor(&preview.blocks[i], i))
                            }),
                        width: 16,
                        height: 9,
                    }));
                }
            }
        }
        Err(error) => data.issues.push(format!(
            "图片来源记录暂时无法读取：{error:#}。正文仍可阅读。"
        )),
    }
    if let Some(manifest) = &preview.course.manifest {
        let parent = dir
            .parent()
            .filter(|parent| parent.file_name().is_some_and(|name| name == "versions"));
        if let Some(parent) = parent {
            match std::fs::read_dir(parent) {
                Ok(entries) => {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if !path.is_dir() || !path.join("manifest.json").is_file() {
                            continue;
                        }
                        match course2md::artifact::read_manifest(&path.join("manifest.json")) {
                            Ok(other) if other.course_id == manifest.course_id => {
                                let mut course = preview.course.clone();
                                course.dir = path;
                                course.title = other.title.clone();
                                course.slides = other.frames.len();
                                let label = format!(
                                    "第 {} 版 · {}{}",
                                    other.revision,
                                    nav::timestamp_local(other.created_at_ms),
                                    if other.partial {
                                        " · 部分完成"
                                    } else {
                                        ""
                                    }
                                );
                                course.manifest = Some(other);
                                data.versions.push(Version { course, label });
                            }
                            Err(error) => data
                                .issues
                                .push(format!("有一个旧版本的清单无法读取：{error:#}")),
                            _ => {}
                        }
                    }
                    data.versions.sort_by_key(|version| {
                        std::cmp::Reverse(
                            version
                                .course
                                .manifest
                                .as_ref()
                                .map(|manifest| (manifest.revision, manifest.created_at_ms))
                                .unwrap_or_default(),
                        )
                    });
                }
                Err(error) => data
                    .issues
                    .push(format!("其他版本暂时无法读取：{error}。当前版本仍可阅读。")),
            }
        }
    }
    data
}
fn checked_frame(mut frame: Frame) -> Frame {
    if let Some((width, height)) = frame
        .path
        .as_ref()
        .and_then(|path| image::image_dimensions(path).ok())
        .filter(|(width, height)| *width > 0 && *height > 0)
    {
        frame.width = width;
        frame.height = height;
    } else {
        frame.path = None;
    }
    frame
}

fn processing_notice(issues: &[crate::notes::ProcessingIssue]) -> Option<String> {
    if issues.is_empty() {
        return None;
    }
    Some(format!(
        "正文已保存，{}未完成。",
        issues
            .iter()
            .map(|issue| issue.stage.label())
            .collect::<Vec<_>>()
            .join("、")
    ))
}
fn files_need_reload(preview: &crate::notes::Preview, issues: &[String], frames: &[Frame]) -> bool {
    !preview.issues.is_empty()
        || !issues.is_empty()
        || frames.iter().any(|frame| frame.path.is_none())
}

/// Preserve the image point under the viewport center while either axis grows
/// from a centered preview into scrollable content, or returns to a fitted image.
fn image_zoom_offset(viewport: f32, old_extent: f32, new_extent: f32, offset: f32) -> f32 {
    if viewport <= 0. || old_extent <= 0. {
        return 0.;
    }
    let old_margin = (viewport - old_extent).max(0.) * 0.5;
    let image_position = ((viewport * 0.5 - offset - old_margin) / old_extent).clamp(0., 1.);
    let new_margin = (viewport - new_extent).max(0.) * 0.5;
    (viewport * 0.5 - new_margin - image_position * new_extent)
        .clamp(-(new_extent - viewport).max(0.), 0.)
}

#[cfg(test)]
mod tests {
    use super::{
        OfflineVideo, OfflineVideoRequest, PreviewBlock, block_time, files_need_reload,
        image_zoom_offset, load_reader_data, processing_notice,
    };
    use crate::{ConversionOptions, notes::Course, source, workspace};

    #[test]
    fn image_zoom_keeps_the_visible_center_and_all_edges_reachable() {
        // A centered small image grows around its midpoint, then fits again.
        assert_eq!(image_zoom_offset(1000., 640., 1280., 0.), -140.);
        assert_eq!(image_zoom_offset(1000., 1280., 640., -140.), 0.);
        // A panned image preserves the same content at the viewport center.
        assert_eq!(image_zoom_offset(1000., 2000., 4000., -400.), -1300.);
        // Shrinking near an edge clamps to reachable bounds without hiding it.
        assert_eq!(image_zoom_offset(1000., 2000., 1200., -1000.), -200.);
        assert_eq!(image_zoom_offset(1000., 2000., 1200., 0.), 0.);
    }

    #[test]
    fn an_untimed_heading_ends_the_preceding_time_scope() {
        let blocks = vec![
            PreviewBlock::Heading {
                text: "0:10".into(),
                anchor: "a".into(),
                seconds: Some(10.),
            },
            PreviewBlock::Paragraph {
                text: "第一段".into(),
                anchor: "b".into(),
            },
            PreviewBlock::Heading {
                text: "补充主题".into(),
                anchor: "c".into(),
                seconds: None,
            },
            PreviewBlock::Paragraph {
                text: "第二段".into(),
                anchor: "d".into(),
            },
        ];
        assert_eq!(block_time(&blocks, 1), Some(10.));
        assert_eq!(block_time(&blocks, 3), None);
    }
    fn fixture_version(
        root: &std::path::Path,
        revision: u64,
        times: &[f64],
    ) -> crate::notes::Course {
        fixture_with_outcomes(root, revision, times, Default::default())
    }
    fn fixture_with_outcomes(
        root: &std::path::Path,
        revision: u64,
        times: &[f64],
        outcomes: course2md::artifact::Outcomes,
    ) -> crate::notes::Course {
        use course2md::{
            artifact,
            fetch::VideoMeta,
            timeline::{Section, TranscriptEvent},
        };
        let work = root.join(format!("work-{revision}"));
        std::fs::create_dir_all(work.join("frames")).unwrap();
        let sections = times
            .iter()
            .enumerate()
            .map(|(index, time)| {
                let relative = format!("frames/frame-{index}.png");
                image::RgbImage::new(4 + revision as u32, 3)
                    .save(work.join(&relative))
                    .unwrap();
                Section {
                    t: *time,
                    end: *time + 10.,
                    image: relative,
                    speech: vec![TranscriptEvent {
                        start: *time,
                        end: *time + 10.,
                        text: format!("第 {revision} 版，{time} 秒的正文"),
                        raw: None,
                    }],
                }
            })
            .collect::<Vec<_>>();
        let target = artifact::Target {
            task_id: format!("task-{revision}"),
            course_id: "reader-fixture".into(),
            source_id: "source-fixture".into(),
            version_id: format!("v{revision}"),
            course_dir: root.join("course"),
        };
        let meta = VideoMeta {
            title: "测试课程".into(),
            uploader: "作者".into(),
            duration: 90.,
            webpage_url: "https://www.bilibili.com/video/BVfixture?p=2".into(),
            extractor: "bilibili".into(),
            id: "fixture".into(),
        };
        let manifest = smol::block_on(artifact::publish(
            &target,
            &work,
            &meta,
            &sections,
            None,
            &[],
            outcomes,
        ))
        .unwrap();
        crate::notes::Course {
            dir: target.version_dir(),
            title: meta.title,
            modified: std::time::SystemTime::now(),
            slides: times.len(),
            segments: times.len(),
            thumbnail: None,
            manifest: Some(manifest),
            warning: None,
        }
    }
    fn offline_task_fixture(
        root: &std::path::Path,
        course: &Course,
    ) -> (workspace::TaskRecord, workspace::LibraryLocation) {
        let manifest = course.manifest.as_ref().unwrap();
        let library = workspace::LibraryLocation {
            id: "offline-library".into(),
            name: "Offline library".into(),
            root: root.to_owned(),
            previous_roots: Vec::new(),
        };
        let task = workspace::TaskRecord {
            id: manifest.task_id.clone(),
            plan: workspace::TaskPlan {
                operation: Default::default(),
                source: source::Source {
                    input: "https://www.bilibili.com/video/BVfixture?p=2".into(),
                    identity: manifest.source_id.clone(),
                    online: true,
                    ..Default::default()
                },
                source_id: manifest.source_id.clone(),
                title: course.title.clone(),
                library_id: library.id.clone(),
                folder: None,
                options: ConversionOptions {
                    keep_video: true,
                    ..Default::default()
                },
                subtitle: None,
                config: Default::default(),
                asr_service: None,
                ai_service: None,
            },
            state: workspace::TaskState::Complete,
            intent: workspace::Intent::Run,
            created: 0,
            updated: 0,
            parent: None,
            handled_by: None,
            work_dir: root.join(".course2md/work").join(&manifest.task_id),
            stages: Default::default(),
            error: None,
            artifact: Some(course.dir.clone()),
            outcomes: None,
            unread: false,
            logs: Vec::new(),
            blocked: Vec::new(),
            resend: Vec::new(),
        };
        (task, library)
    }

    #[test]
    fn offline_video_uses_its_version_task_and_rechecks_removed_media() {
        let root = tempfile::tempdir().unwrap();
        let first = fixture_version(root.path(), 1, &[10.]);
        let second = fixture_version(root.path(), 2, &[20.]);
        let (task, library) = offline_task_fixture(root.path(), &first);
        std::fs::create_dir_all(&task.work_dir).unwrap();
        let media = task.work_dir.join("media.mp4");
        std::fs::write(&media, "retained first video").unwrap();
        let request = OfflineVideoRequest::for_version(&first, &task, &library).unwrap();
        assert_eq!(
            request.inspect(),
            OfflineVideo::Available(media.canonicalize().unwrap())
        );
        assert!(OfflineVideoRequest::for_version(&second, &task, &library).is_none());
        let mut changed = task.clone();
        changed.artifact = Some(second.dir.clone());
        assert!(OfflineVideoRequest::for_version(&first, &changed, &library).is_none());
        changed = task.clone();
        changed.plan.source_id = "another-video".into();
        assert!(OfflineVideoRequest::for_version(&first, &changed, &library).is_none());
        changed = task.clone();
        changed.plan.options.keep_video = false;
        assert!(OfflineVideoRequest::for_version(&first, &changed, &library).is_none());
        std::fs::remove_file(media).unwrap();
        assert_eq!(request.inspect(), OfflineVideo::Missing);
    }

    #[test]
    fn offline_video_refuses_other_task_paths_and_files_outside_its_library() {
        let root = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let course = fixture_version(root.path(), 1, &[10.]);
        let (mut task, library) = offline_task_fixture(root.path(), &course);
        let expected_work = task.work_dir.clone();
        std::fs::write(outside.path().join("media.mp4"), "unrelated video").unwrap();
        task.work_dir = outside.path().to_owned();
        let request = OfflineVideoRequest::for_version(&course, &task, &library).unwrap();
        assert_eq!(request.inspect(), OfflineVideo::Missing);
        task.work_dir = expected_work;
        std::fs::create_dir_all(&task.work_dir).unwrap();
        let mut request = OfflineVideoRequest::for_version(&course, &task, &library).unwrap();
        std::fs::write(task.work_dir.join("media.mp4"), "retained video").unwrap();
        request.version_dir = outside.path().to_owned();
        assert_eq!(request.inspect(), OfflineVideo::Missing);

        #[cfg(unix)]
        {
            std::fs::remove_file(task.work_dir.join("media.mp4")).unwrap();
            std::os::unix::fs::symlink(
                outside.path().join("media.mp4"),
                task.work_dir.join("media.mp4"),
            )
            .unwrap();
            let request = OfflineVideoRequest::for_version(&course, &task, &library).unwrap();
            assert_eq!(request.inspect(), OfflineVideo::Missing);
        }
    }

    #[test]
    fn summary_failure_keeps_body_readable_and_does_not_offer_file_reload() {
        let root = tempfile::tempdir().unwrap();
        let detail = "摘要尚未完成，已保留正文 / Summary incomplete; body retained";
        let course = fixture_with_outcomes(
            root.path(),
            1,
            &[10.],
            course2md::artifact::Outcomes {
                transcript: course2md::artifact::Outcome::succeeded(),
                screenshots: course2md::artifact::Outcome::succeeded(),
                summary: course2md::artifact::Outcome::failed(detail),
                ..Default::default()
            },
        );
        let preview = crate::notes::read_preview(course.clone()).unwrap();
        let data = load_reader_data(&preview);
        assert!(preview.plain_text.contains("10 秒的正文"));
        assert!(preview.issues.is_empty());
        assert_eq!(
            processing_notice(&preview.processing_issues).as_deref(),
            Some("正文已保存，摘要未完成。")
        );
        assert_eq!(
            preview.processing_issues[0].outcome.message.as_deref(),
            Some(detail)
        );
        assert_eq!(preview.course.manifest.as_ref().unwrap().task_id, "task-1");
        assert!(!files_need_reload(&preview, &data.issues, &data.frames));

        std::fs::remove_file(course.dir.join("frames/frame-0.png")).unwrap();
        let damaged = crate::notes::read_preview(course).unwrap();
        let data = load_reader_data(&damaged);
        assert!(files_need_reload(&damaged, &data.issues, &data.frames));
        assert_eq!(
            processing_notice(&damaged.processing_issues).as_deref(),
            Some("正文已保存，摘要未完成。")
        );
    }
    #[test]
    fn a_missing_earlier_frame_does_not_shift_later_times_or_mix_versions() {
        let root = tempfile::tempdir().unwrap();
        let old = fixture_version(root.path(), 1, &[10., 20., 30.]);
        let new = fixture_version(root.path(), 2, &[40.]);
        std::fs::remove_file(old.dir.join("frames/frame-0.png")).unwrap();
        let old_preview = crate::notes::read_preview(old.clone()).unwrap();
        assert_eq!(old_preview.frames.len(), 2);
        let old_data = load_reader_data(&old_preview);
        assert_eq!(old_data.frames.len(), 3);
        assert!(old_data.frames[0].path.is_none());
        assert_eq!(old_data.frames[1].seconds, Some(20.));
        assert!(old_data.frames[1].transcript.contains("20 秒"));
        assert_eq!(old_data.frames[1].body_anchor.as_deref(), Some("section-1"));
        assert_eq!(old_data.versions.len(), 2);
        let new_data = load_reader_data(&crate::notes::read_preview(new.clone()).unwrap());
        assert_eq!(new_data.frames.len(), 1);
        assert_eq!(new_data.frames[0].seconds, Some(40.));
        assert!(
            new_data.frames[0]
                .path
                .as_ref()
                .unwrap()
                .starts_with(&new.dir)
        );
        assert!(
            !new_data.frames[0]
                .path
                .as_ref()
                .unwrap()
                .starts_with(&old.dir)
        );
    }
    #[test]
    fn legacy_images_keep_authored_captions_and_unknown_times() {
        let root = tempfile::tempdir().unwrap();
        image::RgbImage::new(6, 4)
            .save(root.path().join("slide.png"))
            .unwrap();
        std::fs::write(
            root.path().join("course.md"),
            "# 人工课程\n\n## 补充主题\n\n![手写图注](slide.png)\n\n这段说明来自人工原稿。\n",
        )
        .unwrap();
        let scan = crate::notes::scan_library(root.path()).unwrap();
        let preview = crate::notes::read_preview(scan.courses[0].clone()).unwrap();
        let data = load_reader_data(&preview);
        assert_eq!(data.frames.len(), 1);
        assert_eq!(data.frames[0].seconds, None);
        assert_eq!(data.frames[0].caption.as_deref(), Some("手写图注"));
        assert!(data.frames[0].transcript.contains("人工原稿"));
        assert!(data.frames[0].path.is_some());
        assert!(
            data.frames[0]
                .body_anchor
                .as_deref()
                .unwrap()
                .starts_with("legacy-")
        );
    }
}
