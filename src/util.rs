use std::path::Path;

use std::process::Command;

use eframe::egui;

use egui_notify::Toasts;

pub fn app_name() -> &'static str {
  env!("CARGO_PKG_NAME")
}
pub fn app_version() -> &'static str {
  env!("CARGO_PKG_VERSION")
}
pub fn app_author() -> Vec<String,> {
  let authors = env!("CARGO_PKG_AUTHORS");
  authors
    .split(';',)
    .map(|v| v.to_string(),)
    .collect::<Vec<String,>>()
}

///////////////////////////////////////////////
// egui_notify
///////////////////////////////////////////////

static TOASTS: once_cell::sync::Lazy<egui::mutex::Mutex<Toasts,>,> =
  once_cell::sync::Lazy::new(|| egui::mutex::Mutex::new(Toasts::default(),),);

pub fn toaster() -> egui::mutex::MutexGuard<'static, Toasts,> {
  TOASTS.lock()
}

///////////////////////////////////////////////
// 本地剪贴板
///////////////////////////////////////////////

pub fn set_clipboard(
  ctx: &egui::Context,
  s: impl Into<String,>,
) {
  ctx.output_mut(|output| output.copied_text = s.into(),);
}

///////////////////////////////////////////////
// encoding
///////////////////////////////////////////////

/// 注意：chardetng 无法正确检测 UTF-16 without BOM
pub fn guess_encoding(text: &[u8],) -> &'static encoding_rs::Encoding {
  let len = text.len();
  let to = std::cmp::min(1000, len,);
  let mut encoding_detector = chardetng::EncodingDetector::new();
  encoding_detector.feed(&text[0..to], to == len,);
  encoding_detector.guess(None, true,)
}

///////////////////////////////////////////////
// line-ending
///////////////////////////////////////////////
#[derive(Debug, Clone, Copy,)]
pub enum LineEnding {
  Unknown,
  Crlf,
  Lf,
}

impl LineEnding {
  pub const fn as_str(&self,) -> &'static str {
    match self {
      Self::Unknown => "UNKNOWN",
      Self::Crlf => "CRLF",
      Self::Lf => "LF",
    }
  }
}

pub fn guess_line_ending(text: &str,) -> LineEnding {
  let chars_to_test = egui::TextBuffer::char_range(&text, 0..1000,);
  if chars_to_test.contains("\r\n",) {
    LineEnding::Crlf
  } else if chars_to_test.contains('\n',) {
    LineEnding::Lf
  } else {
    LineEnding::Unknown
  }
}

///////////////////////////////////////////////
// 其他
///////////////////////////////////////////////
pub fn open_in_native(path: &Path,) {
  #[cfg(target_os = "windows")]
  let _ = Command::new("explorer.exe",)
    .arg(format!("/select,{}", path.to_string_lossy()),)
    .spawn();

  #[cfg(not(target_os = "windows"))]
  let _ = Command::new("open",)
    .arg("-R",)
    .arg(path.as_os_str(),)
    .spawn();
}

pub fn open_native_file_dialog() -> Option<std::path::PathBuf,> {
  rfd::FileDialog::new()
    .set_directory(std::env::current_dir().unwrap(),)
    .pick_folder()
}
