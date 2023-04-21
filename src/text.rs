use eframe::egui;

use crate::{
  action::{self, Action},
  component::{file_tree, menu_bar, open_file, tool_bar},
  font, window,
};

/// 配置 TextStyle
pub fn configure_style(style: &mut egui::Style,) {
  use eframe::egui::{
    FontFamily::{Monospace, Proportional},
    FontId, TextStyle,
  };

  style.text_styles = [
    (TextStyle::Heading, FontId::new(22.0, Proportional,),),
    (TextStyle::Body, FontId::new(16.0, Proportional,),),
    (TextStyle::Monospace, FontId::new(14.0, Monospace,),),
    (TextStyle::Button, FontId::new(14.0, Proportional,),),
    (TextStyle::Small, FontId::new(10.0, Proportional,),),
  ]
  .into();
}

pub const fn text_editor_font() -> egui::FontId {
  egui::FontId::new(16.0, egui::FontFamily::Monospace,)
}

pub fn window_title(id: &window::WindowId,) -> String {
  match id {
    window::WindowId::Exit => format!("{} {:?}", font::NerdFont::WARN.utf(), id),
    window::WindowId::About => format!("{} {:?}", font::NerdFont::INFO.utf(), id),
    window::WindowId::Setting => format!("{} {:?}", font::NerdFont::SETTINGS_GEAR.utf(), id),
    window::WindowId::Debug => format!("{} {:?}", font::NerdFont::TELESCOPE.utf(), id),
  }
}

pub fn menu_text(id: &menu_bar::MenuId,) -> String {
  format!("{:?}", id)
}

pub fn menu_item_text(
  act: &Action,
  keys: &Vec<&egui::KeyboardShortcut,>,
) -> String {
  if keys.is_empty() {
    act.name().into()
  } else {
    format!("{:<15}{:>15}", act.name(), action::format_key(keys[0]))
  }
}

pub fn tool_hover_text(id: &tool_bar::ToolId,) -> String {
  format!("{:?}", id)
}

pub fn open_files_context_menu_text(action: &open_file::ContextMenuAction,) -> String {
  match action {
    open_file::ContextMenuAction::Close => "Close".into(),
    open_file::ContextMenuAction::CloseAll => "CloseAll".into(),
    open_file::ContextMenuAction::CloseOthers => "CloseOthers".into(),
    open_file::ContextMenuAction::CloseToRight => "CloseToRight".into(),
    open_file::ContextMenuAction::CloseSaved => "CloseSaved".into(),
    open_file::ContextMenuAction::CopyFullPath => "CopyFullPath".into(),
    open_file::ContextMenuAction::CopyRelativePath => "CopyRelativePath".into(),
    open_file::ContextMenuAction::OpenInNative => "OpenInNative".into(),
  }
}

pub fn file_tree_context_menu_text(action: &file_tree::ContextMenuAction,) -> String {
  match action {
    file_tree::ContextMenuAction::CopyFullPath => "CopyFullPath".into(),
    file_tree::ContextMenuAction::CopyRelativePath => "CopyRelativePath".into(),
    file_tree::ContextMenuAction::OpenInNative => "OpenInNative".into(),
  }
}
