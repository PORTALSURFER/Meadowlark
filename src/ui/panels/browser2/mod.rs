use std::default;

use crate::ui::{browser_state::*, state};

use crate::ui::state::{BrowserEvent, BrowserState, PanelEvent, PanelState};
use crate::ui::{Panel, ResizableStack, UiData, UiState};
use log::info;
use vizia::prelude::*;

pub mod browser_widgets {
    // todo better structure please, ffs boyyyy

    use vizia::{prelude::Context, views::Label};

    use crate::ui::browser_state::{self, DirectoryNode, NodeType};

    pub struct File {}

    impl File {}

    pub struct Directory {}

    impl Directory {
        pub fn new(cx: &mut Context, node: DirectoryNode) {
            let label = format!("{}", node.label);
            Label::new(cx, &label);
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

    fn build(self, cx: &mut Context) {
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

    fn build(&self, cx: &mut Context) {
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

    fn build(self, cx: &mut Context) {
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
    fn new(node_type: NodeType) -> Self {
        Self {}
    }

    fn build(&self, cx: &mut Context) {
        Label::new(cx, "NODE ITEM");
    }
}

struct FileView {
    root_node: state::browser_state::BrowserTree, // todo long name broooooo
}

impl Default for FileView {
    fn default() -> Self {
        Self { root_node: BrowserTree::empty() } // todo long name broooooo
    }
}

impl FileView {
    fn new() -> Self {
        Self::default()
    }

    fn build<L>(&self, cx: &mut Context, root_node: L)
    where
        L: Lens<Target = BrowserTree>, // todo long name broooooo
        L::Source: Model,
    {
        Panel::new(
            cx,
            |cx| {
                Header::new().build(cx);
            },
            |cx| {
                // menu to adjust the file view
                // todo - should be elsewhere
                FileViewMenu::new().build(cx);
                ScrollView::new(cx, 0.0, 0.0, false, false, move |cx| {
                    Binding::new(
                        cx,
                        root_node.clone().then(BrowserTree::children),
                        move |cx, children| {
                            VStack::new(cx, |cx| {
                                List::new(cx, children, move |cx, index, item| {
                                    let item = item.clone().get(cx);
                                    info!("list element {:?} {:?}", item, index);
                                    match item {
                                        NodeType::File(file) => {
                                            Label::new(cx, "FILE");
                                        }
                                        NodeType::Directory(directory) => {
                                            browser_widgets::Directory::new(cx, directory);
                                        }
                                        NodeType::None => {
                                            Label::new(cx, "NONE");
                                        }
                                    };
                                })
                                .height(Auto);
                            })
                            .height(Auto);
                        },
                    );
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

    fn build(self, cx: &mut Context) {
        ResizableStack::new(
            cx,
            UiData::state.then(UiState::panels.then(PanelState::browser_width)),
            |cx, width| {
                cx.emit(PanelEvent::SetBrowserWidth(width));
            },
            |cx| {
                // The actual browser panel
                FileView::new()
                    .build(cx, UiData::state.then(UiState::browser.then(BrowserState::tree)));
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

    pub fn build(self, cx: &mut Context) {
        HStack::new(cx, |cx| {
            SideBar::new().build(cx);
            FileViewPanel::new().build(cx);
        })
        .width(Auto)
        .class("level1");
    }
}
