use eframe::{egui, epaint::vec2};

fn main() -> Result<(), eframe::Error,> {
  let options = eframe::NativeOptions {
    initial_window_size: Some(vec2(300., 200.,),),
    ..Default::default()
  };

  eframe::run_native(
    "scroll_to_me",
    options,
    Box::new(|_cc| Box::<MyApp,>::new(MyApp {},),),
  )
}

struct MyApp {}

impl eframe::App for MyApp {
  fn update(
    &mut self,
    ctx: &egui::Context,
    _frame: &mut eframe::Frame,
  ) {
    egui::CentralPanel::default().show(ctx, |ui| {
      // TOP
      egui::TopBottomPanel::top("top",).show_inside(ui, |ui| {
        egui::ScrollArea::horizontal().show(ui, |ui| {
          let response = ui.button("123",);
          if response.clicked() {
            response.scroll_to_me(Some(egui::Align::Center,),);
          }
        },);
      },);

      //
      egui::ScrollArea::vertical().show(ui, |ui| {
        for i in 0..20 {
          ui.monospace(format!("label {i}"),);
        }
      },)
      //
    },);
  }
}
