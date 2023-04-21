#![feature(fn_traits)]
use eframe::{
  egui::{self, Response},
  epaint::vec2,
};

fn main() -> Result<(), eframe::Error,> {
  let options = eframe::NativeOptions {
    initial_window_size: Some(vec2(300., 200.,),),
    ..Default::default()
  };

  eframe::run_native(
    "tree",
    options,
    Box::new(|_cc| Box::<MyApp,>::new(MyApp {},),),
  )
}

struct MyApp {}

impl eframe::App for MyApp {
  fn update(
    &mut self,
    ctx: &egui::Context,
    _frame: &mut eframe::Frame,
  ) {
    egui::CentralPanel::default().show(ctx, |ui| {
      //
      let mut menu_bar = MenuBar::new();
      menu_bar.show(ui, render_menu,);
      //
    },);
  }
}

fn render_menu(
  ui: &mut egui::Ui,
  menu: &mut Tree<Menu,>,
) -> Response {
  match &menu.value {
    Menu::SubMenu(v,) => {
      ui.menu_button(v, |ui| {
        for m in menu.kids.iter_mut() {
          m.ui(ui, render_menu,);
        }
      },)
        .response
    }
    Menu::Item(v,) => ui.button(v,),
  }
}

pub struct Tree<T,> {
  value: T,
  kids: Vec<Tree<T,>,>,
}

impl<T,> Tree<T,> {
  pub fn new_leaf(value: T,) -> Self {
    Self {
      value,
      kids: vec![],
    }
  }

  pub fn new_branch(
    value: T,
    kids: Vec<Tree<T,>,>,
  ) -> Self {
    Self {
      value,
      kids,
    }
  }
}

impl<T,> Tree<T,> {
  pub fn ui<F,>(
    &mut self,
    ui: &mut egui::Ui,
    mut f: F,
  ) where
    F: FnMut(&mut egui::Ui, &mut Tree<T,>,) -> egui::Response,
  {
    let _ = f.call_mut((ui, self,),);
  }
}

pub enum Menu {
  SubMenu(String,),
  Item(String,),
}

struct MenuBar {
  title: String,
  menu: Vec<Tree<Menu,>,>,
}

impl MenuBar {
  fn new() -> Self {
    let menus = vec![
      Tree::new_branch(
        Menu::SubMenu("menu1".into(),),
        vec![Tree::new_branch(
          Menu::SubMenu("submenu1".into(),),
          vec![Tree::new_branch(Menu::Item("item1".into(),), vec![],)],
        )],
      ),
      Tree::new_branch(Menu::Item("item2".into(),), vec![],),
    ];
    Self {
      title: "MenuBar".into(),
      menu: menus,
    }
  }

  fn show<F,>(
    &mut self,
    ui: &mut egui::Ui,
    mut f: F,
  ) where
    F: FnMut(&mut egui::Ui, &mut Tree<Menu,>,) -> egui::Response,
  {
    ui.heading(self.title.clone(),);
    for m in self.menu.iter_mut() {
      m.ui(ui, &mut f,);
    }
  }
}
