use std::sync::mpsc::SyncSender;

use crate::{
  action::{self, Action},
  id,
  images::{self, CachedImage},
  style, text, ui,
};

#[derive(Debug, Clone, PartialEq, Eq,)]
pub enum ToolId {
  Explorer,
  Search,
  Extension,
  Setting,
  #[allow(unused)]
  Custom(String,),
}

impl From<&ToolId,> for String {
  fn from(value: &ToolId,) -> Self {
    format!("{:?}", value,)
  }
}

pub struct Item {
  pub id: ToolId,
  pub action: Action,
  pub image: CachedImage<&'static [u8],>,
}

// --------------------------------------------------------

pub struct ToolBar {
  tx: SyncSender<Action,>,
  top: Vec<Item,>,
  bottom: Vec<Item,>,
  current_index: usize,
  hover_index: usize,
}

impl ToolBar {
  pub fn push_top(
    mut self,
    item: Item,
  ) -> Self {
    self.top.push(item,);
    self
  }

  pub fn push_bottom(
    mut self,
    item: Item,
  ) -> Self {
    self.bottom.push(item,);
    self
  }

  pub fn reset(&mut self,) {
    for item in self.top.iter_mut() {
      item.image.reset();
    }

    for item in self.bottom.iter_mut() {
      item.image.reset();
    }
  }

  pub fn current_item(&self,) -> Option<&Item,> {
    self.top.get(self.current_index,)
  }

  fn toggle_item(
    &mut self,
    item_id: ToolId,
  ) {
    let cindex = self.current_index;
    for (i, item,) in self.top.iter().enumerate() {
      if item.id == item_id {
        if cindex == i {
          self.current_index = usize::MAX;
        } else {
          self.current_index = i;
        }
        break;
      }
    }
  }

  fn select_item(
    &mut self,
    item_id: ToolId,
  ) {
    for (i, item,) in self.top.iter().enumerate() {
      if item.id == item_id {
        self.current_index = i;
        break;
      }
    }
  }
}

impl ToolBar {
  pub fn new(tx: SyncSender<Action,>,) -> Self {
    let explorer_image = CachedImage::new(&ToolId::Explorer, images::explorer,);
    let search_image = CachedImage::new(&ToolId::Search, images::search,);
    let extensions_image = CachedImage::new(&ToolId::Extension, images::extension,);
    let settings_image = CachedImage::new(&ToolId::Setting, images::setting,);

    let slf = Self {
      tx,
      top: vec![],
      bottom: vec![],
      current_index: usize::MAX,
      hover_index: usize::MAX,
    };

    slf
      .push_top(Item {
        id: ToolId::Explorer,
        action: Action::ToggleExplorer,
        image: explorer_image,
      },)
      .push_top(Item {
        id: ToolId::Search,
        action: Action::NoOp,
        image: search_image,
      },)
      .push_top(Item {
        id: ToolId::Extension,
        action: Action::NoOp,
        image: extensions_image,
      },)
      .push_bottom(Item {
        id: ToolId::Setting,
        action: Action::OpenSettingWindow,
        image: settings_image,
      },)
  }
}

// ------------------------------------ UI

use eframe::egui;

impl ToolBar {
  pub fn show(
    &mut self,
    ui: &mut egui::Ui,
    show: bool,
  ) {
    ui::left_panel(id::TOOL_BAR, ui.ctx(),)
      .resizable(false,)
      .exact_width(style::TOOL_BAR_WIDTH,)
      .show_animated_inside(ui, show, |ui| {
        // item大小
        let size =
          egui_extras::image::FitTo::Size(style::TOOL_BUTTON_SIZE, style::TOOL_BUTTON_SIZE,);
        // item间距
        ui.style_mut().spacing.item_spacing.y = style::TOOL_BUTTON_SPACING;
        let spacing_size = ui.spacing().item_spacing;

        // 上半部分
        ui.vertical_centered(|ui| {
          // 顶部填充
          ui.add_space(spacing_size.y,);
          for (i, item,) in self.top.iter_mut().enumerate() {
            let widget = item.image.as_image_button(ui, size,).tint(ui::tint_color(
              self.hover_index == i || self.current_index == i,
            ),);
            let hover_text = text::tool_hover_text(&item.id,);
            let resp = ui.add(widget,).on_hover_text_at_pointer(hover_text,);
            if resp.hovered() {
              self.hover_index = i;
            } else if self.hover_index == i {
              self.hover_index = usize::MAX;
            }
            if resp.clicked() {
              self.current_index = if self.current_index == i {
                usize::MAX
              } else {
                i
              };
            }
            // if resp.clicked() {
            //   let _ = self.tx.send(item.action.clone(),);
            // }
          }
        },);
        // 下半部分
        ui.with_layout(egui::Layout::bottom_up(egui::Align::Center,), |ui| {
          // 底部填充
          ui.add_space(spacing_size.y,);
          for (_i, item,) in self.bottom.iter_mut().enumerate() {
            let widget = item
              .image
              .as_image_button(ui, size,)
              .tint(ui::tint_color(false,),);

            let hover_text = text::tool_hover_text(&item.id,);
            let resp = ui.add(widget,).on_hover_text_at_pointer(hover_text,);
            if resp.clicked() {
              let _ = self.tx.send(item.action.clone(),);
            }
          }
        },);
      },);
  }
}

#[allow(clippy::single_match)]
impl action::Handle for ToolBar {
  fn handle(
    &mut self,
    action: &Action,
  ) {
    match action {
      Action::ToggleExplorer => self.toggle_item(ToolId::Explorer,),
      _ => {}
    }
  }
}
