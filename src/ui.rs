use eframe::egui;

use crate::style;

pub fn central_panel(ctx: &egui::Context,) -> egui::CentralPanel {
  egui::CentralPanel::default().frame(adjust_panel_frame(egui::Frame::central_panel(
    ctx.style().as_ref(),
  ),),)
}

pub fn top_panel(
  id: impl Into<eframe::egui::Id,>,
  ctx: &egui::Context,
) -> egui::TopBottomPanel {
  egui::TopBottomPanel::top(id,).frame(adjust_panel_frame(egui::Frame::side_top_panel(
    ctx.style().as_ref(),
  ),),)
}

pub fn bottom_panel(
  id: impl Into<eframe::egui::Id,>,
  ctx: &egui::Context,
) -> egui::TopBottomPanel {
  egui::TopBottomPanel::bottom(id,).frame(adjust_panel_frame(egui::Frame::side_top_panel(
    ctx.style().as_ref(),
  ),),)
}

pub fn left_panel(
  id: impl Into<eframe::egui::Id,>,
  ctx: &egui::Context,
) -> egui::SidePanel {
  egui::SidePanel::left(id,).frame(adjust_panel_frame(egui::Frame::side_top_panel(
    ctx.style().as_ref(),
  ),),)
}

#[allow(unused)]
pub fn right_panel(
  id: impl Into<eframe::egui::Id,>,
  ctx: &egui::Context,
) -> egui::SidePanel {
  egui::SidePanel::right(id,).frame(adjust_panel_frame(egui::Frame::side_top_panel(
    ctx.style().as_ref(),
  ),),)
}

pub fn custom_collapsing<HR, BR,>(
  ui: &mut egui::Ui,
  id: impl Into<egui::Id,>,
  title: impl Into<egui::WidgetText,>,
  open: bool,
  header_fn: impl FnOnce(&mut egui::Ui,) -> HR,
  body_fn: impl FnOnce(&mut egui::Ui,) -> BR,
) -> (
  egui::collapsing_header::CollapsingState,
  egui::InnerResponse<HR,>,
  Option<egui::InnerResponse<egui::scroll_area::ScrollAreaOutput<BR,>,>,>,
) {
  let mut state =
    egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id.into(), open,);

  // header
  let mut header_res = ui.horizontal(|ui| {
    // 使用默认 icon
    state.show_toggle_button(
      ui,
      |ui: &mut egui::Ui, openness: f32, response: &egui::Response| {
        egui::collapsing_header::paint_default_icon(ui, openness, response,)
      },
    );
    // 标题
    ui.label(title,);
    // 右侧自定义组件
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center,), header_fn,)
      .inner
  },);

  // 点击 header 触发折叠/展开
  header_res.response = header_res.response.interact(egui::Sense::click(),);
  if header_res.response.clicked() {
    state.toggle(ui,);
  }

  // body
  let body_res = state.show_body_indented(&header_res.response, ui, |ui| {
    egui::ScrollArea::both()
      .auto_shrink([false, false,],)
      .show(ui, body_fn,)
  },);

  (state, header_res, body_res,)
}

///////////////////////////////////////////////
// 透明度
///////////////////////////////////////////////
static TRANSPARENCY: once_cell::sync::Lazy<egui::mutex::Mutex<f32,>,> =
  once_cell::sync::Lazy::new(|| egui::mutex::Mutex::new(style::DEFAULT_TRANSPARENCY,),);

/// APP 窗口透明度
pub fn eframe_transparency(visuals: &egui::Visuals,) -> eframe::epaint::Color32 {
  visuals.window_fill.gamma_multiply(*TRANSPARENCY.lock(),)
}

/// 统一处理 pane frame 样式
pub fn adjust_panel_frame(frame: egui::Frame,) -> egui::Frame {
  frame
    .multiply_with_opacity(*TRANSPARENCY.lock(),)
    .inner_margin(egui::Margin::same(0.,),)
    .outer_margin(egui::Margin::same(0.,),)
}

/// 透明度调整组件
pub fn transparency_slider(ui: &mut egui::Ui,) {
  ui.horizontal(|ui| {
    let mut v = TRANSPARENCY.lock();
    let transparency_slider = egui::Slider::new(&mut *v, 0.0..=1.0,);
    ui.add(transparency_slider,);

    if ui.button("重置",).clicked() {
      *v = style::DEFAULT_TRANSPARENCY;
    };
  },);
}

///////////////////////////////////////////////
// 白天/黑夜 模式
///////////////////////////////////////////////
static DARK_MODE: once_cell::sync::Lazy<std::sync::atomic::AtomicBool,> =
  once_cell::sync::Lazy::new(|| std::sync::atomic::AtomicBool::new(true,),);

pub fn sync_mode(dark_mode: bool,) -> bool {
  let current = DARK_MODE.load(std::sync::atomic::Ordering::Relaxed,);
  if current != dark_mode {
    DARK_MODE.store(dark_mode, std::sync::atomic::Ordering::Relaxed,);
    return true;
  }
  false
}

pub fn dark_mode() -> bool {
  DARK_MODE.load(std::sync::atomic::Ordering::Relaxed,)
}

pub fn tint_color(highlight: bool,) -> egui::Color32 {
  let dark_mode = dark_mode();
  if highlight {
    if dark_mode {
      egui::Color32::from_additive_luminance(u8::MAX,)
    } else {
      egui::Color32::from_black_alpha(u8::MAX,)
    }
  } else {
    //
    if dark_mode {
      egui::Color32::from_additive_luminance(100_u8,)
    } else {
      egui::Color32::from_black_alpha(100_u8,)
    }
  }
}

///////////////////////////////////////////////
// zoom
///////////////////////////////////////////////
pub fn zoom_clamp(new_zoom: f32,) -> f32 {
  let mut pixels_per_point = new_zoom;
  pixels_per_point = pixels_per_point.clamp(0.2, 4.,);
  pixels_per_point = (pixels_per_point * 10.).round() / 10.;
  pixels_per_point
}

// ///////////////////////////////////////////////
// // 全屏
// ///////////////////////////////////////////////

// pub fn toggle_fullscreen(frame: &mut eframe::Frame,) {
//   frame.set_fullscreen(!frame.info().window_info.fullscreen,);
// }

// pub fn enter_fullscreen(frame: &mut eframe::Frame,) {
//   frame.set_fullscreen(true,);
// }

// pub fn exit_fullscreen(frame: &mut eframe::Frame,) {
//   frame.set_fullscreen(false,);
// }

// ///////////////////////////////////////////////
// // decorations
// ///////////////////////////////////////////////

// static DECORATIONS: once_cell::sync::Lazy<AtomicBool,> =
//   once_cell::sync::Lazy::new(|| AtomicBool::new(true,),);

// pub fn toggle_decorations(frame: &mut eframe::Frame,) {
//   if let Ok(v,) = DECORATIONS.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |v| Some(!v,),) {
//     frame.set_decorations(!v,);
//   }
// }
