use std::path::{Path, PathBuf};

use super::UiEvent;
use vizia::prelude::*;

#[derive(Debug, Lens, Clone, Data)]
pub struct BrowserState {
    pub root_node: BrowserNode,
    pub selected: Option<PathBuf>,
    pub search_expression: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BrowserEvent {
    ViewAll,
    SetRoot(PathBuf),
    SetSelected(PathBuf),
    SelectNext,
    SelectPrev,
    ToggleOpen,
    PlaySelected,
    StopSelected,
    SetSearchExpression(String),
}

mod browser_node_tree {
    use vizia::prelude::Lens;

    #[derive(Debug)]
    enum NodeType {
        Root,
        File,
        Directory,
    }

    #[derive(Debug, Lens)]
    struct Tree {
        label: String,
        children: Vec<NodeType>,
    }

    impl Default for Tree {
        fn default() -> Self {
            Self { label: String::from("Default"), children: vec![] }
        }
    }

    impl Tree {
        fn new() -> Self {
            Self::default()
        }
    }

    struct RootNode {}
    struct FileNode {}
    struct DirectoryNode {}
}

#[derive(Debug, Clone, Data, Lens)]
pub struct BrowserNode {
    pub name: String,
    pub file_path: Option<PathBuf>,
    pub children: Vec<BrowserNode>,
    pub is_open: bool,
    pub is_visible: bool,
}

impl Default for BrowserNode {
    fn default() -> Self {
        Self {
            name: String::new(),
            file_path: None,
            children: Vec::new(),
            is_open: true,
            is_visible: true,
        }
    }
}

impl Default for BrowserState {
    fn default() -> Self {
        Self {
            root_node: BrowserNode {
                name: String::from("root"),
                file_path: Some(PathBuf::from("assets/test_files")),
                children: vec![],
                is_open: true,
                is_visible: true,
            },
            selected: Some(PathBuf::from("assets/test_files")),
            search_expression: String::from("..."),
        }
    }
}

impl Model for BrowserState {
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|browser_event, _| match browser_event {
            BrowserEvent::SetSearchExpression(search_expression) => {
                self.search_expression = search_expression.clone();
                self.root_node = filter_root_node(search_expression, &mut self.root_node);
            }

            // Temp: Load the assets directory for the treeview // todo remove
            BrowserEvent::ViewAll => {
                if let Some(root) = build_node_tree(&Path::new("assets/test_files")) {
                    self.root_node = root;
                }
            }

            // Set the new root from where the browser build the file view
            BrowserEvent::SetRoot(path) => {
                if let Some(root_node) = build_node_tree(path.as_path()) {
                    self.root_node = root_node;
                }
            }

            // Play the selected file
            BrowserEvent::PlaySelected => {
                if let Some(path) = &self.selected {
                    if path.is_file() {
                        cx.emit(UiEvent::BrowserFileClicked(path.clone()));
                    }
                }
            }

            BrowserEvent::StopSelected => {
                cx.emit(UiEvent::BrowserFileStop());
            }

            // toggle open/closed a folder
            BrowserEvent::ToggleOpen => {
                //println!("Toggle Open: {:?}", path);
                if let Some(path) = &self.selected {
                    toggle_open(&mut self.root_node, path);
                }
            }

            // Set the selected directory item by path
            BrowserEvent::SetSelected(path) => {
                self.selected = Some(path.clone());
            }

            // Move selection the next directory item
            BrowserEvent::SelectNext => {
                let next = recursive_next(&self.root_node, None, self.selected.clone());
                match next {
                    RetItem::Found(path) => self.selected = path,
                    _ => {}
                }
            }

            // Move selection the previous directory item
            BrowserEvent::SelectPrev => {
                let next = recursive_prev(&self.root_node, None, self.selected.clone());
                match next {
                    RetItem::Found(path) => self.selected = path,
                    _ => {}
                }
            }
        });
    }
}

fn filter_root_node(search_expression: &str, root_node: &mut BrowserNode) -> BrowserNode {
    for node in &mut root_node.children {
        if node.name.contains(search_expression) {
            node.is_visible = true;
        } else {
            node.is_visible = false;
        }
    }

    root_node.to_owned()
}

#[derive(Debug, Clone)]
enum RetItem<'a> {
    Found(Option<PathBuf>),
    NotFound(Option<&'a BrowserNode>),
}

fn toggle_open(root: &mut BrowserNode, path: &PathBuf) {
    if root.file_path == Some(path.clone()) {
        root.is_open ^= true;
    } else {
        for child in root.children.iter_mut() {
            toggle_open(child, path);
        }
    }
}

// Returns the next directory item after `dir` by recursing down the hierarchy
fn recursive_next<'a>(
    root: &'a BrowserNode,
    mut prev: Option<&'a BrowserNode>,
    dir: Option<PathBuf>,
) -> RetItem<'a> {
    if let Some(prev) = prev {
        if prev.file_path == dir {
            return RetItem::Found(root.file_path.clone());
        }
    }

    prev = Some(root);
    if root.is_open {
        for child in root.children.iter() {
            let next = recursive_next(child, prev, dir.clone());
            match next {
                RetItem::Found(_) => return next,
                RetItem::NotFound(file) => prev = file,
            }
        }
    }

    RetItem::NotFound(prev)
}

// Returns the previous directory item before `dir` by recursing down the hierarchy
fn recursive_prev<'a>(
    root: &'a BrowserNode,
    mut prev: Option<&'a BrowserNode>,
    dir: Option<PathBuf>,
) -> RetItem<'a> {
    if root.file_path == dir {
        if let Some(prev) = prev {
            return RetItem::Found(prev.file_path.clone());
        }
    }

    prev = Some(root);
    if root.is_open {
        for child in root.children.iter() {
            let next = recursive_prev(child, prev, dir.clone());
            match next {
                RetItem::Found(_) => return next,
                RetItem::NotFound(file) => prev = file,
            }
        }
    }

    RetItem::NotFound(prev)
}

// Recursively build directory tree from root path
fn build_node_tree(root_directory: &Path) -> Option<BrowserNode> {
    let name = format!("{}", root_directory.file_name()?.to_str()?);
    let mut children = Vec::new();

    if root_directory.is_dir() {
        for entry in std::fs::read_dir(root_directory).ok()? {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.is_dir() {
                children.push(build_node_tree(&path)?);
            } else {
                children.push(BrowserNode {
                    name: format!("{}", entry.path().file_name()?.to_str()?),
                    file_path: Some(entry.path()),
                    children: vec![],
                    is_open: true,
                    is_visible: true,
                })
            }
        }
    }

    // Sort by alphabetical
    children.sort_by(|a, b| a.name.cmp(&b.name));
    // Sort by directory vs file
    children.sort_by(|a, b| {
        let a_is_dir: bool = a.children.is_empty();
        let b_is_dir: bool = b.children.is_empty();
        a_is_dir.cmp(&b_is_dir)
    });

    Some(BrowserNode {
        name,
        file_path: Some(PathBuf::from(root_directory)),
        children,
        is_open: true,
        is_visible: true,
    })
}

// Return the path of a file directory
fn dir_path(path: &Path) -> Option<&Path> {
    if path.is_dir() {
        Some(path)
    } else {
        path.parent()
    }
}
