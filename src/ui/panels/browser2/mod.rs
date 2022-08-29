use std::default;

use crate::ui::{browser_state::*, state};

use crate::ui::state::{BrowserEvent, BrowserState, PanelEvent, PanelState};
use crate::ui::{Panel, ResizableStack, UiData, UiState};
use log::info;
use state::browser_state;
use state::browser_state::node_type_derived_lenses::Directory;
use vizia::prelude::*;

pub mod browser_widgets {
    // todo better structure please, ffs boyyyy

    use log::info;
    use vizia::{
        prelude::{
            Actions, Context, Lens,
            Units::{Pixels, Stretch},
        },
        state::{Binding, LensExt, Model},
        views::{HStack, Label, List},
        window::CursorIcon,
    };

    use crate::ui::{
        browser2::NodeView,
        browser_state::{self, DirectoryNode, DirectoryNodeEvent, NodeEvent, NodeType, TreeNode},
        BrowserEvent,
    };

    pub struct File {}

    impl File {}

    pub struct Directory {}

    impl Directory {
        pub fn new<L>(cx: &mut Context, node: L)
        //todo rename to view? have new build something else here? this is weird.
        where
            L: Lens<Target = DirectoryNode>,
            L::Source: Model,
        {
            Binding::new(cx, node.clone(), move |cx, node| {
                let label = format!("{}", node.get(cx).label);
                info!("Trigger Directory Binding");

                let is_open = node.clone().then(DirectoryNode::is_open);

                let node1 = node.clone();

                HStack::new(cx, |cx| {
                    Label::new(cx, "\u{e75c}")
                        .font("icon")
                        .height(Stretch(1.0))
                        .child_top(Stretch(1.0))
                        .child_bottom(Stretch(1.0))
                        .hoverable(false)
                        .rotate(is_open.map(|b| if *b { 0.0 } else { -90.0 }));

                    // File or directory name
                    Label::new(cx, &label).width(Stretch(1.0)).text_wrap(false).hoverable(false);
                })
                .cursor(CursorIcon::Hand)
                .on_press(move |cx| {
                    let item = node.clone().get(cx);
                    cx.focus();
                    //cx.emit(NodeEvent::SetSelected);
                    cx.emit(DirectoryNodeEvent::ToggleOpen(item));
                })
                .col_between(Pixels(4.0))
                .child_left(Pixels(15.0 * 0 as f32 + 5.0));

                let item = node1.clone();

                if item.get(cx).is_open {
                    List::new(cx, item.then(DirectoryNode::children), |cx, index, node| {
                        match node.get(cx).node_type {
                            NodeType::File(file) => {
                                Label::new(cx, "FILE - ()");
                            }
                            NodeType::Directory(directory) => {
                                //let node = node.then(TreeNode::node_type).then(NodeType::directory);
                                //Directory::new(cx, node);
                                // Label::new(
                                //     cx,
                                //     &format!("DIRECTORY - ({})", directory.path.to_str().unwrap()),
                                // );
                                NodeView::new().view(cx, node.clone());
                            }
                            NodeType::None => {
                                Label::new(cx, "NONE");
                            }
                        };
                        //              NodeView::new().view(cx, node);
                    });
                }
            });
        }
    }
}

struct Header {}
impl Default for Header {
    fn default() -> Self {
        Self {}
    }
}

impl Header {
    fn new() -> Self {
        Self::default()
    }

    fn view(self, cx: &mut Context) {
        // Browser label
        Label::new(cx, "BROWSER").text_wrap(false).class("small");
    }
}

struct SideBar {}

impl Default for SideBar {
    fn default() -> Self {
        Self {}
    }
}

impl SideBar {
    fn new() -> Self {
        Self::default()
    }

    fn view(&self, cx: &mut Context) {
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
    }
}

struct FileViewMenu {}
impl Default for FileViewMenu {
    fn default() -> Self {
        Self {}
    }
}
impl FileViewMenu {
    fn new() -> Self {
        Self::default()
    }

    fn view(self, cx: &mut Context) {
        // Button to set a new root_node (Target Folder)
        Label::new(cx, "SET ROOT").on_release(|cx| {
            if let Some(folder_path) = rfd::FileDialog::new().pick_folder() {
                cx.emit(BrowserEvent::SetRoot(folder_path.clone()));
            }
        });

        // todo - split this off
        HStack::new(cx, |cx| {
            Label::new(cx, "Filter :"); //.text_wrap(false).class("small");
            Textbox::new(
                cx,
                UiData::state.then(UiState::browser.then(BrowserState::search_expression)),
            )
            .on_submit(|cx, text, _| {
                cx.emit(BrowserEvent::SetSearchExpression(text));
            })
            .height(Auto);
        });
    }
}

struct NodeView {}

impl NodeView {
    fn new() -> Self {
        Self {}
    }

    fn view<L>(&self, cx: &mut Context, node: L)
    where
        L: Lens<Target = TreeNode>,
        L::Source: Model,
    {
        match node.get(cx).node_type {
            NodeType::File(file) => {
                Label::new(cx, "FILE");
            }

            NodeType::Directory(_) => {
                let node = node.then(TreeNode::node_type).then(NodeType::directory);
                //Label::new(cx, "DIRECTORY");
                browser_widgets::Directory::new(cx, node);

                //
            }
            NodeType::None => {
                Label::new(cx, "NONE");
            }
        };
    }
}

struct FileView {
    root_node: BrowserTree,
}

impl Default for FileView {
    fn default() -> Self {
        Self { root_node: BrowserTree::empty() }
    }
}

impl FileView {
    fn new() -> Self {
        Self::default()
    }

    fn view<L>(&self, cx: &mut Context, root_node: L)
    where
        L: Lens<Target = BrowserTree>,
        L::Source: Model,
    {
        Panel::new(
            cx,
            |cx| {
                Header::new().view(cx);
            },
            |cx| {
                // menu to adjust the file view
                // todo - should be elsewhere, file menu belongs in it's own little house
                FileViewMenu::new().view(cx);

                ScrollView::new(cx, 0.0, 0.0, false, false, move |cx| {
                    Binding::new(cx, root_node, move |cx, node| {
                        info!("FileView building..");
                        let children = node.then(BrowserTree::children);
                        VStack::new(cx, |cx| {
                            List::new(cx, children, move |cx, index, node| {
                                info!("ROOT:({}) IDX:({})", node.clone().get(cx).label, index);
                                let node = node;
                                NodeView::new().view(cx, node);
                            })
                            .height(Auto);
                        })
                        .height(Auto);
                    });
                });
            },
        )
        .class("level3");
    }
}

struct FileViewPanel {}

impl Default for FileViewPanel {
    fn default() -> Self {
        Self {}
    }
}

impl FileViewPanel {
    fn new() -> Self {
        Self::default()
    }

    fn view(self, cx: &mut Context) {
        ResizableStack::new(
            cx,
            UiData::state.then(UiState::panels.then(PanelState::browser_width)),
            |cx, width| {
                cx.emit(PanelEvent::SetBrowserWidth(width));
            },
            |cx| {
                // The actual browser panel

                let browser_tree =
                    UiData::state.then(UiState::browser.then(BrowserState::browser_tree));
                FileView::new().view(cx, browser_tree);
            },
        )
        .class("browser")
        .toggle_class("hidden", UiData::state.then(UiState::panels.then(PanelState::hide_browser)));
    }
}

pub struct Browser {}

impl Default for Browser {
    fn default() -> Self {
        Self {}
    }
}

impl Browser {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn view(self, cx: &mut Context) {
        HStack::new(cx, |cx| {
            SideBar::new().view(cx);
            FileViewPanel::new().view(cx);
        })
        .width(Auto)
        .class("level1");
    }
}
