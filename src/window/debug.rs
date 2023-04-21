use eframe::{egui, epaint};

use crate::text;

use super::WindowExt;

#[derive(Default,)]
pub struct DebugWindow {
  show: bool,
  pub open_at_last_close_pos: bool,
  pub debug_options: egui::style::DebugOptions,
}

impl WindowExt for DebugWindow {
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

impl DebugWindow {
  pub fn configure_style(
    &self,
    style: &mut egui::Style,
  ) {
    style.debug = self.debug_options;
  }

  pub fn show(
    &mut self,
    ctx: &egui::Context,
    show: &mut bool,
  ) {
    let title = text::window_title(&super::WindowId::Debug,);
    self.create_window(ctx, title, show,).show(ctx, |ui| {
      // tessellation_options
      let mut t_opts = ctx.tessellation_options(|opts| *opts,);
      ui.group(|ui| {
        ui.checkbox(&mut t_opts.debug_paint_text_rects, "paint text rects",);
        ui.checkbox(&mut t_opts.debug_paint_clip_rects, "paint clip rects",);
        ui.checkbox(&mut t_opts.debug_ignore_clip_rects, "ignore clip rects",);
        ui.checkbox(
          &mut t_opts.coarse_tessellation_culling,
          "coarse tessellation culling",
        );
        ui.checkbox(&mut t_opts.feathering, "feathering",);
        ui.checkbox(&mut t_opts.prerasterized_discs, "prerasterized discs",);
        ui.checkbox(&mut t_opts.round_text_to_pixels, "round text to pixels",);
        ui.add(
          egui::Slider::new(&mut t_opts.bezier_tolerance, 0.0..=1.,).text("bezier tolerance",),
        );
        ui.add(
          egui::Slider::new(&mut t_opts.feathering_size_in_pixels, 0.0..=10.,)
            .text("feathering size in pixels",),
        );
        ui.vertical_centered(|ui| {
          if ui.button("Reset",).clicked() {
            t_opts = epaint::TessellationOptions::default();
          }
        },)
      },);
      ctx.tessellation_options_mut(|opts| *opts = t_opts,);
      // debug options
      ui.group(|ui| {
        self.debug_options.ui(ui,);
      },)
    },);
  }
}
