use eframe::egui;

pub mod about;
pub mod debug;
pub mod exit;
pub mod setting;

pub trait WindowExt {
  fn open_at_last_close_pos(&self,) -> bool;
  fn is_show(&self,) -> bool;
  fn set_show(
    &mut self,
    show: bool,
  );
  fn change_show(
    &mut self,
    show: bool,
  ) -> bool {
    let old = self.is_show();
    self.set_show(show,);
    old ^ show
  }

  fn create_window<'a,>(
    &mut self,
    ctx: &'a egui::Context,
    title: impl Into<egui::WidgetText,>,
    show: &'a mut bool,
  ) -> egui::Window<'a,> {
    let changed = self.change_show(*show,);

    let screen_center_pos = egui::Align2::CENTER_CENTER.pos_in_rect(&ctx.screen_rect(),);
    let default_pos = screen_center_pos;

    let win = egui::Window::new(title,)
      .open(show,)
      .collapsible(false,)
      .resizable(false,)
      .movable(true,)
      .default_pos(default_pos,)
      .pivot(egui::Align2::CENTER_CENTER,);

    if changed && !self.open_at_last_close_pos() {
      win.current_pos(default_pos,)
    } else {
      win
    }
  }
}

#[derive(Debug, Clone, Copy,)]
pub enum WindowId {
  Exit,
  About,
  Setting,
  Debug,
}
