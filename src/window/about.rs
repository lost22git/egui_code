use eframe::egui;

use crate::{
  images::{self, CachedImage},
  text, util,
};

use super::{WindowExt, WindowId};

pub struct AboutWindow {
  pub show: bool,
  pub open_at_last_close_pos: bool,
  pub logo_image: CachedImage<&'static [u8],>,
}

impl WindowExt for AboutWindow {
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

impl AboutWindow {
  pub fn new() -> Self {
    let logo_image = CachedImage::new("logo", images::logo,);
    Self {
      show: false,
      open_at_last_close_pos: false,
      logo_image,
    }
  }
}

impl AboutWindow {
  pub fn show(
    &mut self,
    ctx: &egui::Context,
    show: &mut bool,
  ) {
    let title = text::window_title(&WindowId::About,);
    self.create_window(ctx, title, show,).show(ctx, |ui| {
      ui.vertical_centered_justified(|ui| {
        let logo_size = 120.;
        let logo_image = self
          .logo_image
          .as_image_widget(ui, egui::Vec2::splat(logo_size,),);
        ui.add(logo_image,);

        ui.add_space(-20.,);

        ui.group(|ui| {
          ui.set_width(ui.available_width(),); //填充可用宽度
          egui::Grid::new("app_info",)
            .num_columns(2,)
            .spacing([40.0, 4.0,],)
            .show(ui, |ui| {
              ui.monospace("应用名",);
              ui.monospace(util::app_name(),);
              ui.end_row();

              ui.monospace("版本",);
              ui.monospace(util::app_version(),);
              ui.end_row();

              ui.monospace("贡献者",);
              ui.monospace(util::app_author().join("; ",),);
              ui.end_row();
            },);
        },);
      },);
    },);
  }
}
