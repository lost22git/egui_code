pub struct Tree<T,> {
  pub value: T,
  pub kids: Vec<Tree<T,>,>,
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

// ------------------------------------ UI

// use eframe::egui;

// impl<T,> Tree<T,> {
//   pub fn ui<F,>(
//     &mut self,
//     ui: &mut egui::Ui,
//     mut f: F,
//   ) where
//     F: FnMut(&mut egui::Ui, &mut Tree<T,>,) -> egui::Response,
//   {
//     let _ = f.call_mut((ui, self,),);
//   }
// }
