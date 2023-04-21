use eframe::egui;

use crate::{action, text};

#[derive(Default,)]
pub struct ExitWindow {
  allowed_to_close: bool,
  pub show: bool,
}

impl ExitWindow {
  pub fn show(
    &mut self,
    ctx: &egui::Context,
    frame: &mut eframe::Frame,
  ) {
    if !self.show {
      return;
    }
    let title = text::window_title(&super::WindowId::Exit,);
    egui::Window::new(title,)
      .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::splat(0.,),)
      .collapsible(false,)
      .resizable(false,)
      .show(ctx, |ui| {
        ui.add_space(10.,);
        ui.monospace("确定退出当前应用程序？",);
        // ui.shrink_width_to_current();
        ui.add_space(10.,);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP,), |ui| {
          {
            let ok_button = egui::Button::new("[Y] 确定",);
            if ui.add(ok_button,).clicked() {
              self.allowed_to_close = true;
              frame.close();
            }
            ctx.input_mut(|input| {
              if input.consume_shortcut(&action::parse_shortcut("Y",).unwrap(),) {
                self.allowed_to_close = true;
                frame.close();
              }
            },);
          }
          {
            let cancel_button = egui::Button::new("[N] 取消",);
            if ui.add(cancel_button,).clicked() {
              self.show = false;
            }
            ctx.input_mut(|input| {
              if input.consume_shortcut(&action::parse_shortcut("N",).unwrap(),) {
                self.show = false;
              }
            },);
          }
        },);
      },);
  }

  pub fn on_frame_close_event(&mut self,) -> bool {
    let allowed_to_close = self.allowed_to_close;
    self.show = !allowed_to_close;
    allowed_to_close
  }
}
