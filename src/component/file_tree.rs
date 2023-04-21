use std::{
  borrow::{BorrowMut, Cow},
  cell::RefCell,
  collections::BTreeSet,
  fs,
  path::{Path, PathBuf},
  rc::{Rc, Weak},
  time::Duration,
};

use eframe::egui;

use crate::{text, ui, util};

use super::open_file::OpenFiles;

#[derive(Debug,)]
pub enum Node {
  Dir {
    path: PathBuf,
    children: BTreeSet<Rc<RefCell<Node,>,>,>,
    parent: Option<Weak<RefCell<Node,>,>,>,
    expand: bool,
  },
  File {
    path: PathBuf,
    parent: Weak<RefCell<Node,>,>,
  },
}

impl core::hash::Hash for Node {
  fn hash<H: core::hash::Hasher,>(
    &self,
    state: &mut H,
  ) {
    self.order().hash(state,);
    self.path().hash(state,);
  }
}

impl PartialEq for Node {
  fn eq(
    &self,
    other: &Self,
  ) -> bool {
    self.order().eq(&other.order(),) && self.path().eq(other.path(),)
  }
}

impl Eq for Node {
}

impl PartialOrd for Node {
  fn partial_cmp(
    &self,
    other: &Self,
  ) -> Option<std::cmp::Ordering,> {
    Some(self.cmp(other,),)
  }
}

impl Ord for Node {
  fn cmp(
    &self,
    other: &Self,
  ) -> std::cmp::Ordering {
    let cmp = self.order().cmp(&other.order(),);
    let cmp2 = || self.path().cmp(other.path(),);
    cmp.then_with(cmp2,)
  }
}

impl Node {
  pub fn order(&self,) -> usize {
    match self {
      Node::Dir {
        ..
      } => 1,
      Node::File {
        ..
      } => 2,
    }
  }
  pub fn is_expand(&self,) -> bool {
    match self {
      Node::Dir {
        path: _,
        children: _,
        parent: _,
        expand,
      } => expand.to_owned(),
      Node::File {
        path: _,
        parent: _,
      } => false,
    }
  }
  pub fn expand(
    &mut self,
    to_expand: bool,
  ) {
    match self {
      Node::Dir {
        path: _,
        children: _,
        parent: _,
        expand,
      } => *expand = to_expand,
      Node::File {
        path: _,
        parent: _,
      } => {}
    }
  }
  pub fn is_dir(&self,) -> bool {
    match self {
      Node::Dir {
        path: _,
        children: _,
        expand: _,
        parent: _,
      } => true,
      Node::File {
        path: _,
        parent: _,
      } => false,
    }
  }

  pub fn name(&self,) -> Cow<str,> {
    match self {
      Node::Dir {
        path,
        children: _,
        expand: _,
        parent: _,
      } => path.file_name().unwrap().to_string_lossy(),
      Node::File {
        path,
        parent: _,
      } => path.file_name().unwrap().to_string_lossy(),
    }
  }

  pub fn path(&self,) -> &PathBuf {
    match self {
      Node::Dir {
        path,
        children: _,
        parent: _,
        expand: _,
      } => path,
      Node::File {
        path,
        parent: _,
      } => path,
    }
  }

  pub fn set_parent(
    &mut self,
    node: Weak<RefCell<Node,>,>,
  ) {
    match self {
      Node::Dir {
        path: _,
        children: _,
        parent,
        expand: _,
      } => {
        let _ = parent.insert(node,);
      }
      Node::File {
        path: _,
        parent: _,
      } => {}
    }
  }

  pub fn add_child(
    &mut self,
    node: Rc<RefCell<Node,>,>,
  ) -> bool {
    match self {
      Node::Dir {
        path: _,
        children,
        expand: _,
        parent: _,
      } => {
        children.borrow_mut().insert(node,);
        true
      }
      Node::File {
        path: _,
        parent: _dir,
      } => false,
    }
  }

  pub fn children(&self,) -> Option<&BTreeSet<Rc<RefCell<Node,>,>,>,> {
    match self {
      Node::Dir {
        path: _,
        children,
        parent: _,
        expand: _,
      } => Some(children,),
      Node::File {
        path: _,
        parent: _,
      } => None,
    }
  }

  pub fn set_children(
    &mut self,
    nodes: BTreeSet<Rc<RefCell<Node,>,>,>,
  ) {
    match self {
      Node::Dir {
        path: _,
        children,
        expand: _,
        parent: _,
      } => {
        *children = nodes;
      }
      Node::File {
        path: _,
        parent: _,
      } => {}
    }
  }

  pub fn need_load_children(&self,) -> bool {
    match self {
      Node::Dir {
        path: _,
        children,
        parent: _,
        expand: _,
      } => children.is_empty(),
      Node::File {
        path: _,
        parent: _,
      } => false,
    }
  }
}

pub fn load_children(parent: &Rc<RefCell<Node,>,>,) {
  let parent_ref = parent.borrow();
  if !parent_ref.is_dir() || !parent_ref.is_expand() || !parent_ref.need_load_children() {
    return;
  }
  let dir_path = parent_ref.path().to_owned();
  drop(parent_ref,);

  if let Ok(rd,) = fs::read_dir(dir_path,) {
    let mut children: BTreeSet<Rc<RefCell<Node,>,>,> = BTreeSet::new();
    rd.into_iter()
      .filter_map(|v| v.ok(),)
      .filter_map(|v| match v.file_type() {
        Ok(ft,) => {
          if ft.is_dir() {
            let node = Node::Dir {
              path: v.path(),
              children: BTreeSet::new(),
              parent: Some(Rc::downgrade(&parent.clone(),),),
              expand: false,
            };
            Some(node,)
          } else if ft.is_file() {
            let node = Node::File {
              path: v.path(),
              parent: Rc::downgrade(&parent.clone(),),
            };
            Some(node,)
          } else {
            None
          }
        }
        Err(_,) => None,
      },)
      .for_each(|v| {
        children.insert(Rc::new(RefCell::new(v,),),);
      },);

    RefCell::borrow_mut(parent,).set_children(children,);
  }
}

// ------------------------------------ UI
pub fn show_tree(
  node: &Rc<RefCell<Node,>,>,
  root: &Rc<RefCell<Node,>,>,
  ui: &mut egui::Ui,
  open_files: &mut OpenFiles,
  mut prefix: Vec<Rc<RefCell<Node,>,>,>,
) {
  load_children(node,);

  let is_root = root == node;
  let root_path = {
    let root_ref = root.borrow();
    root_ref.path().to_owned()
  };

  let node_ref = RefCell::borrow(node,);
  let name = node_ref.name();
  let path = node_ref.path().to_owned();
  let is_dir = node_ref.is_dir();
  let expand = node_ref.is_expand();

  // Dir
  if is_dir {
    // UI
    let response = if is_root {
      let (state, header_res, body_res,) = ui::custom_collapsing(
        ui,
        "file_tree",
        name,
        expand,
        |_ui| {
          // TODO
        },
        |ui| {
          node_ref.children().unwrap().iter().for_each(|v| {
            show_tree(v, root, ui, open_files, vec![],);
          },);
        },
      );
      egui::CollapsingResponse {
        header_response: header_res.response,
        body_response: body_res.map(|v| v.response,),
        body_returned: None,
        openness: state.openness(ui.ctx(),),
      }
    }
    // not root dir
    else {
      let kids = node_ref.children().unwrap();
      if kids.len() == 1 {
        let v = kids.iter().next().unwrap();
        if v.borrow().is_dir() {
          prefix.push(node.clone(),);
          show_tree(v, root, ui, open_files, prefix,);
          return;
        }
      }
      let title: String = if prefix.is_empty() {
        name.into()
      } else {
        let mut s = String::new();
        for n in prefix.iter() {
          let n_ref = RefCell::borrow(n,);
          let v = n_ref.name();
          if !s.is_empty() {
            s.push('/',);
          }
          s.push_str(&v,);
        }
        s
      };
      egui::CollapsingHeader::new(title,)
        .default_open(expand,)
        .show(ui, |ui| {
          kids.iter().for_each(|v| {
            show_tree(v, root, ui, open_files, vec![],);
          },);
        },)
    };

    drop(node_ref,);

    // 展开/折叠 action
    RefCell::borrow_mut(node,).expand(response.fully_open(),);

    let mut header_response = response.header_response;
    // 鼠标悬停提示
    header_response = header_response.on_hover_text(path.to_string_lossy(),);
    // 右键菜单
    header_response.context_menu(|ui| {
      context_menu_ui(ui, &path, &root_path,);
    },);
  }
  // File
  else {
    // UI
    let selected = open_files.is_current_file(&path,);
    let mut response = ui.selectable_label(selected, name,);

    // 鼠标悬停提示
    response = response.on_hover_text(path.to_string_lossy(),);
    // 鼠标点击
    if response.clicked() {
      if let Err(e,) = open_files.open_file(&path,) {
        util::toaster()
          .error(format!("无法读取文件：{path:?}\nErr: {e}"),)
          .set_duration(Some(Duration::from_secs(5,),),);
      }
    }
    // 右键菜单
    response.context_menu(|ui| {
      context_menu_ui(ui, &path, &root_path,);
    },);
  }
}

#[derive(Debug, Clone, Copy,)]
pub enum ContextMenu {
  Separator,
  Item(ContextMenuAction,),
}

#[derive(Debug, Clone, Copy,)]
pub enum ContextMenuAction {
  CopyFullPath,
  CopyRelativePath,
  OpenInNative,
}

fn context_menu_ui(
  ui: &mut egui::Ui,
  path: &Path,
  root_path: &PathBuf,
) {
  ui.style_mut().wrap = Some(false,);
  let menus = vec![
    ContextMenu::Item(ContextMenuAction::OpenInNative,),
    ContextMenu::Separator,
    ContextMenu::Item(ContextMenuAction::CopyRelativePath,),
    ContextMenu::Item(ContextMenuAction::CopyFullPath,),
  ];
  let handle_context_menu = |action, ui: &mut egui::Ui| match action {
    ContextMenuAction::CopyFullPath => {
      util::set_clipboard(ui.ctx(), path.to_string_lossy(),);
    }
    ContextMenuAction::CopyRelativePath => {
      util::set_clipboard(
        ui.ctx(),
        path.strip_prefix(root_path,).unwrap().to_string_lossy(),
      );
    }
    ContextMenuAction::OpenInNative => {
      util::open_in_native(path,);
    }
  };
  for m in menus.iter() {
    match m {
      ContextMenu::Separator => {
        ui.separator();
      }
      &ContextMenu::Item(action,) => {
        let text = text::file_tree_context_menu_text(&action,);
        let btn = egui::Button::new(text,);
        if ui.add(btn,).clicked() {
          ui.close_menu();
          handle_context_menu(action, ui,);
        }
      }
    }
  }
}

#[cfg(test)]
mod test {
  use std::{cell::RefCell, path::PathBuf, rc::Rc};

  use crate::component::file_tree::{load_children, Node};

  #[test]
  fn test_load_children() {
    let node = Node::Dir {
      path: std::env::current_dir().unwrap(),
      children: Default::default(),
      parent: Default::default(),
      expand: true,
    };

    let node = Rc::new(RefCell::new(node,),);
    load_children(&node,);

    let node = RefCell::borrow_mut(&node,);
    println!("children count => {}", node.children().unwrap().len());
    println!(
      "children => {:#?}",
      node
        .children()
        .unwrap()
        .iter()
        .map(|v| RefCell::borrow(v).path().clone())
        .collect::<Vec<PathBuf,>>()
    );
    assert!(!node.children().unwrap().is_empty());
  }
}
