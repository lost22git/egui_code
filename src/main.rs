#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![feature(fn_traits)]
#![feature(io_error_other)]

mod action;
mod app;
mod component;
mod dev_tool;
mod font;
mod frame_history;
mod hl;
mod id;
#[allow(unused)]
mod images;
mod style;
mod text;
mod ui;
mod util;
mod window;
use app::MyApp;
use eframe::{egui, IconData};

fn main() -> Result<(), eframe::Error,> {
  dev_tool::start_puffin_server();

  tracing_subscriber::fmt().init();

  let app_icon = load_app_icon();
  let options = eframe::NativeOptions {
    default_theme: style::DEFAULT_THEME,
    drag_and_drop_support: true,
    transparent: true,
    initial_window_pos: Some(egui::pos2(style::MAIN_X, style::MAIN_Y,),),
    initial_window_size: Some(egui::vec2(style::MAIN_WIDTH, style::MAIN_HEIGHT,),),
    icon_data: Some(app_icon,),
    ..Default::default()
  };

  eframe::run_native(
    util::app_name(),
    options,
    Box::new(|cc| Box::<MyApp,>::new(MyApp::new(cc,),),),
  )
}

fn load_app_icon() -> IconData {
  let logo_image = image::load_from_memory(images::logo(),).unwrap();
  let (width, height,) = logo_image.as_rgba8().unwrap().dimensions();
  tracing::info!("Load app icon => {width}x{height}");
  IconData {
    rgba: logo_image.into_bytes(),
    width,
    height,
  }
}
