//! Course organization and presentation are independent, persistent choices.
use super::*;
use crate::theme::*;
use gpui_component::{
    button::*,
    menu::{DropdownMenu, PopupMenuItem},
};

struct CourseRenameDialog {
    desktop: Entity<Desktop>,
    input: Entity<InputState>,
    course: Course,
    root: PathBuf,
    error: Option<String>,
    _subscription: Subscription,
}

impl CourseRenameDialog {
    fn save(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let name = self.input.read(cx).value().to_string();
        match organize::rename_course(&self.root, &self.course.storage_dir(), &name) {
            Ok(()) => {
                self.desktop.update(cx, |desktop, cx| {
                    desktop.apply_course_title_aliases();
                    desktop.message = Some(format!("笔记已更名为「{}」。", name.trim()));
                    cx.notify();
                });
                window.close_dialog(cx);
            }
            Err(error) => {
                self.error = Some(format!("{error:#}"));
                self.input.update(cx, |state, cx| state.focus(window, cx));
                cx.notify();
            }
        }
    }
}

impl Render for CourseRenameDialog {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .id("rename-note-dialog")
            .role(Role::Dialog)
            .aria_label("重命名笔记")
            .gap_3()
            .child(accessible_text("rename-note-label", "笔记名称"))
            .child(Input::new(&self.input).aria_label("笔记名称"))
            .when_some(self.error.clone(), |view, error| {
                view.child(
                    accessible_text("rename-note-error", error)
                        .role(Role::Alert)
                        .text_color(rgb(0xa32626)),
                )
            })
            .child(
                accessible_text(
                    "rename-note-scope",
                    "更改在课程库中显示的名称，已生成的正文和原视频保持完整。",
                )
                .text_sm()
                .text_color(rgb(MUTED)),
            )
            .child(
                h_flex()
                    .justify_end()
                    .gap_2()
                    .child(
                        control("cancel-note-rename")
                            .label("取消")
                            .on_click(|_, window, cx| window.close_dialog(cx)),
                    )
                    .child(
                        control("save-note-rename")
                            .primary()
                            .label("保存名称")
                            .on_click(cx.listener(|this, _, window, cx| this.save(window, cx))),
                    ),
            )
    }
}

impl Desktop {
    pub fn course_display_title(&self, course: &Course) -> String {
        self.courses
            .iter()
            .find(|item| item.storage_dir() == course.storage_dir())
            .map(|item| item.title.clone())
            .unwrap_or_else(|| course.title.clone())
    }

    /// Call after a library scan, and after a name edit. Each library's small index
    /// is read once; the published notes themselves are left byte-for-byte intact.
    pub fn apply_course_title_aliases(&mut self) {
        let locations = self
            .workspace
            .as_ref()
            .map(|workspace| workspace.state.libraries.clone())
            .unwrap_or_default();
        let mut aliases = BTreeMap::new();
        for location in locations {
            match organize::title_aliases(&location.root) {
                Ok(names) => {
                    aliases.insert(location.root, names);
                }
                Err(error) => self
                    .library_issues
                    .push(format!("{}的显示名称尚未读取：{error:#}", location.name)),
            }
        }
        for course in &mut self.courses {
            let storage = course.storage_dir();
            if let Some((_, names, relative)) = aliases
                .iter()
                .filter_map(|(root, names)| {
                    organize::relative_key(root, &storage)
                        .ok()
                        .map(|relative| (root, names, relative))
                })
                .max_by_key(|(root, _, _)| root.components().count())
            {
                if let Some(name) = names.get(&relative) {
                    course.title = name.clone();
                }
            }
        }
        let title = self
            .preview
            .as_ref()
            .map(|preview| self.course_display_title(&preview.course));
        if let (Some(preview), Some(title)) = (&mut self.preview, title) {
            preview.course.title = title;
        }
    }

    pub fn begin_course_rename(
        &mut self,
        course: Course,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(root) = self
            .course_location(&course)
            .map(|library| library.root.clone())
        else {
            self.message = Some("这份笔记的保存位置暂时无法确认，请重新检查课程库。".into());
            cx.notify();
            return;
        };
        let name = self.course_display_title(&course);
        let input = cx.new(|cx| InputState::new(window, cx));
        input.update(cx, |state, cx| state.set_value(name, window, cx));
        let desktop = cx.entity();
        let focus = input.clone();
        let content = cx.new(|cx| {
            let subscription = cx.subscribe_in(
                &input,
                window,
                |this: &mut CourseRenameDialog, _, event, window, cx| {
                    if matches!(event, InputEvent::PressEnter { .. }) {
                        this.save(window, cx);
                    }
                    if matches!(event, InputEvent::Change) {
                        this.error = None;
                        cx.notify();
                    }
                },
            );
            CourseRenameDialog {
                desktop,
                input,
                course,
                root,
                error: None,
                _subscription: subscription,
            }
        });
        window.open_dialog(cx, move |dialog, _, _| {
            dialog
                .title("重命名笔记")
                .w(px(480.))
                .overlay_closable(false)
                .child(content.clone())
        });
        window.defer(cx, move |window, cx| {
            focus.update(cx, |state, cx| state.focus(window, cx))
        });
    }

    fn course_actions(&self, course: Course, index: usize, cx: &mut Context<Self>) -> AnyElement {
        let entity = cx.entity().downgrade();
        control(("course-actions", index))
            .ghost()
            .label("笔记操作")
            .accessibility_label(format!("《{}》的笔记操作", course.title))
            .dropdown_menu(move |menu, _, _| {
                let rename = entity.clone();
                let selected = course.clone();
                let path = course.storage_dir();
                let mut menu = menu
                    .item(
                        PopupMenuItem::new("重命名笔记…").on_click(move |_, window, cx| {
                            let _ = rename.update(cx, |this, cx| {
                                this.begin_course_rename(selected.clone(), window, cx)
                            });
                        }),
                    )
                    .item(
                        PopupMenuItem::new("打开笔记保存位置")
                            .on_click(move |_, _, cx| cx.open_with_system(&path)),
                    );
                for (format, label) in [
                    (course2md::config::OutputFormat::Md, "导出 Markdown 包…"),
                    (course2md::config::OutputFormat::Html, "导出网页文件…"),
                    (course2md::config::OutputFormat::Json, "导出结构化数据…"),
                ] {
                    let entity = entity.clone();
                    let course = course.clone();
                    menu = menu.item(PopupMenuItem::new(label).on_click(move |_, window, cx| {
                        let _ = entity.update(cx, |this, cx| {
                            this.export_course(course.clone(), format, window, cx)
                        });
                    }));
                }
                menu
            })
            .into_any_element()
    }

    fn recover_library_folders(&mut self, root: PathBuf, rebuild: bool, cx: &mut Context<Self>) {
        match organize::recover(&root, rebuild) {
            Ok((library, preserved)) => {
                self.library_indexes.insert(root.clone(), library.clone());
                if self.library_root == root {
                    self.library = library;
                    self.library_error = None;
                }
                self.message = Some(format!(
                    "分类已{}，原损坏记录保留在 {}。笔记正文和显示名称仍保留。",
                    if rebuild { "重建" } else { "恢复" },
                    preserved.display()
                ));
                self.refresh_library(cx);
            }
            Err(error) => self.message = Some(format!("分类尚未恢复：{error:#}")),
        }
        cx.notify();
    }

    fn library_recovery_view(
        &self,
        access: &crate::storage::LibraryAccess,
        cx: &mut Context<Self>,
    ) -> Div {
        let mut view = v_flex().gap_3();
        let Some(workspace) = &self.workspace else {
            return view;
        };
        for (index, location) in workspace.state.libraries.iter().enumerate() {
            if access.unavailable.contains(&location.root) {
                let name = location.name.clone();
                let root = location.root.clone();
                view = view.child(
                    v_flex()
                        .gap_2()
                        .child(accessible_text(
                            ("library-unavailable", index),
                            format!(
                                "{name} 的保存位置暂时无法访问：{}。重新连接后，笔记会继续显示。",
                                root.display()
                            ),
                        ))
                        .child(
                            control(("retry-library-location", index))
                                .label("重新检查保存位置")
                                .on_click(cx.listener(|this, _, _, cx| this.refresh_library(cx))),
                        ),
                );
                continue;
            }
            if let Ok(Some(recovery)) = organize::recovery(&location.root) {
                let restore = location.root.clone();
                let rebuild = location.root.clone();
                let folder = location.root.clone();
                let mut row = h_flex().gap_2().flex_wrap();
                if recovery.has_backup {
                    row = row.child(
                        control(("restore-classification", index))
                            .label("恢复最近分类备份")
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.recover_library_folders(restore.clone(), false, cx)
                            })),
                    );
                }
                row = row
                    .child(
                        control(("rebuild-classification", index))
                            .label("保留损坏记录并重建分类")
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.recover_library_folders(rebuild.clone(), true, cx)
                            })),
                    )
                    .child(
                        control(("show-classification-files", index))
                            .ghost()
                            .label("打开保存位置")
                            .on_click(move |_, _, cx| cx.open_with_system(&folder)),
                    );
                view = view.child(v_flex().gap_2().child(accessible_text(("classification-recovery", index), format!("{} 的分类记录无法读取。现有笔记文件会保留；重建分类后可重新整理到文件夹。", location.name)))
                    .child(row));
            }
            if let Ok(Some(recovery)) = organize::title_recovery(&location.root) {
                let mut row = h_flex().gap_2().flex_wrap();
                for reset in [false, true] {
                    if !reset && !recovery.has_backup {
                        continue;
                    }
                    let root = location.root.clone();
                    row = row.child(
                        control((
                            if reset {
                                "reset-note-names"
                            } else {
                                "restore-note-names"
                            },
                            index,
                        ))
                        .label(if reset {
                            "保留损坏记录并恢复原名称"
                        } else {
                            "恢复最近名称备份"
                        })
                        .on_click(cx.listener(move |this, _, _, cx| {
                            match organize::recover_titles(&root, reset) {
                                Ok(path) => {
                                    this.message = Some(format!(
                                        "笔记名称已恢复，损坏记录保留在 {}。正文和分类保持完整。",
                                        path.display()
                                    ));
                                    this.refresh_library(cx);
                                }
                                Err(error) => {
                                    this.message = Some(format!("名称尚未恢复：{error:#}"))
                                }
                            }
                            cx.notify();
                        })),
                    );
                }
                view = view.child(
                    v_flex()
                        .gap_2()
                        .child(accessible_text(
                            ("title-recovery", index),
                            format!(
                                "{} 的笔记名称记录无法读取，现有正文文件会保留。",
                                location.name
                            ),
                        ))
                        .child(row),
                );
            }
        }
        view
    }

    pub fn library_page(&mut self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let columns = ((f32::from(window.bounds().size.width) - 272.)
            / (264. * self.preferences.application().font_scale))
            .floor()
            .max(1.) as usize;
        let query = self.value(Field::Search, cx).to_lowercase();
        let roots = self
            .workspace
            .as_ref()
            .map(|workspace| {
                workspace
                    .state
                    .libraries
                    .iter()
                    .map(|library| library.root.clone())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_else(|| vec![self.library_root.clone()]);
        let all_access = crate::storage::library_access(roots);
        let in_scope =
            |root: &&PathBuf| self.folder_filter.is_none() || **root == self.library_root;
        let scope = crate::storage::LibraryAccess {
            available: all_access
                .available
                .iter()
                .filter(in_scope)
                .cloned()
                .collect(),
            unavailable: all_access
                .unavailable
                .iter()
                .filter(in_scope)
                .cloned()
                .collect(),
        };
        let coverage = scope.coverage();
        let courses: Vec<_> = self
            .courses
            .iter()
            .enumerate()
            .filter(|(_, course)| {
                self.course_location(course)
                    .map(|location| scope.available.contains(&location.root))
                    .unwrap_or_else(|| {
                        self.workspace.is_none()
                            && scope.available.iter().any(|root| {
                                organize::relative_key(root, &course.storage_dir()).is_ok()
                            })
                    })
                    && course.title.to_lowercase().contains(&query)
                    && self.folder_filter.is_none_or(|id| {
                        self.course_location(course)
                            .is_some_and(|location| location.root == self.library_root)
                            && (!self.library_indexes.contains_key(&self.library_root)
                                || self.course_folder(course).unwrap_or(0) == id)
                    })
            })
            .map(|(index, course)| (index, course.clone()))
            .collect();
        let mut view = v_flex()
            .w_full()
            .gap_6()
            .child(self.library_recovery_view(&all_access, cx));
        if coverage != crate::storage::LibraryCoverage::Complete {
            let message = if coverage == crate::storage::LibraryCoverage::Unavailable {
                if query.is_empty() {
                    "当前范围的保存位置暂时无法访问，笔记列表尚未读取。重新连接后可继续浏览。"
                        .to_owned()
                } else {
                    "当前范围的保存位置暂时无法访问，尚未搜索笔记。关键词已保留，重新连接后可继续搜索。".to_owned()
                }
            } else if query.is_empty() {
                format!(
                    "当前仅显示 {} 个可访问位置中的笔记；另外 {} 个位置尚未读取。",
                    scope.available.len(),
                    scope.unavailable.len()
                )
            } else {
                format!(
                    "当前仅搜索 {} 个可访问的保存位置；另外 {} 个位置尚未纳入搜索结果。",
                    scope.available.len(),
                    scope.unavailable.len()
                )
            };
            view = view.child(
                accessible_text("library-search-coverage", message)
                    .text_sm()
                    .text_color(rgb(MUTED)),
            );
        }
        if !self.library_issues.is_empty()
            && coverage != crate::storage::LibraryCoverage::Unavailable
        {
            view = view.child(
                v_flex()
                    .gap_2()
                    .p_3()
                    .bg(rgb(0xfff0db))
                    .child(accessible_text(
                        "library-issues-title",
                        "有笔记或分类记录暂时无法读取。列表和搜索结果仅包含已读取的笔记。",
                    ))
                    .children(
                        self.library_issues
                            .iter()
                            .enumerate()
                            .map(|(index, issue)| {
                                accessible_text(("library-issue", index), issue.clone()).text_sm()
                            }),
                    ),
            );
        }
        let materials = self
            .library_materials
            .iter()
            .filter(|path| {
                path.is_dir()
                    && scope
                        .available
                        .iter()
                        .any(|root| organize::relative_key(root, path).is_ok())
            })
            .cloned()
            .collect::<Vec<_>>();
        if !materials.is_empty() {
            view = view.child(
                accessible_text(
                    "library-materials-state",
                    format!(
                        "发现 {} 份尚无可读正文的历史任务材料。原文件已保留。",
                        materials.len()
                    ),
                )
                .text_sm()
                .text_color(rgb(MUTED)),
            );
            view = view.child(
                control("open-library-materials")
                    .self_start()
                    .label("查看保留的任务材料")
                    .dropdown_menu(move |menu, _, _| {
                        materials.iter().fold(menu, |menu, path| {
                            let path = path.clone();
                            menu.item(
                                PopupMenuItem::new(path.display().to_string())
                                    .on_click(move |_, _, cx| cx.open_with_system(&path)),
                            )
                        })
                    }),
            );
        }
        if self.loading {
            return view
                .child(accessible_text("library-loading", "正在读取课程…"))
                .into_any_element();
        }
        if coverage == crate::storage::LibraryCoverage::Unavailable {
            return view.into_any_element();
        }
        if courses.is_empty() {
            if query.is_empty()
                && (coverage == crate::storage::LibraryCoverage::Partial
                    || !self.library_materials.is_empty()
                    || !self.library_issues.is_empty())
            {
                return view.into_any_element();
            }
            return view
                .child(
                    v_flex()
                        .py_12()
                        .gap_4()
                        .items_center()
                        .child(
                            Icon::new(IconName::BookOpen)
                                .size(px(32.))
                                .text_color(rgb(MUTED)),
                        )
                        .child(
                            accessible_text(
                                "library-empty-heading",
                                if !query.is_empty() {
                                    if coverage == crate::storage::LibraryCoverage::Partial
                                        || !self.library_issues.is_empty()
                                    {
                                        "已读取的笔记中没有匹配的课程"
                                    } else {
                                        "没有匹配的课程"
                                    }
                                } else if self.folder_filter.is_some() {
                                    "这个文件夹还没有课程"
                                } else {
                                    "从第一门课程开始"
                                },
                            )
                            .text_lg()
                            .font_weight(FontWeight::SEMIBOLD),
                        )
                        .child(
                            accessible_text(
                                "library-empty-description",
                                if query.is_empty() {
                                    "添加视频链接或本地视频，生成课程笔记。"
                                } else if coverage == crate::storage::LibraryCoverage::Partial {
                                    "未连接的位置尚未搜索。可以重新连接保存位置，或调整关键词。"
                                } else {
                                    "试试更短的关键词。"
                                },
                            )
                            .text_color(rgb(MUTED)),
                        )
                        .child(if query.is_empty() {
                            control("empty-library-add")
                                .primary()
                                .label(if self.folder_filter.is_some_and(|id| id != 0) {
                                    "在此文件夹生成笔记"
                                } else {
                                    "生成笔记"
                                })
                                .on_click(cx.listener(|this, _, window, cx| {
                                    let inherit = this.folder_filter.is_some_and(|id| id != 0);
                                    this.new_draft(true, inherit, window, cx);
                                }))
                        } else {
                            control("clear-search")
                                .label("清空搜索")
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.inputs[&Field::Search]
                                        .update(cx, |state, cx| state.set_value("", window, cx));
                                    cx.notify();
                                }))
                        }),
                )
                .into_any_element();
        }
        if self.desktop_settings.library_group_folders
            && self.folder_filter.is_none()
            && self.library_error.is_none()
        {
            let mut groups: BTreeMap<(PathBuf, u64), Vec<(usize, Course)>> = BTreeMap::new();
            for entry in courses {
                let root = self
                    .course_location(&entry.1)
                    .map(|lib| lib.root.clone())
                    .unwrap_or_else(|| self.library_root.clone());
                groups
                    .entry((root, self.course_folder(&entry.1).unwrap_or(0)))
                    .or_default()
                    .push(entry);
            }
            for (group_index, ((root, id), entries)) in groups.into_iter().enumerate() {
                let location = self
                    .workspace
                    .as_ref()
                    .and_then(|w| w.state.libraries.iter().find(|lib| lib.root == root));
                let key = format!(
                    "{}:{id}",
                    location.map(|lib| lib.id.as_str()).unwrap_or("legacy")
                );
                let collapsed = query.is_empty()
                    && self
                        .workspace
                        .as_ref()
                        .is_some_and(|w| w.state.collapsed.contains(&key));
                let folder_name = self
                    .library_indexes
                    .get(&root)
                    .and_then(|lib| lib.folders.get(&id))
                    .cloned()
                    .unwrap_or_else(|| {
                        if self.library_indexes.contains_key(&root) {
                            "未分类"
                        } else {
                            "分类记录暂不可用"
                        }
                        .into()
                    });
                let name = if self
                    .workspace
                    .as_ref()
                    .is_some_and(|w| w.state.libraries.len() > 1)
                {
                    format!(
                        "{} · {folder_name}",
                        location.map(|lib| lib.name.as_str()).unwrap_or("课程库")
                    )
                } else {
                    folder_name
                };
                let mut group = v_flex().gap_3().child(
                    control(("library-group", group_index))
                        .ghost()
                        .w_full()
                        .justify_start()
                        .accessibility_label(format!(
                            "{} {name}，{} 门课程",
                            if collapsed { "展开" } else { "收起" },
                            entries.len()
                        ))
                        .child(
                            h_flex()
                                .w_full()
                                .gap_2()
                                .child(
                                    Icon::new(if collapsed {
                                        IconName::ChevronRight
                                    } else {
                                        IconName::ChevronDown
                                    })
                                    .size_4(),
                                )
                                .child(div().font_weight(FontWeight::SEMIBOLD).child(name))
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(rgb(MUTED))
                                        .child(entries.len().to_string()),
                                ),
                        )
                        .on_click(cx.listener(move |this, _, _, cx| {
                            if let Some(workspace) = &mut this.workspace {
                                if let Err(error) = workspace.transaction(|state| {
                                    if !state.collapsed.remove(&key) {
                                        state.collapsed.insert(key.clone());
                                    }
                                    Ok(())
                                }) {
                                    this.workspace_error =
                                        Some(format!("分组展开状态尚未保存：{error:#}"));
                                }
                            }
                            cx.notify();
                        })),
                );
                let collection = self.course_collection(&entries, columns, cx);
                group = group.child(disclosure(
                    ("folder-disclosure", group_index),
                    !collapsed,
                    collection,
                    window,
                    cx,
                ));
                view = view.child(group);
            }
        } else {
            view = view.child(self.course_collection(&courses, columns, cx));
        }
        view.into_any_element()
    }

    pub fn library_toolbar(&self, cx: &mut Context<Self>) -> Div {
        let controls = h_flex()
            .gap_2()
            .flex_wrap()
            .min_w_0()
            .max_w_full()
            .flex_shrink_0()
            .when_some(self.folder_filter.filter(|id| *id != 0), |row, id| {
                let entity = cx.entity().downgrade();
                row.child(control("manage-folder").label("管理文件夹").dropdown_menu(
                    move |menu, _, _| {
                        let rename = entity.clone();
                        let remove = entity.clone();
                        menu.item(PopupMenuItem::new("重命名").on_click(move |_, window, cx| {
                            let _ = rename
                                .update(cx, |this, cx| this.begin_folder(Some(id), window, cx));
                        }))
                        .item(
                            PopupMenuItem::new("删除文件夹…").on_click(move |_, window, cx| {
                                let _ = remove.update(cx, |this, cx| {
                                    this.begin_delete_folder(id, window, cx)
                                });
                            }),
                        )
                    },
                ))
            })
            .child(
                h_flex()
                    .gap_2()
                    .flex_wrap()
                    .min_w_0()
                    .max_w_full()
                    .when(self.folder_filter.is_none(), |row| {
                        row.child(
                            choice(
                                control("group-folders")
                                    .icon(IconName::Folder)
                                    .label("按文件夹"),
                                self.desktop_settings.library_group_folders,
                            )
                            .disabled(self.library_error.is_some())
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.desktop_settings.library_group_folders =
                                    !this.desktop_settings.library_group_folders;
                                this.save_library_presentation(cx);
                                cx.notify();
                            })),
                        )
                    })
                    .child(
                        h_flex().gap_1().flex_shrink_0().children(
                            [(false, "列表"), (true, "卡片")]
                                .into_iter()
                                .map(|(cards, label)| {
                                    choice(
                                        control(label).label(label),
                                        self.desktop_settings.library_cards == cards,
                                    )
                                    .on_click(cx.listener(
                                        move |this, _, _, cx| {
                                            this.desktop_settings.library_cards = cards;
                                            this.save_library_presentation(cx);
                                            cx.notify();
                                        },
                                    ))
                                }),
                        ),
                    )
                    .child(
                        control("refresh-library")
                            .ghost()
                            .icon(icons::refresh())
                            .accessibility_label("刷新课程库")
                            .disabled(self.loading)
                            .on_click(cx.listener(|this, _, _, cx| this.refresh_library(cx))),
                    ),
            );
        h_flex()
            .w_full()
            .min_w_0()
            .flex_wrap()
            .gap_3()
            .pb_4()
            .child(
                div()
                    .flex_1()
                    .flex_basis(rems(24.))
                    .min_w_0()
                    .max_w_full()
                    .child(
                        Input::new(&self.inputs[&Field::Search])
                            .aria_label("搜索课程")
                            .w_full()
                            .min_h(rems(2.6))
                            .h_auto(),
                    ),
            )
            .child(controls)
    }

    fn course_collection(
        &self,
        courses: &[(usize, Course)],
        columns: usize,
        cx: &mut Context<Self>,
    ) -> Div {
        if !self.desktop_settings.library_cards {
            return v_flex()
                .gap_2()
                .children(courses.iter().map(|(index, course)| {
                    h_flex()
                        .w_full()
                        .flex_wrap()
                        .min_h(px(88.))
                        .p_3()
                        .gap_4()
                        .bg(rgb(SURFACE))
                        .border_1()
                        .border_color(rgb(LINE))
                        .rounded_md()
                        .child(
                            control(("read-course", *index))
                                .ghost()
                                .w_full()
                                .min_w_0()
                                .h_auto()
                                .p_0()
                                .justify_start()
                                .accessibility_label(format!("阅读 {}", course.title))
                                .child(
                                    h_flex()
                                        .w_full()
                                        .gap_3()
                                        .child(
                                            self.course_cover(course)
                                                .w(px(96.))
                                                .h(px(54.))
                                                .flex_shrink_0(),
                                        )
                                        .child(
                                            v_flex()
                                                .flex_1()
                                                .min_w_0()
                                                .gap_1()
                                                .child(
                                                    div()
                                                        .line_clamp(2)
                                                        .font_weight(FontWeight::SEMIBOLD)
                                                        .child(course.title.clone()),
                                                )
                                                .child(
                                                    div()
                                                        .text_sm()
                                                        .text_color(rgb(MUTED))
                                                        .child(course.description()),
                                                ),
                                        ),
                                )
                                .on_click({
                                    let course = course.clone();
                                    cx.listener(move |this, _, _, cx| {
                                        this.open_course(course.clone(), cx)
                                    })
                                }),
                        )
                        .child(div().w(px(152.)).flex_shrink_0().child(self.folder_picker(
                            Some(course.dir.clone()),
                            index + 1,
                            cx,
                        )))
                        .child(self.course_actions(course.clone(), *index, cx))
                }));
        }
        v_flex()
            .gap_4()
            .children(courses.chunks(columns).map(|row| {
                h_flex()
                    .gap_4()
                    .items_stretch()
                    .children(row.iter().map(|(index, course)| {
                        v_flex()
                            .flex_1()
                            .min_w_0()
                            .bg(rgb(SURFACE))
                            .border_1()
                            .border_color(rgb(LINE))
                            .rounded_md()
                            .overflow_hidden()
                            .child(
                                control(("read-course", *index))
                                    .ghost()
                                    .w_full()
                                    .h_auto()
                                    .p_0()
                                    .aspect_ratio(16. / 9.)
                                    .accessibility_label(format!("阅读 {}", course.title))
                                    .child(self.course_cover(course).size_full())
                                    .on_click({
                                        let course = course.clone();
                                        cx.listener(move |this, _, _, cx| {
                                            this.open_course(course.clone(), cx)
                                        })
                                    }),
                            )
                            .child(
                                v_flex()
                                    .p_4()
                                    .gap_3()
                                    .child(
                                        control(("read-title", *index))
                                            .accessibility_label(format!("阅读 {}", course.title))
                                            .ghost()
                                            .w_full()
                                            .h_auto()
                                            .min_h(rems(2.6))
                                            .p_0()
                                            .justify_start()
                                            .child(
                                                div()
                                                    .w_full()
                                                    .line_clamp(2)
                                                    .font_weight(FontWeight::SEMIBOLD)
                                                    .child(course.title.clone()),
                                            )
                                            .on_click({
                                                let course = course.clone();
                                                cx.listener(move |this, _, _, cx| {
                                                    this.open_course(course.clone(), cx)
                                                })
                                            }),
                                    )
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(rgb(MUTED))
                                            .child(course.description()),
                                    )
                                    .child(self.folder_picker(
                                        Some(course.dir.clone()),
                                        index + 1,
                                        cx,
                                    ))
                                    .child(self.course_actions(course.clone(), *index, cx)),
                            )
                    }))
                    .children((row.len()..columns).map(|_| div().flex_1()))
            }))
    }

    fn course_cover(&self, course: &Course) -> Div {
        div()
            .overflow_hidden()
            .bg(rgb(SIDEBAR))
            .border_1()
            .border_color(rgba(0x0000001a))
            .flex()
            .items_center()
            .justify_center()
            .when_some(course.thumbnail.clone(), |view, path| {
                view.child(img(path).size_full().object_fit(ObjectFit::Cover))
            })
            .when(course.thumbnail.is_none(), |view| {
                view.child(
                    Icon::new(IconName::BookOpen)
                        .size_6()
                        .text_color(rgb(MUTED)),
                )
            })
    }
}
