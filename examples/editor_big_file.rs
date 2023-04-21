use eframe::egui::{self, CentralPanel, TextEdit};

fn main() -> Result<(), eframe::Error,> {
  let options = eframe::NativeOptions {
    ..Default::default()
  };

  eframe::run_native(
    "editor big file test",
    options,
    Box::new(|_cc| Box::<MyApp,>::new(MyApp::new(),),),
  )
}

struct MyApp {
  text: String,
}

impl MyApp {
  fn new() -> Self {
    let bytes = include_bytes!("c:/Users/zzz/Desktop/wallpaper/4gha4c7w7gt91.png");
    let encoding = detect_encoding(bytes,);
    let (str, _, _,) = encoding.decode(bytes,);
    let string: String = str.into();
    MyApp {
      text: string,
    }
  }
}

pub fn detect_encoding(text: &[u8],) -> &'static encoding_rs::Encoding {
  let len = text.len();
  let to = std::cmp::min(1000, len,);
  let mut encoding_detector = chardetng::EncodingDetector::new();
  encoding_detector.feed(&text[0..to], to == len,);
  encoding_detector.guess(None, true,)
}

impl eframe::App for MyApp {
  fn update(
    &mut self,
    ctx: &egui::Context,
    _frame: &mut eframe::Frame,
  ) {
    CentralPanel::default().show(ctx, |ui| {
      let code_editor = TextEdit::multiline(&mut self.text,)
        .code_editor()
        .frame(true,)
        .lock_focus(true,)
        .desired_width(f32::INFINITY,)
        .desired_rows(40,);
      let _output = code_editor.show(ui,);
    },);
  }
}
