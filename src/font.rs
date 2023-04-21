use eframe::egui;

/// 安装字体
pub fn install_font(ctx: &egui::Context,) {
  // Start with the default fonts (we will be adding to them rather than replacing them).
  let mut fonts = egui::FontDefinitions::default();

  let to_install_fonts = vec![
    ("微软雅黑", &include_bytes!("../res/fonts/微软雅黑.ttc")[..],),
    (
      "SpaceMono_NFM",
      &include_bytes!("../res/fonts/SpaceMono_NFM.ttf")[..],
    ),
  ];

  for (font_name, font_bytes,) in to_install_fonts.into_iter() {
    // Install my own font (maybe supporting non-latin characters).
    // .ttf and .otf files supported.
    fonts.font_data.insert(
      font_name.to_owned(),
      egui::FontData::from_static(font_bytes,),
    );

    fonts
      .families
      .entry(egui::FontFamily::Proportional,)
      .or_default()
      .insert(0, font_name.to_owned(),);

    fonts
      .families
      .entry(egui::FontFamily::Monospace,)
      .or_default()
      .insert(0, font_name.to_owned(),);
  }

  // Tell egui to use these fonts:
  ctx.set_fonts(fonts,);
}

/// Nerd Fonts
pub struct NerdFont<'a,>(char, &'a str,);
impl NerdFont<'static,> {
  pub fn class(&self,) -> &'static str {
    self.1
  }
  pub fn utf(&self,) -> char {
    self.0
  }
}
impl NerdFont<'static,> {
  // 
  pub const WARN: NerdFont<'static,> = NerdFont('\u{ea6c}', "nf-cod-warning",);
  // 
  pub const INFO: NerdFont<'static,> = NerdFont('\u{ea74}', "nf-cod-info",);
  // 
  pub const SETTINGS_GEAR: NerdFont<'static,> = NerdFont('\u{eb51}', "nf-cod-settings_gear",);
  // 
  pub const TELESCOPE: NerdFont<'static,> = NerdFont('\u{eb68}', "nf-cod-telescope",);
  // 
  pub const FOLDER: NerdFont<'static,> = NerdFont('\u{f07b}', "nf-fa-folder",);
  // 
  pub const FOLDER_OPEN: NerdFont<'static,> = NerdFont('\u{f07c}', "nf-fa-folder_open",);
  // 
  pub const RUST: NerdFont<'static,> = NerdFont('\u{e7a8}', "nf-dev-rust",);
}
