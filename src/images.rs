use crate::ui;

pub fn logo() -> &'static [u8] {
  include_bytes!("../res/icons/logo.png")
}

pub fn explorer() -> &'static [u8] {
  match ui::dark_mode() {
    true => include_bytes!("../res/icons/dark/files.svg"),
    false => include_bytes!("../res/icons/light/files.svg"),
  }
}

pub fn search() -> &'static [u8] {
  match ui::dark_mode() {
    true => include_bytes!("../res/icons/dark/search.svg"),
    false => include_bytes!("../res/icons/light/search.svg"),
  }
}

pub fn extension() -> &'static [u8] {
  match ui::dark_mode() {
    true => include_bytes!("../res/icons/dark/extensions.svg"),
    false => include_bytes!("../res/icons/light/extensions.svg"),
  }
}

pub fn setting() -> &'static [u8] {
  match ui::dark_mode() {
    true => include_bytes!("../res/icons/dark/settings-gear.svg"),
    false => include_bytes!("../res/icons/light/settings-gear.svg"),
  }
}

//
use eframe::egui;
use egui_extras::{image::FitTo, RetainedImage};

pub struct CachedImage<T: AsRef<[u8],>,> {
  name: String,
  load_raw_bytes: Box<dyn Fn() -> T,>,
  image: Option<RetainedImage,>,
}

impl<T: AsRef<[u8],>,> CachedImage<T,> {
  pub fn new<F,>(
    name: impl Into<String,>,
    load_raw_bytes: F,
  ) -> Self
  where
    F: Fn() -> T + 'static,
  {
    Self {
      name: name.into(),
      load_raw_bytes: Box::new(load_raw_bytes,),
      image: None,
    }
  }

  fn load_new_svg_image(
    &self,
    size: FitTo,
  ) -> RetainedImage {
    let debug_name = &self.name;
    let svg_bytes = self.load_raw_bytes.call((),);
    RetainedImage::from_svg_bytes_with_size(debug_name, svg_bytes.as_ref(), size,).unwrap()
  }

  fn load_new_image(&self,) -> RetainedImage {
    let debug_name = &self.name;
    let image_bytes = self.load_raw_bytes.call((),);
    RetainedImage::from_image_bytes(debug_name, image_bytes.as_ref(),).unwrap()
  }

  //---------------------------------------------

  pub fn reset(&mut self,) {
    self.image = None;
  }

  pub fn as_image_button(
    &mut self,
    ui: &mut egui::Ui,
    size: FitTo,
  ) -> egui::ImageButton {
    if self.image.is_none() {
      let image = self.load_new_svg_image(size,);
      self.image = Some(image,);
    }
    let image = self.image.as_ref().unwrap();

    egui::ImageButton::new(image.texture_id(ui.ctx(),), image.size_vec2(),).frame(false,)
  }

  pub fn as_image_widget(
    &mut self,
    ui: &egui::Ui,
    size: impl Into<egui::Vec2,>,
  ) -> egui::Image {
    if self.image.is_none() {
      let image = self.load_new_image();
      self.image = Some(image,);
    }
    let image = self.image.as_ref().unwrap();

    egui::Image::new(image.texture_id(ui.ctx(),), size,)
  }
}
