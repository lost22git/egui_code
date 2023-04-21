use std::sync::mpsc::SyncSender;

use crate::{
  action::{self, Action},
  frame_history, id, style, ui,
};

use super::open_file::OpenFile;

use eframe::egui;

#[derive(Debug, Clone, Copy,)]
pub enum ItemId {
  FilePath,
  FileEncoding,
  FileLineEnding,
  CursorStat,
  Fps,
}

#[derive(Debug, Clone, Copy,)]
pub enum Region {
  Left,
  Center,
  Right,
}

impl ItemId {
  pub fn show(
    &self,
    ui: &mut egui::Ui,
    region: Region,
    file: Option<&OpenFile,>,
  ) {
    let rich_text = match self {
      ItemId::FilePath => {
        if let Some(f,) = file {
          if f.changed() {
            Some(egui::RichText::new(format!(
              "{} [+]",
              f.path().to_string_lossy()
            ),),)
          } else {
            Some(egui::RichText::new(f.path().to_string_lossy(),),)
          }
        } else {
          None
        }
      }
      ItemId::FileEncoding => file.map(|f| egui::RichText::new(f.encoding().name(),),),
      ItemId::FileLineEnding => file.map(|f| egui::RichText::new(f.line_ending().as_str(),),),
      ItemId::CursorStat => {
        if let Some(f,) = file {
          if let Some((row, col, selected,),) = f.cursor_stat() {
            Some(egui::RichText::new(format!(
              "Row {row}, Col {col} ({selected} Selected)"
            ),),)
          } else {
            None
          }
        } else {
          None
        }
      }
      ItemId::Fps => {
        let rich_text = egui::RichText::new(format!(
          "FPS: {:.1} CPU: {:.2} ms/frame",
          frame_history::fps(),
          1e3 * frame_history::mean_frame_time()
        ),);
        Some(rich_text,)
      }
    };

    if rich_text.is_none() {
      return;
    }

    let mut rich_text = rich_text.unwrap();

    rich_text = match region {
      Region::Left => rich_text.strong(),
      Region::Center => rich_text,
      Region::Right => rich_text.strong(),
    };

    ui.monospace(rich_text,);
  }
}

pub struct StatusBar {
  tx: SyncSender<Action,>,
  left: Vec<ItemId,>,
  center: Vec<ItemId,>,
  right: Vec<ItemId,>,
}

impl StatusBar {
  pub fn push_left(
    mut self,
    item: ItemId,
  ) -> Self {
    self.left.push(item,);
    self
  }
  pub fn push_center(
    mut self,
    item: ItemId,
  ) -> Self {
    self.center.push(item,);
    self
  }
  pub fn push_right(
    mut self,
    item: ItemId,
  ) -> Self {
    self.right.push(item,);
    self
  }
}

impl StatusBar {
  pub fn new(tx: SyncSender<Action,>,) -> Self {
    let slf = Self {
      tx,
      left: vec![],
      right: vec![],
      center: vec![],
    };
    slf
      .push_left(ItemId::Fps,)
      .push_right(ItemId::FileLineEnding,)
      .push_right(ItemId::FileEncoding,)
      .push_right(ItemId::CursorStat,)
      .push_center(ItemId::FilePath,)
  }
}

// ------------------------------------ UI

impl StatusBar {
  pub fn show(
    &self,
    ui: &mut egui::Ui,
    show: bool,
    file: Option<&OpenFile,>,
  ) {
    ui.style_mut().wrap = Some(false,);
    ui::bottom_panel(id::STATUS_BAR, ui.ctx(),)
      .exact_height(style::STATUS_BAR_HEIGHT,)
      .resizable(false,)
      .show_animated_inside(ui, show, |ui| {
        let spacing_size = ui.spacing().item_spacing * 2.;
        ui.spacing_mut().item_spacing = spacing_size;

        ui.horizontal_centered(|ui| {
          // 左半部分
          ui.horizontal_centered(|ui| {
            // 左侧填充
            ui.add_space(spacing_size.x,);

            for item in self.left.iter() {
              item.show(ui, Region::Left, file,);
            }
          },);

          // 右半部分
          ui::right_panel("status_bar_right", ui.ctx(),)
            .show_separator_line(false,)
            .show_inside(ui, |ui| {
              ui.with_layout(egui::Layout::right_to_left(egui::Align::Center,), |ui| {
                // 右侧填充
                ui.add_space(spacing_size.x,);
                for item in self.right.iter() {
                  item.show(ui, Region::Right, file,);
                }
              },);
            },);

          // 中间部分
          egui::CentralPanel::default().show_inside(ui, |ui| {
            egui::ScrollArea::horizontal()
              .stick_to_right(true,)
              .show(ui, |ui| {
                ui.horizontal_centered(|ui| {
                  for item in self.center.iter() {
                    item.show(ui, Region::Center, file,);
                  }
                },);
              },);
          },);
          //
        },);
      },);
  }
}

impl action::Handle for StatusBar {
  fn handle(
    &mut self,
    _action: &Action,
  ) {
    //
  }
}
