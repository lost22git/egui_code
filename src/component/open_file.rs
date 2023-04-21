use std::{path::PathBuf, sync::mpsc::SyncSender, time::Duration, vec};

use eframe::egui;
use encoding_rs::Encoding;

use crate::{
  action::{self, KeyActions},
  hl, id, style, text, ui,
  util::{self, LineEnding},
};

pub struct OpenFile {
  content: String,
  path: PathBuf,
  changed: bool,
  cursor_range: Option<egui::widgets::text_edit::CursorRange,>,
  encoding: &'static Encoding,
  line_ending: LineEnding,
}

impl OpenFile {
  fn new(path: &PathBuf,) -> Result<Self, std::io::Error,> {
    // 读取并解码文件
    let content_bytes = std::fs::read(path,)?;
    let encoding = util::guess_encoding(&content_bytes,);
    let content_str = encoding
      .decode_without_bom_handling_and_without_replacement(&content_bytes,)
      .ok_or(std::io::Error::other(format!(
        "读取文件失败：{} 解码失败",
        encoding.name()
      ),),)?;

    // guess line_ending
    let line_ending = util::guess_line_ending(&content_str,);

    let f = Self {
      content: content_str.into(),
      path: path.to_owned(),
      changed: false,
      cursor_range: None,
      encoding,
      line_ending,
    };
    Ok(f,)
  }

  pub fn id(&self,) -> egui::Id {
    egui::Id::new(format!("text_editor_{}", self.path.to_string_lossy()),)
  }

  pub fn path(&self,) -> &PathBuf {
    &self.path
  }

  pub fn extension(&self,) -> std::borrow::Cow<str,> {
    self.path.extension().unwrap().to_string_lossy()
  }

  pub fn encoding(&self,) -> &'static Encoding {
    self.encoding
  }

  pub fn line_ending(&self,) -> LineEnding {
    self.line_ending
  }

  pub fn changed(&self,) -> bool {
    self.changed
  }

  fn mark_changed(&mut self,) {
    self.changed = true;
  }

  pub fn save(&self,) -> std::io::Result<(),> {
    let (content_bytes, _, _,) = self.encoding.encode(&self.content,);
    std::fs::write(&self.path, content_bytes,)
  }

  pub fn cursor_stat(&self,) -> Option<(usize, usize, usize,),> {
    match self.cursor_range {
      Some(range,) => {
        let to = range.primary;
        let from = range.secondary;
        Some((
          to.rcursor.row + 1,
          to.rcursor.column + 1,
          to.ccursor.index.abs_diff(from.ccursor.index,),
        ),)
      }
      None => None,
    }
  }
}

#[derive(Debug, Clone, Copy,)]
pub enum ContextMenu {
  Separator,
  Item(ContextMenuAction,),
}

#[derive(Debug, Clone, Copy,)]
pub enum ContextMenuAction {
  Close,
  CloseAll,
  CloseOthers,
  CloseToRight,
  CloseSaved,
  CopyFullPath,
  CopyRelativePath,
  OpenInNative,
}

#[derive(Debug, Clone, Copy,)]
pub enum CloseAction {
  Close(usize,),
  CloseOthers(usize,),
  CloseToRight(usize,),
  CloseSaved,
  CloseAll,
}

pub struct OpenFiles {
  tx: SyncSender<action::Action,>,
  files: Vec<OpenFile,>,
  current_index: usize,
  current_index_changed: bool,
}

impl OpenFiles {
  pub fn new(tx: SyncSender<action::Action,>,) -> Self {
    Self {
      tx,
      files: vec![],
      current_index: usize::MAX,
      current_index_changed: false,
    }
  }
  fn set_current_index(
    &mut self,
    index: usize,
  ) {
    self.current_index = index;
    self.current_index_changed = true;
  }

  fn get_and_reset_current_index_changed(&mut self,) -> bool {
    let old = self.current_index_changed;
    self.current_index_changed = false;
    old
  }

  pub fn current_file(&self,) -> Option<&OpenFile,> {
    if self.current_index >= self.files.len() {
      None
    } else {
      Some(&self.files[self.current_index],)
    }
  }

  fn current_file_mut(&mut self,) -> Option<&mut OpenFile,> {
    if self.current_index >= self.files.len() {
      None
    } else {
      Some(&mut self.files[self.current_index],)
    }
  }

  pub fn is_empty(&self,) -> bool {
    self.files.is_empty()
  }

  pub fn is_current_file(
    &self,
    path: &PathBuf,
  ) -> bool {
    self
      .current_file()
      .map(|f| f.path.eq(path,),)
      .unwrap_or(false,)
  }

  pub fn open_file(
    &mut self,
    path: &PathBuf,
  ) -> Result<(), std::io::Error,> {
    let mut found: usize = usize::MAX;
    for (i, f,) in self.files.iter().enumerate() {
      if f.path.eq(path,) {
        found = i;
      }
    }
    // not found
    if found == usize::MAX {
      let f = OpenFile::new(path,)?;
      self.files.push(f,);
      self.set_current_index(self.files.len() - 1,);
    } else {
      self.set_current_index(found,);
    }
    Ok((),)
  }

  fn close_files(
    &mut self,
    action: CloseAction,
  ) {
    puffin::profile_function!(format!("{action:?}"));
    tracing::info!("CloseAction => {action:?}");

    // 根据 action 推导出 selected_index_list
    let len = self.files.len();
    let selected_index_list = match action {
      CloseAction::CloseOthers(index,) => {
        let mut index_list = (0..len).collect::<Vec<usize,>>();
        index_list.remove(index,);
        index_list
      }
      CloseAction::CloseToRight(index,) => (index + 1..len).collect::<Vec<usize,>>(),
      CloseAction::CloseSaved => self
        .files
        .iter()
        .enumerate()
        .filter_map(|(i, f,)| if f.changed { None } else { Some(i,) },)
        .collect::<Vec<usize,>>(),
      CloseAction::CloseAll => (0..len).collect::<Vec<usize,>>(),
      CloseAction::Close(index,) => vec![index],
    };
    tracing::info!("selected_index_list => {selected_index_list:?}");

    // 从 selected_index_list 根据【是否已保存】推导出 saved_index_list 和 unsaved_files
    let mut saved_index_list: Vec<usize,> = vec![];
    let mut unsaved_files: Vec<&OpenFile,> = vec![];
    for index in selected_index_list.iter() {
      let f: Option<&OpenFile,> = self.files.get(*index,);
      if let Some(f,) = f {
        if f.changed {
          unsaved_files.push(f,);
        } else {
          saved_index_list.push(*index,);
        }
      } else {
        unreachable!("illegal index");
      }
    }

    tracing::info!("saved_index_list => {saved_index_list:?}");

    // 提示 unsaved_files
    if !unsaved_files.is_empty() {
      let one = unsaved_files[0];
      util::toaster()
        .warning(format!("未保存文件：{:?}", &one.path),)
        .set_duration(Some(Duration::from_secs(5,),),);
    }

    // 删除 saved_index_list
    let files: Vec<OpenFile,> = std::mem::take(&mut self.files,);
    self.files = files
      .into_iter()
      .enumerate()
      .filter_map(|(i, f,)| {
        if saved_index_list.contains(&i,) {
          None
        } else {
          Some(f,)
        }
      },)
      .collect();

    // 重新设置 current_index
    let len = self.files.len();
    let current_index = self.current_index;
    let current_index = if len == 0 {
      // 1. 关闭所有 => 重置为 MAX
      usize::MAX
    } else {
      // 2. 删除后只有一个 或者 current_index 是第一个 => 0
      if len == 1 || current_index == 0 {
        0
      }
      // 3. 其他 => current_index 在删除后的新位置 或者 上一个
      else {
        //
        if saved_index_list.contains(&current_index,) {
          index_after_remove(current_index, &saved_index_list,) - 1
        } else {
          index_after_remove(current_index, &saved_index_list,)
        }
      }
    };
    self.set_current_index(current_index,);

    tracing::info!("current_index => {current_index:?}");
  }
}

// ------------------------------------ UI
impl OpenFiles {
  fn show_tabs(
    &mut self,
    ui: &mut egui::Ui,
    open_dir: Option<&PathBuf,>,
    key_actions: &KeyActions,
  ) {
    let current_index = self.current_index;
    let len = self.files.len();

    // tracing::info!("files.len => {len:?}");

    let current_index_changed = self.get_and_reset_current_index_changed();

    for i in 0..len {
      self.show_tab(
        ui,
        open_dir,
        i,
        current_index,
        current_index_changed,
        key_actions,
      );
    }
  }

  fn show_tab(
    &mut self,
    ui: &mut egui::Ui,
    open_dir: Option<&PathBuf,>,
    i: usize,
    current_index: usize,
    current_index_changed: bool,
    _key_actions: &KeyActions,
  ) {
    let f = self.files.get(i,);
    if f.is_none() {
      return;
    }
    let f = f.unwrap();
    let name = f.path().file_name().unwrap().to_string_lossy();
    let relative_path = f
      .path
      .strip_prefix(open_dir.unwrap_or(&PathBuf::new(),),)
      .map(|v| v.to_owned(),)
      .unwrap_or(f.path.to_owned(),);
    let absolute_path = f.path.to_owned();

    // UI
    let tab_title = if f.changed {
      format!("{name} [+]")
    } else {
      format!("{name}")
    };
    // response
    let response = ui
      .selectable_label(i == current_index, tab_title,)
      .on_hover_text_at_pointer(f.path.to_string_lossy(),);
    // 滚动条自动滚动
    // 保证 selected tab 在可见区域
    if current_index_changed && i == self.current_index {
      //TODO
      // response.scroll_to_me(Some(egui::Align::Center,),)
    }
    // 鼠标点击
    if response.clicked() {
      self.set_current_index(i,);
    }
    // 鼠标中键点击
    if response.middle_clicked() {
      self.close_files(CloseAction::Close(i,),);
    }
    // 右键菜单
    response.context_menu(|ui| {
      let menus: Vec<ContextMenu,> = vec![
        ContextMenu::Item(ContextMenuAction::Close,),
        ContextMenu::Item(ContextMenuAction::CloseOthers,),
        ContextMenu::Item(ContextMenuAction::CloseToRight,),
        ContextMenu::Item(ContextMenuAction::CloseSaved,),
        ContextMenu::Item(ContextMenuAction::CloseAll,),
        ContextMenu::Item(ContextMenuAction::CopyFullPath,),
        ContextMenu::Item(ContextMenuAction::CopyRelativePath,),
        ContextMenu::Separator,
        ContextMenu::Item(ContextMenuAction::OpenInNative,),
      ];
      let mut handle_context_menu = |action, ui: &mut egui::Ui| match action {
        ContextMenuAction::Close => self.close_files(CloseAction::Close(i,),),
        ContextMenuAction::CloseAll => self.close_files(CloseAction::CloseAll,),
        ContextMenuAction::CloseOthers => self.close_files(CloseAction::CloseOthers(i,),),
        ContextMenuAction::CloseToRight => self.close_files(CloseAction::CloseToRight(i,),),
        ContextMenuAction::CloseSaved => self.close_files(CloseAction::CloseSaved,),
        ContextMenuAction::CopyFullPath => {
          util::set_clipboard(ui.ctx(), absolute_path.to_string_lossy(),);
        }
        ContextMenuAction::CopyRelativePath => {
          util::set_clipboard(ui.ctx(), relative_path.to_string_lossy(),);
        }
        ContextMenuAction::OpenInNative => {
          util::open_in_native(&absolute_path,);
        }
      };
      ui.style_mut().wrap = Some(false,);
      for m in menus.iter() {
        match m {
          ContextMenu::Separator => {
            ui.separator();
          }
          &ContextMenu::Item(action,) => {
            let text = text::open_files_context_menu_text(&action,);
            let btn = egui::Button::new(text,);
            if ui.add(btn,).clicked() {
              ui.close_menu();
              handle_context_menu(action, ui,);
            }
          }
        }
      }
    },);
  }

  pub fn show_tab_bar(
    &mut self,
    ui: &mut egui::Ui,
    vertical: bool,
    open_dir: Option<&std::path::PathBuf,>,
    key_actions: &KeyActions,
  ) -> egui::InnerResponse<(),> {
    puffin::profile_function!();

    let id = id::TAB_BAR;

    if vertical {
      ui::left_panel(id, ui.ctx(),).show_inside(ui, |ui| {
        ui.style_mut().wrap = Some(false,);
        egui::ScrollArea::both()
          .auto_shrink([true, false,],)
          .show(ui, |ui| {
            ui.vertical(|ui| {
              let spacing_size = ui.spacing().item_spacing.y;
              ui.add_space(spacing_size,);
              self.show_tabs(ui, open_dir, key_actions,);
              ui.add_space(spacing_size,);
            },);
          },);
      },)
    } else {
      ui::top_panel(id, ui.ctx(),)
        .exact_height(style::TAB_BAR_HEIGHT,)
        .show_inside(ui, |ui| {
          egui::ScrollArea::horizontal().show(ui, |ui| {
            ui.horizontal_centered(|ui| {
              let spacing_size = ui.spacing().item_spacing.x;
              ui.add_space(spacing_size,);
              self.show_tabs(ui, open_dir, key_actions,);
              ui.add_space(spacing_size,);
            },);
          },);
        },)
    }
  }

  pub fn show_text_editor(
    &mut self,
    ui: &mut egui::Ui,
  ) {
    puffin::profile_function!();

    let current_file_mut = self.current_file_mut();
    if current_file_mut.is_none() {
      return;
    }
    // File info
    let f = current_file_mut.unwrap();
    let file_ext = f.extension().to_string();
    let line_count = f.content.lines().count();
    // TextEditor info
    let hl_line_number = f.cursor_range.map(|v| v.primary.rcursor.row + 1,);
    let text_editor_id = f.id();
    let outter_scroll_area_id = egui::Id::new(format!(
      "text_editor_outter_scroll_area_{}",
      f.path.to_string_lossy()
    ),);
    let inner_scroll_area_id = egui::Id::new(format!(
      "text_editor_inner_scroll_area_{}",
      f.path.to_string_lossy()
    ),);

    // 竖向滚动
    egui::ScrollArea::vertical()
      .id_source(outter_scroll_area_id,)
      .show(ui, |ui| {
        // 行号栏
        show_line_number_bar(ui, line_count, hl_line_number,);
        // editor UI
        let response = text_editor_ui(
          ui,
          &file_ext,
          &mut f.content,
          text_editor_id,
          inner_scroll_area_id,
        );
        // editor response
        let mut editor_output = response.inner;
        let mut cursor_range = editor_output.cursor_range;
        let galley = editor_output.galley;
        let mut content_changed = false;

        // 记录 cursor, 给 status_bar 使用
        f.cursor_range = cursor_range;

        // 如果文本出现变更，设置“未保存”状态
        if editor_output.response.changed() {
          f.mark_changed();
        }

        // Ctrl+J 换行并缩进
        ui.ctx().input_mut(|i| {
          let key = action::parse_shortcut("Ctrl+J",).unwrap();
          if i.consume_shortcut(&key,) {
            (cursor_range, content_changed,) =
              new_line_and_auto_indent(f, cursor_range, galley.clone(),);
          }
        },);

        if content_changed {
          f.mark_changed();
        }

        // 设置 cursor
        if editor_output.cursor_range != cursor_range {
          editor_output.state.set_cursor_range(cursor_range,);
          tracing::info!("==> store state...");
          editor_output.state.store(ui.ctx(), text_editor_id,);
          tracing::info!("==> store successfully!");
        }

        // Ctrl+S 保存文件
        ui.ctx().input_mut(|i| {
          let key = action::parse_shortcut("Ctrl+S",).unwrap();
          if i.consume_shortcut(&key,) {
            match f.save() {
              Ok(_,) => f.changed = false,
              Err(e,) => {
                // toast
                util::toaster()
                  .error(e.to_string(),)
                  .set_duration(Some(Duration::from_secs(5,),),);
              }
            }
          }
        },);
        //
      },);
  }
}

fn text_editor_ui(
  ui: &mut egui::Ui,
  file_ext: &str,
  content: &mut String,
  text_editor_id: egui::Id,
  inner_scroll_area_id: egui::Id,
) -> egui::scroll_area::ScrollAreaOutput<egui::text_edit::TextEditOutput,> {
  let mut layouter = |ui: &egui::Ui, text: &str, _wrap_width: f32| {
    let hl_key = hl::HlKey::new(None, ui::dark_mode(), file_ext,);
    let layout_job = hl::layout::get_layout_job_from_cache(ui.ctx(), &hl_key, text,);
    ui.fonts(|f| f.layout_job(layout_job,),)
  };

  let text_editor = egui::TextEdit::multiline(content,)
    .id(text_editor_id,)
    .code_editor()
    .layouter(&mut layouter,)
    .frame(true,)
    // .lock_focus(true,)
    .desired_width(f32::INFINITY,)
    .desired_rows(40,);

  // 横向滚动
  egui::ScrollArea::horizontal()
    .id_source(inner_scroll_area_id,)
    .show(ui, |ui| {
      // ui.horizonta(|ui| {
      // text editor
      // (*) 获取焦点，保证 cursor 正确显示
      ui.memory_mut(|m| m.request_focus(text_editor_id,),);
      text_editor.show(ui,)
      // },)
      // .inner
    },)
}

/// 行号栏
fn show_line_number_bar(
  ui: &mut egui::Ui,
  line_count: usize,
  hl_line_number: Option<usize,>,
) {
  ui::left_panel("line_number_bar", ui.ctx(),)
    .resizable(false,)
    .min_width(0.,) // 保证除内容外没有额外的宽度占用
    .show_inside(ui, |ui| {
      ui.vertical(|ui| {
        ui.style_mut().wrap = Some(false,); // 防止内容换行
        ui.style_mut().spacing.item_spacing.y = 0.;
        ui.add_space(2.,);
        for i in 1..=line_count {
          let mut text = egui::RichText::new(format!(" {i} "),).font(text::text_editor_font(),);
          if let Some(nr,) = hl_line_number {
            if nr == i {
              text = egui::RichText::new(format!("{i} "),)
                .font(text::text_editor_font(),)
                .strong();
            }
          }
          ui.label(text,);
        }
      },);
    },);
}

fn new_line_and_auto_indent(
  f: &mut OpenFile,
  cursor_range: Option<egui::text_edit::CursorRange,>,
  galley: std::sync::Arc<eframe::epaint::Galley,>,
) -> (Option<egui::widgets::text_edit::CursorRange,>, bool,) {
  match cursor_range {
    None => (None, false,),
    Some(cr,) => {
      let row = cr.primary.rcursor.row;
      match f.content.lines().nth(row,) {
        None => (None, false,),
        Some(line,) => {
          tracing::info!("current line => {line}");
          let space_count = calc_next_line_indent_space_count(line,);
          tracing::info!("next line indent space_count => {space_count}");
          let spaces = " ".repeat(space_count,);

          let row_end_index = galley.cursor_end_of_row(&cr.primary,).ccursor.index;
          egui::TextBuffer::insert_text(&mut f.content, &format!("\n{spaces}"), row_end_index,);

          let new_cr = Some(egui::widgets::text_edit::CursorRange::one(
            galley.cursor_end_of_row(&galley.cursor_down_one_row(&cr.primary,),),
          ),);
          (new_cr, true,)
        }
      }
    }
  }
}

fn calc_next_line_indent_space_count(cur_line: &str,) -> usize {
  let mut space_count = calc_current_line_indent_space_count(cur_line,);
  let line = cur_line.trim_end();
  if line.ends_with('{',) || line.ends_with('[',) || line.ends_with('(',) {
    space_count += 4;
  }
  space_count
}

fn calc_current_line_indent_space_count(cur_line: &str,) -> usize {
  let mut space_count = 0;
  for c in cur_line.chars() {
    match c {
      ' ' => space_count += 1,
      '\t' => space_count += 4,
      _ => break,
    }
  }
  space_count
}

fn index_after_remove(
  index: usize,
  to_remove_index_list: &[usize],
) -> usize {
  index - to_remove_index_list.iter().filter(|&i| *i < index,).count()
}

impl action::Handle for OpenFiles {
  fn handle(
    &mut self,
    _action: &action::Action,
  ) {
    {}
  }
}
