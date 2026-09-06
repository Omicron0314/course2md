//! Course organization and presentation are independent, persistent choices.
use super::*;
use crate::theme::*;
use gpui_component::{
    button::*,
    menu::{DropdownMenu, PopupMenuItem},
};

impl Desktop {
    pub fn library_page(&mut self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let columns = ((f32::from(window.bounds().size.width) - 272.) / 264.)
            .floor()
            .max(1.) as usize;
        let query = self.value(Field::Search, cx).to_lowercase();
        let courses: Vec<_> = self
            .courses
            .iter()
            .enumerate()
            .filter(|(_, course)| {
                course.title.to_lowercase().contains(&query)
                    && self.folder_filter.is_none_or(|id| {
                        self.library
                            .folder(&self.library_root, &course.dir)
                            .unwrap_or(0)
                            == id
                    })
            })
            .map(|(index, course)| (index, course.clone()))
            .collect();
        let mut view = v_flex().w_full().gap_6();
        if self.loading {
            return view.child("正在读取课程…").into_any_element();
        }
        if courses.is_empty() {
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
                        .child(div().text_lg().font_weight(FontWeight::SEMIBOLD).child(
                            if !query.is_empty() {
                                "没有匹配的课程"
                            } else if self.folder_filter.is_some() {
                                "这个文件夹还没有课程"
                            } else {
                                "从第一门课程开始"
                            },
                        ))
                        .child(div().text_color(rgb(MUTED)).child(if query.is_empty() {
                            "添加视频链接或本地视频，生成课程笔记。"
                        } else {
                            "试试更短的关键词。"
                        }))
                        .child(if query.is_empty() {
                            control("empty-library-add")
                                .primary()
                                .label("添加课程")
                                .on_click(
                                    cx.listener(|this, _, window, cx| this.begin_add(window, cx)),
                                )
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
            let mut groups: BTreeMap<u64, Vec<(usize, Course)>> = BTreeMap::new();
            for entry in courses {
                groups
                    .entry(
                        self.library
                            .folder(&self.library_root, &entry.1.dir)
                            .unwrap_or(0),
                    )
                    .or_default()
                    .push(entry);
            }
            // Named folders first, unfiled last; folder names remain stable across view changes.
            let mut ids: Vec<_> = groups.keys().copied().collect();
            ids.sort_by_key(|id| {
                (
                    *id == 0,
                    self.library.folders.get(id).cloned().unwrap_or_default(),
                )
            });
            for id in ids {
                let entries = &groups[&id];
                let collapsed = self.collapsed_folders.contains(&id);
                let name = self
                    .library
                    .folders
                    .get(&id)
                    .cloned()
                    .unwrap_or_else(|| "未分类".into());
                let mut group = v_flex().gap_3().child(
                    control(("library-group", id as usize))
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
                            if !this.collapsed_folders.remove(&id) {
                                this.collapsed_folders.insert(id);
                            }
                            cx.notify();
                        })),
                );
                let collection = self.course_collection(entries, columns, cx);
                group = group.child(disclosure(
                    ("folder-disclosure", id as usize),
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
        h_flex()
            .w_full()
            .gap_3()
            .pb_4()
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
                            PopupMenuItem::new("删除文件夹…").on_click(move |_, _, cx| {
                                let _ = remove.update(cx, |this, cx| {
                                    this.delete_folder = Some(id);
                                    this.folder_editor = None;
                                    cx.notify();
                                });
                            }),
                        )
                    },
                ))
            })
            .child(
                div().flex_1().min_w(px(120.)).child(
                    Input::new(&self.inputs[&Field::Search])
                        .aria_label("搜索课程")
                        .min_h(px(36.))
                        .max_h(px(36.)),
                ),
            )
            .child(
                h_flex()
                    .gap_2()
                    .flex_shrink_0()
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
                                cx.notify();
                            })),
                        )
                    })
                    .child(
                        h_flex().gap_1().children(
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
            )
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
                                .flex_1()
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
                                                    div().text_sm().text_color(rgb(MUTED)).child(
                                                        format!(
                                                            "{} 段笔记 · {} 张截图",
                                                            course.segments, course.slides
                                                        ),
                                                    ),
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
                                            .h(px(44.))
                                            .min_h(px(44.))
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
                                    .child(div().text_sm().text_color(rgb(MUTED)).child(format!(
                                        "{} 段笔记 · {} 张截图",
                                        course.segments, course.slides
                                    )))
                                    .child(self.folder_picker(
                                        Some(course.dir.clone()),
                                        index + 1,
                                        cx,
                                    )),
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
