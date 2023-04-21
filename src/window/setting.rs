use eframe::egui;

use crate::{text, ui};

use super::WindowExt;

#[derive(Default,)]
pub struct SettingWindow {
  pub show: bool,
  pub open_at_last_close_pos: bool,
}

impl WindowExt for SettingWindow {
  fn open_at_last_close_pos(&self,) -> bool {
    self.open_at_last_close_pos
  }

  fn is_show(&self,) -> bool {
    self.show
  }

  fn set_show(
    &mut self,
    show: bool,
  ) {
    self.show = show;
  }
}

impl SettingWindow {
  pub fn show(
    &mut self,
    ctx: &egui::Context,
    show: &mut bool,
  ) {
    let title = text::window_title(&super::WindowId::Setting,);
    self.create_window(ctx, title, show,).show(ctx, |ui| {
      let spacing_size = 30.;
      egui::Grid::new("my_grid",)
        .num_columns(2,)
        .spacing([spacing_size, spacing_size,],)
        .striped(true,)
        .show(ui, |ui| {
          ui.monospace("主题：",);
          egui::widgets::global_dark_light_mode_buttons(ui,);
          ui.end_row();

          ui.monospace("透明度：",);
          ui::transparency_slider(ui,);
          ui.end_row();
        },);
    },);
  }
}
