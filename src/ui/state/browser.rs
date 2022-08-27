use std::path::{Path, PathBuf};

use self::browser_state::{BrowserTree, DirectoryNode, NodeType};

use super::UiEvent;
use log::info;
use vizia::prelude::*;

#[derive(Debug, Clone, Lens)]
pub struct BrowserState {
    pub tree: BrowserTree,
    pub selected: NodeType,
    pub search_expression: String,
}

#[derive(Debug)]
pub struct SomeError {} //todo filler error type, need to define these properly
impl From<std::io::Error> for SomeError {
    fn from(_: std::io::Error) -> Self {
        SomeError {}
    }
}

#[derive(Debug, Clone)]
pub enum BrowserEvent {
    SetRoot(PathBuf),
    SetSelected(NodeType),
    SelectNext,
    SelectPrev,
    ToggleOpen,
    PlaySelected,
    StopSelected,
    SetSearchExpression(String),
}

pub mod browser_state {
    //todo this is a mess, needs more logical structure
    use log::info;
    use std::{fs::File, io::Error, path::PathBuf};
    use vizia::{
        prelude::{Data, Event, EventContext, Lens},
        state::Model,
        tree::Tree,
    };

    use super::SomeError;

    pub enum NodeEvent {
        SetSelected,
    }

    #[derive(Debug, Clone, Lens, Data)]
    pub struct TreeNode {
        pub label: String,
        pub node_type: NodeType,
        pub is_selected: bool,
    }
    impl Model for TreeNode {
        fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
            event.map(|node_event, _| match node_event {
                NodeEvent::SetSelected => {
                    info!("Set Selected");
                    self.is_selected = true;
                }
            });
        }
    }

    #[derive(Debug, Clone, Lens, Data)]
    pub enum NodeType {
        File(FileNode),
        Directory(DirectoryNode),
        None,
    }

    #[derive(Debug, Clone, Lens)]
    pub struct BrowserTree {
        pub label: String,
        pub children: Vec<TreeNode>,
    }

    impl Default for BrowserTree {
        fn default() -> Self {
            Self { label: String::from("Default"), children: vec![] }
        }
    }

    impl BrowserTree {
        pub fn empty() -> Self {
            Self::default()
        }

        pub fn update(&mut self, path_buffer: &PathBuf) -> Result<(), SomeError> {
            info!("Updating BrowserTree with target at - {}", path_buffer.to_str().unwrap());

            if path_buffer.is_dir() {
                info!("Target is valid Directory");

                self.label = String::from(path_buffer.file_name().unwrap().to_str().unwrap()); // todo, better error handling here

                let directory_node =
                    DirectoryNode::new(String::from(&self.label), path_buffer.to_owned());

                info!(
                    "Adding new Tree Root. Label({}) Path({})",
                    self.label,
                    path_buffer.to_str().unwrap()
                );

                let node =
                    TreeNode::new(String::from(&self.label), NodeType::Directory(directory_node));

                self.children.push(node);

                return Ok(());
            }
            Err(SomeError {})
        }
    }

    #[derive(Debug, Clone, Lens, Data)]
    pub struct FileNode {}

    pub enum DirectoryNodeEvent {
        ToggleOpen,
    }

    #[derive(Debug, Clone, Lens, Data)]
    pub struct DirectoryNode {
        pub label: String,
        pub path: PathBuf,
        pub children: Vec<NodeType>,
        pub is_open: bool,
        pub set_selected: bool,
    }

    impl Model for DirectoryNode {
        fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
            event.map(|directory_node_event, _| match directory_node_event {
                DirectoryNodeEvent::ToggleOpen => {
                    info!("Toggle Open");
                    self.is_open = !self.is_open;
                }
            });
        }
    }

    impl FileNode {
        pub fn new() -> Self {
            Self {}
        }
    }
    impl DirectoryNode {
        pub fn new(label: String, path: PathBuf) -> Self {
            Self { label, path, children: vec![], is_open: false, set_selected: false }
        }

        pub fn recursive_scan(mut self) -> Result<NodeType, SomeError> {
            if self.path.is_dir() {
                //     info!("Directory found: {}", self.path.to_str().unwrap());

                for child in std::fs::read_dir(self.path)? {
                    let entry = child?;
                    let path = entry.path();

                    if path.is_dir() {
                        info!("Directory found: {}", path.to_str().unwrap());
                        let directory_node = DirectoryNode::new(
                            String::from(path.file_name().unwrap().to_str().unwrap()), //todo better error handlilng
                            path,
                        );

                        self.children.push(directory_node.recursive_scan()?);
                    } else {
                        info!("File found: {}", path.to_str().unwrap());
                        self.children.push(NodeType::File(FileNode::new()));
                    }
                }

                // }
                // Err(SomeError {}) // todo error handling, scanning something which is not a directory or file.
            }
            Ok(NodeType::None)
        }

        pub fn scan(mut self) -> Result<NodeType, SomeError> {
            if self.path.is_dir() {
                //     info!("Directory found: {}", self.path.to_str().unwrap());

                for child in std::fs::read_dir(self.path)? {
                    let entry = child?;
                    let path = entry.path();

                    if path.is_dir() {
                        info!("Directory found: {}", path.to_str().unwrap());
                        let directory_node = DirectoryNode::new(
                            String::from(path.file_name().unwrap().to_str().unwrap()), //todo better error handlilng
                            path,
                        );

                        self.children.push(NodeType::Directory(directory_node));
                    } else {
                        info!("File found: {}", path.to_str().unwrap());
                        self.children.push(NodeType::File(FileNode::new()));
                    }
                }

                // }
                // Err(SomeError {}) // todo error handling, scanning something which is not a directory or file.
            }
            Ok(NodeType::None)
        }
    }
    impl TreeNode {
        pub(crate) fn new(label: String, node_type: NodeType) -> Self {
            Self { label, node_type, is_selected: false }
        }
    }
}

impl Default for BrowserState {
    fn default() -> Self {
        Self {
            tree: BrowserTree::empty(),
            selected: NodeType::None,
            search_expression: String::from("..."),
        }
    }
}

impl Model for BrowserState {
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|browser_event, _| match browser_event {
            BrowserEvent::SetSearchExpression(search_expression) => {
                self.search_expression = search_expression.clone();
                //self.tree = filter_root_node(search_expression, &mut self.tree); //todo implement filter code
            }

            // Set the new root from where the browser build the file view
            BrowserEvent::SetRoot(path) => {
                self.tree.update(path).expect("Failed to update Root"); //todo better error handling here
            }

            // Play the selected file
            BrowserEvent::PlaySelected => {
                todo!()
                // if let Some(path) = &self.selected {
                //     if path.is_file() {
                //         cx.emit(UiEvent::BrowserFileClicked(path.clone()));
                //     }
                // }
            }

            BrowserEvent::StopSelected => {
                cx.emit(UiEvent::BrowserFileStop());
            }

            // toggle open/closed a folder
            BrowserEvent::ToggleOpen => {
                //todo!()
                //println!("Toggle Open: {:?}", path);

                if let NodeType::Directory(node) = &mut self.selected {
                    node.is_open = !node.is_open;
                }
            }

            // Set the selected directory item by path
            BrowserEvent::SetSelected(node) => {
                info!("Toggle node ()");
                self.selected = node.clone();
                //self.selected = Some(path.clone());
            }

            // Move selection the next directory item
            BrowserEvent::SelectNext => {
                todo!()
                // let next = recursive_next(&self.tree, None, self.selected.clone());
                // match next {
                //     RetItem::Found(path) => self.selected = path,
                //     _ => {}
                // }
            }

            // Move selection the previous directory item
            BrowserEvent::SelectPrev => {
                todo!()
                // let next = recursive_prev(&self.tree, None, self.selected.clone());
                // match next {
                //     RetItem::Found(path) => self.selected = path,
                //     _ => {}
                // }
            }
        });
    }
}

// fn filter_root_node(search_expression: &str, root_node: &mut BrowserNode) -> BrowserNode {
//     for node in &mut root_node.children {
//         if node.name.contains(search_expression) {
//             node.is_visible = true;
//         } else {
//             node.is_visible = false;
//         }
//     }

//     root_node.to_owned()
// }

// #[derive(Debug, Clone)]
// enum RetItem<'a> {
//     Found(Option<PathBuf>),
//     NotFound(Option<&'a BrowserNode>),
// }

// fn toggle_open(root: &mut BrowserNode, path: &PathBuf) {
//     if root.file_path == Some(path.clone()) {
//         root.is_open ^= true;
//     } else {
//         for child in root.children.iter_mut() {
//             toggle_open(child, path);
//         }
//     }
// }

// // Returns the next directory item after `dir` by recursing down the hierarchy
// fn recursive_next<'a>(
//     root: &'a BrowserNode,
//     mut prev: Option<&'a BrowserNode>,
//     dir: Option<PathBuf>,
// ) -> RetItem<'a> {
//     if let Some(prev) = prev {
//         if prev.file_path == dir {
//             return RetItem::Found(root.file_path.clone());
//         }
//     }

//     prev = Some(root);
//     if root.is_open {
//         for child in root.children.iter() {
//             let next = recursive_next(child, prev, dir.clone());
//             match next {
//                 RetItem::Found(_) => return next,
//                 RetItem::NotFound(file) => prev = file,
//             }
//         }
//     }

//     RetItem::NotFound(prev)
// }

// // Returns the previous directory item before `dir` by recursing down the hierarchy
// fn recursive_prev<'a>(
//     root: &'a BrowserNode,
//     mut prev: Option<&'a BrowserNode>,
//     dir: Option<PathBuf>,
// ) -> RetItem<'a> {
//     if root.file_path == dir {
//         if let Some(prev) = prev {
//             return RetItem::Found(prev.file_path.clone());
//         }
//     }

//     prev = Some(root);
//     if root.is_open {
//         for child in root.children.iter() {
//             let next = recursive_prev(child, prev, dir.clone());
//             match next {
//                 RetItem::Found(_) => return next,
//                 RetItem::NotFound(file) => prev = file,
//             }
//         }
//     }

//     RetItem::NotFound(prev)
// }

// // Recursively build directory tree from root path
// fn build_node_tree(root_directory: &Path) -> Option<BrowserNode> {
//     let name = format!("{}", root_directory.file_name()?.to_str()?);
//     let mut children = Vec::new();

//     if root_directory.is_dir() {
//         for entry in std::fs::read_dir(root_directory).ok()? {
//             let entry = entry.ok()?;
//             let path = entry.path();
//             if path.is_dir() {
//                 children.push(build_node_tree(&path)?);
//             } else {
//                 children.push(BrowserNode {
//                     name: format!("{}", entry.path().file_name()?.to_str()?),
//                     file_path: Some(entry.path()),
//                     children: vec![],
//                     is_open: true,
//                     is_visible: true,
//                 })
//             }
//         }
//     }

//     // Sort by alphabetical
//     children.sort_by(|a, b| a.name.cmp(&b.name));
//     // Sort by directory vs file
//     children.sort_by(|a, b| {
//         let a_is_dir: bool = a.children.is_empty();
//         let b_is_dir: bool = b.children.is_empty();
//         a_is_dir.cmp(&b_is_dir)
//     });

//     Some(BrowserNode {
//         name,
//         file_path: Some(PathBuf::from(root_directory)),
//         children,
//         is_open: true,
//         is_visible: true,
//     })
// }

// // Return the path of a file directory
// fn dir_path(path: &Path) -> Option<&Path> {
//     if path.is_dir() {
//         Some(path)
//     } else {
//         path.parent()
//     }
// }
