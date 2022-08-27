use std::path::Path;
use std::rc::Rc;

use vizia::prelude::*;

mod keymap;
use keymap::*;
use vizia::state::{Index, Then};

use crate::ui::browser_node_derived_lenses::children;
use crate::ui::state::{BrowserEvent, BrowserNode, BrowserState, PanelEvent, PanelState};
use crate::ui::{Panel, ResizableStack, UiData, UiEvent, UiState};

// A simple file browser.
pub fn browser(cx: &mut Context) {
    // For testing purposes this event is emitted on browser creation to trigger the browser state to update.
    cx.emit(BrowserEvent::ViewAll);

    browser_keymap(cx);

    HStack::new(cx, |cx| {
        // Placeholder for Left Bar
        VStack::new(cx, |cx| {
            Element::new(cx).class("level4").size(Pixels(32.0)).bottom(Pixels(1.0));

            Element::new(cx).class("level2").size(Pixels(32.0));

            Element::new(cx).class("level3").size(Pixels(32.0));

            Element::new(cx).class("level2").size(Pixels(32.0));

            Element::new(cx).class("level2").size(Pixels(32.0));

            Element::new(cx).class("level2").size(Pixels(32.0));
        })
        .width(Pixels(32.0))
        .class("level2");

        // Browser
        // A resizable stack so that the user can change the width of the browser panel.
        // Resizing the panel smaller than a certain size will collapse the panel (see panels state).
        ResizableStack::new(
            cx,
            UiData::state.then(UiState::panels.then(PanelState::browser_width)),
            |cx, width| {
                cx.emit(PanelEvent::SetBrowserWidth(width));
            },
            |cx| {
                // The actual browser panel
                Panel::new(
                    cx,
                    |cx| {
                        // Header
                        Label::new(cx, "BROWSER").text_wrap(false).class("small");
                        Label::new(cx, "BROWSE2").on_release(|cx| {
                            if let Some(folder_path) = rfd::FileDialog::new().pick_folder() {
                                cx.emit(BrowserEvent::SetRoot(folder_path.clone()));
                            }
                        });
                    },
                    |cx| {
                        // search box stuff
                        Label::new(cx, "SEARCH"); //.text_wrap(false).class("small");
                        Textbox::new(
                            cx,
                            UiData::state
                                .then(UiState::browser.then(BrowserState::search_expression)),
                        )
                        .on_submit(|cx, text, _| {
                            cx.emit(BrowserEvent::SetSearchExpression(text));
                        })
                        .height(Auto);

                        // The tree view of files in the browser in constructed recursively from the root file.
                        // Bind to the root file so that if it changes the tree view will be rebuilt.
                        // TODO: Add more levels for the treeview
                        ScrollView::new(cx, 0.0, 0.0, false, false, |cx| {
                            treeview(
                                cx,
                                UiData::state.then(UiState::browser.then(BrowserState::root_node)),
                                0,
                                directory_header,
                                |cx, item, level| {
                                    treeview(
                                        cx,
                                        item,
                                        level,
                                        directory_header,
                                        |cx, item, level| {
                                            treeview(
                                                cx,
                                                item,
                                                level,
                                                directory_header,
                                                |cx, item, level| {
                                                    treeview(
                                                        cx,
                                                        item,
                                                        level,
                                                        directory_header,
                                                        |cx, item, level| {
                                                            treeview(
                                                                cx,
                                                                item,
                                                                level,
                                                                directory_header,
                                                                file,
                                                            );
                                                        },
                                                    );
                                                },
                                            );
                                        },
                                    );
                                },
                            );
                        })
                        .class("level3");
                    },
                )
                .display(
                    UiData::state
                        .then(UiState::panels.then(PanelState::hide_browser.map(|flag| !flag))),
                );
            },
        )
        .class("browser")
        .toggle_class("hidden", UiData::state.then(UiState::panels.then(PanelState::hide_browser)));
    })
    .width(Auto)
    .class("level1");
}

fn directory_header<L>(cx: &mut Context, root: L, level: u32)
where
    L: Lens<Target = BrowserNode>,
{
    Binding::new(
        cx,
        root.clone().then(BrowserNode::children).map(|items| items.len()),
        move |cx, num_items| {
            let num_children = num_items.get(cx);
            if num_children == 0 {
                file(cx, root.clone(), level);
            } else {
                directory(cx, root.clone(), level);
            }
        },
    );
}

/// It creates a new binding that creates a new HStack that creates a new Label that creates a new Label
/// Arguments:
///
/// * `cx`: &mut Context
/// * `root`: The root node of the tree.
/// * `level`: u32
fn directory<L>(cx: &mut Context, root: L, level: u32)
where
    L: Lens<Target = BrowserNode>,
{
    Binding::new(cx, root.clone().then(BrowserNode::file_path), move |cx, file_path| {
        let file_path1 = file_path.get(cx);
        let file_path2 = file_path.get(cx);
        let file_path3 = file_path.get(cx);

        HStack::new(cx, |cx| {
            //Icon::new(cx, IconCode::Dropdown, 24.0, 23.0)
            // Arrow Icon
            Label::new(cx, "\u{e75c}")
                .font("icon")
                .height(Stretch(1.0))
                .child_top(Stretch(1.0))
                .child_bottom(Stretch(1.0))
                .hoverable(false)
                .rotate(root.clone().then(BrowserNode::is_open).map(|flag| {
                    if *flag {
                        0.0
                    } else {
                        -90.0
                    }
                }));
            // File or directory name
            Label::new(cx, root.clone().then(BrowserNode::name))
                .width(Stretch(1.0))
                .text_wrap(false)
                .hoverable(false);
        })
        .cursor(CursorIcon::Hand)
        .class("dir-file")
        .toggle_class(
            "focused",
            UiData::state.then(UiState::browser.then(BrowserState::selected.map(
                move |selected| match (&file_path1, selected) {
                    (Some(fp), Some(s)) => s.starts_with(fp),

                    _ => false,
                },
            ))),
        )
        .toggle_class(
            "selected",
            UiData::state.then(
                UiState::browser
                    .then(BrowserState::selected.map(move |selected| &file_path2 == selected)),
            ),
        )
        .on_press(move |cx| {
            cx.focus();
            if let Some(file_path) = &file_path3 {
                cx.emit(BrowserEvent::SetSelected(file_path.clone()));
                cx.emit(BrowserEvent::ToggleOpen);
            }
        })
        .col_between(Pixels(4.0))
        .child_left(Pixels(15.0 * level as f32 + 5.0));
    });
}

/// Constructs a `Label` which represents a file in the browser ui
///
/// Arguments:
///
/// * `cx`: &mut `Context` - the vizia context
/// * `node_lens`: `Lens` pointing to a [`BrowserNode`]
/// * `level`: `u32`
fn file<L>(cx: &mut Context, node_lens: L, level: u32)
where
    L: Lens<Target = BrowserNode>,
{
    let node = node_lens.get(cx);
    if node.is_visible {
        Binding::new(cx, node_lens.clone().then(BrowserNode::is_visible), move |cx, is_visible| {
            let file_path1 = node_lens.get(cx).file_path;
            let file_path2 = node_lens.get(cx).file_path;
            let file_path3 = node_lens.get(cx).file_path;

            Label::new(cx, node_lens.clone().then(BrowserNode::name))
                .class("dir-file")
                .width(Stretch(1.0))
                .text_wrap(false)
                .cursor(CursorIcon::Hand)
                .child_left(Pixels(15.0 * level as f32 + 5.0))
                .toggle_class(
                    "focused",
                    UiData::state.then(UiState::browser.then(BrowserState::selected.map(
                        move |selected| match (&file_path1, selected) {
                            (Some(fp), Some(s)) => s.starts_with(fp),
                            _ => false,
                        },
                    ))),
                )
                .toggle_class(
                    "selected",
                    UiData::state.then(
                        UiState::browser.then(
                            BrowserState::selected.map(move |selected| &file_path2 == selected),
                        ),
                    ),
                )
                .on_press(move |cx| {
                    cx.focus();
                    if let Some(file_path) = &file_path3 {
                        cx.emit(UiEvent::BrowserFileClicked(file_path.clone()));
                        cx.emit(BrowserEvent::SetSelected(file_path.clone()));
                    }
                });
        });
    }
}

/// `treeview` takes a `BrowserNode` and a level, and displays the node and its children
///
/// Arguments:
///
/// * `cx`: &mut Context,
/// * `root_file`: The root file to start the treeview from.
/// * `level`: the level of the treeview
/// * `header`: a function that takes a Context, a Lens, and a level, and returns a Widget. This is the
/// widget that will be displayed for the root node.
/// * `content`: This is the function that will be called for each file in the directory.
fn treeview<L>(
    cx: &mut Context,
    root_file: L,
    level: u32,
    header: impl Fn(&mut Context, L, u32),
    content: impl Fn(&mut Context, Then<Then<L, children>, Index<Vec<BrowserNode>, BrowserNode>>, u32)
        + 'static,
) where
    L: Lens<Target = BrowserNode>,
    L::Source: Model,
{
    let content = Rc::new(content);
    VStack::new(cx, |cx| {
        // Label::new(cx, lens.clone().then(File::name));
        (header)(cx, root_file.clone(), level);

        Binding::new(cx, root_file.clone().then(BrowserNode::is_open), move |cx, is_open| {
            if is_open.get(cx) {
                let content1 = content.clone();
                let root_file2 = root_file.clone();
                //let search_expression = search_expression.clone();

                VStack::new(cx, |cx| {
                    // list of files in the browser

                    let file_children = root_file2.clone().then(BrowserNode::children);
                    let content2 = content1.clone();

                    List::new(cx, file_children, move |cx, index, item| {
                        let file = item.clone();

                        (content2.clone())(cx, file, level + 1);
                    })
                    .height(Auto);

                    let root_file3 = root_file.clone();
                    let file_path1 = root_file3.clone().get(cx).file_path.clone();

                    // lines drawn in front of directories to show focus
                    Element::new(cx)
                        .left(Pixels(15.0 * (level + 1) as f32 - 5.0))
                        .height(Stretch(1.0))
                        .width(Pixels(2.0))
                        .position_type(PositionType::SelfDirected)
                        .display(root_file3.then(BrowserNode::is_open))
                        .class("dir-line")
                        .toggle_class(
                            "focused",
                            UiData::state.then(UiState::browser.then(BrowserState::selected.map(
                                move |selected| {
                                    if let Some(path) = &file_path1 {
                                        if let Some(selected) = selected {
                                            if let Some(dir) = dir_path(selected) {
                                                dir == path
                                            } else {
                                                false
                                            }
                                        } else {
                                            false
                                        }
                                    } else {
                                        false
                                    }
                                },
                            ))),
                        );
                })
                .height(Auto);
                //.display(root.clone().then(File::is_open));
            }
        });
    })
    .height(Auto);
}

/// It takes a `BrowserNode` and a `&str` and returns a `bool`
///
/// Arguments:
///
/// * `file`: BrowserNode - This is the file that we are currently searching.
/// * `search_expression`: The string that we're searching for.
///
/// Returns:
///
/// A boolean value.
fn is_valid_directory(file: BrowserNode, search_expression: &str) -> bool {
    if file.name.to_lowercase().contains(&search_expression.to_lowercase()) {
        true
    } else {
        for child in file.children {
            is_valid_directory(child, &search_expression.to_owned());
        }
        false
    }
}

/// It takes a file and a search expression and returns true if the file's name contains the search
/// expression
///
/// Arguments:
///
/// * `file`: BrowserNode - This is the file that we're checking to see if it's valid.
/// * `search_expression`: The string that the user has entered into the search box.
///
/// Returns:
///
/// A boolean value.
fn is_valid_file(file: BrowserNode, search_expression: &str) -> bool {
    file.name.to_lowercase().contains(&search_expression.to_lowercase())
}

/// Checking if the path is a directory. If it is, it returns the path. If it is not, it returns the
/// parent directory.
fn dir_path(path: &Path) -> Option<&Path> {
    if path.is_dir() {
        Some(path)
    } else {
        path.parent()
    }
}
