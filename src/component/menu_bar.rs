use std::sync::mpsc::SyncSender;

use crate::{
  action::{self, Action, KeyActions},
  text,
};

use super::tree::Tree;

#[derive(Debug, Clone, Copy,)]
pub enum MenuId {
  File,
  Edit,
  View,
  Appearance,
  About,
}

#[allow(clippy::enum_variant_names)]
pub enum Menu {
  SubMenu(MenuId,),
  Item(Action,),
  Separator,
}

pub struct MenuBar {
  tx: SyncSender<Action,>,
  menus: Vec<Tree<Menu,>,>,
}

impl MenuBar {
  pub fn new(tx: SyncSender<Action,>,) -> Self {
    let file = Tree::new_branch(
      Menu::SubMenu(MenuId::File,),
      vec![
        Tree::new_leaf(Menu::Item(Action::OpenFolder,),),
        Tree::new_leaf(Menu::Separator,),
        Tree::new_leaf(Menu::Item(Action::ExitApp,),),
      ],
    );
    let edit = Tree::new_branch(
      Menu::SubMenu(MenuId::Edit,),
      vec![
      //   Tree::new_leaf(Menu::Item(Action::OpenFolder,),),
      //   Tree::new_leaf(Menu::Item(Action::ExitApp,),),
      ],
    );

    let view = Tree::new_branch(
      Menu::SubMenu(MenuId::View,),
      vec![Tree::new_branch(
        Menu::SubMenu(MenuId::Appearance,),
        vec![
          Tree::new_leaf(Menu::Item(Action::ToggleFullScreen,),),
          Tree::new_leaf(Menu::Item(Action::ToggleStatusBar,),),
          Tree::new_leaf(Menu::Item(Action::ToggleToolBar,),),
          Tree::new_leaf(Menu::Item(Action::ToggleTerminal,),),
          Tree::new_leaf(Menu::Separator,),
          Tree::new_leaf(Menu::Item(Action::ZoomIn,),),
          Tree::new_leaf(Menu::Item(Action::ZoomOut,),),
          Tree::new_leaf(Menu::Item(Action::ZoomReset,),),
        ],
      )],
    );

    let about = Tree::new_branch(
      Menu::SubMenu(MenuId::About,),
      vec![
        Tree::new_leaf(Menu::Item(Action::OpenDebugWindow,),),
        Tree::new_leaf(Menu::Item(Action::OpenPuffinViewer,),),
        Tree::new_leaf(Menu::Separator,),
        Tree::new_leaf(Menu::Item(Action::OpenAboutWindow,),),
      ],
    );

    Self {
      tx,
      menus: vec![file, edit, view, about],
    }
  }
}

// ------------------------------------ UI

use eframe::egui;

impl MenuBar {
  pub fn show(
    &mut self,
    ui: &mut egui::Ui,
    key_actions: &KeyActions,
  ) {
    egui::menu::bar(ui, |ui| {
      let menus = &self.menus;
      for m in menus.iter() {
        self.show_menu(ui, m, key_actions,);
      }
    },);
  }

  fn show_menu(
    &self,
    ui: &mut egui::Ui,
    menu: &Tree<Menu,>,
    key_actions: &KeyActions,
  ) {
    ui.style_mut().wrap = Some(false,);
    match &menu.value {
      Menu::SubMenu(v,) => {
        // 子菜单
        let text = text::menu_text(v,);
        ui.menu_button(text, |ui| {
          for m in menu.kids.iter() {
            self.show_menu(ui, m, key_actions,);
          }
        },);
      }
      // 菜单项
      Menu::Item(v,) => {
        let keys = key_actions.get_action_keys(v,);
        let text = text::menu_item_text(v, &keys,);
        if ui.button(text,).clicked() {
          ui.close_menu();
          let _ = self.tx.send(v.clone(),);
        }
      }
      // 分割符
      Menu::Separator => {
        ui.separator();
      }
    }
  }
}

impl action::Handle for MenuBar {
  fn handle(
    &mut self,
    _action: &Action,
  ) {
    //
  }
}
