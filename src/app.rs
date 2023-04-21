use std::{
  cell::RefCell,
  rc::Rc,
  sync::mpsc::{sync_channel, Receiver, SyncSender},
};

use eframe::egui;

use crate::{
  action::{self, Action, Handle, KeyActions},
  component::{
    file_tree::{self, Node},
    menu_bar::MenuBar,
    open_file::OpenFiles,
    status_bar::StatusBar,
    tool_bar::{self, ToolBar},
  },
  dev_tool, font, frame_history, id,
  images::{self, CachedImage},
  style, text, ui, util,
  window::about::AboutWindow,
  window::debug::DebugWindow,
  window::exit::ExitWindow,
  window::setting::SettingWindow,
};

pub struct MyApp {
  exit_app: bool,
  fullscreen: bool,
  decorations: bool,
  zoom_default: f32,
  zoom: f32,

  vertical_tab_bar: bool,
  show_tool_bar: bool,
  show_status_bar: bool,
  show_terminal: bool,

  exit_window: ExitWindow,

  show_debug_window: bool,
  debug_window: DebugWindow,

  about_window: AboutWindow,
  show_about_window: bool,

  setting_window: SettingWindow,
  show_setting_window: bool,

  menu_bar: MenuBar,
  tool_bar: ToolBar,
  status_bar: StatusBar,

  // logo 图片
  logo_image: CachedImage<&'static [u8],>,

  // 打开的目录
  open_dir: Option<Rc<RefCell<Node,>,>,>,

  // 打开的文件列表
  open_files: OpenFiles,

  key_actions: KeyActions,

  tx: SyncSender<Action,>,
  rx: Receiver<Action,>,
}

impl eframe::App for MyApp {
  fn clear_color(
    &self,
    visuals: &egui::Visuals,
  ) -> [f32; 4] {
    ui::eframe_transparency(visuals,).to_normalized_gamma_f32()
  }

  fn on_close_event(&mut self,) -> bool {
    self.exit_window.on_frame_close_event()
  }

  fn update(
    &mut self,
    ctx: &egui::Context,
    frame: &mut eframe::Frame,
  ) {
    // exit app ?
    if self.exit_app {
      self.exit_app = false;
      frame.close();
    }
    // 记录 frame
    frame_history::record_frame_time(ctx, frame,);
    // 快捷键绑定
    self.key_actions.bind_to_context(ctx,);
    // 配置 Style
    self.configure_style(ctx,);
    // fullscreen
    frame.set_fullscreen(self.fullscreen,);
    // decorations
    frame.set_decorations(self.decorations,);
    // zoom
    ctx.set_pixels_per_point(self.zoom,);
    // Toast UI
    util::toaster().show(ctx,);
    // 退出确认窗口
    self.exit_window.show(ctx, frame,);
    // Debug 窗口
    self.debug_window.show(ctx, &mut self.show_debug_window,);
    // 设置窗口
    self
      .setting_window
      .show(ctx, &mut self.show_setting_window,);
    // 关于窗口
    self.about_window.show(ctx, &mut self.show_about_window,);

    // 主界面
    ui::central_panel(ctx,).show(ctx, |ui| {
      // 同步 dark_mode
      // 当 dark_mode 发生变更时，执行一些回调
      if ui::sync_mode(ui.visuals().dark_mode,) {
        self.on_mode_changed();
      }

      // 有其他 window 可交互时，main pane 不可交互
      ui.set_enabled(!self.show_setting_window && !self.exit_window.show,);

      // 主界面
      self.show_top_menu_bar(ui, frame,);
      self
        .status_bar
        .show(ui, self.show_status_bar, self.open_files.current_file(),);
      self.show_center_panel(ui,);
    },);

    // handle action
    loop {
      match self.rx.try_recv() {
        Err(_,) => break,
        Ok(action,) => {
          self.handle(&action,);
          self.open_files.handle(&action,);
          self.menu_bar.handle(&action,);
          self.tool_bar.handle(&action,);
          self.status_bar.handle(&action,);
        }
      }
    }
  }
}

impl MyApp {
  pub fn new(cc: &eframe::CreationContext<'_,>,) -> Self {
    // 初始化字体 支持中文
    font::install_font(&cc.egui_ctx,);
    // logo 图片
    let logo_image = CachedImage::new("logo_image".to_string(), images::logo,);

    let zoom_default = cc.integration_info.native_pixels_per_point.unwrap_or(1.,);

    let (tx, rx,) = sync_channel::<Action,>(1000,);

    let mut key_actions = KeyActions::new(tx.clone(),);
    key_actions.init().expect("init KeyAction 失败",);

    Self {
      key_actions,

      exit_app: false,
      fullscreen: false,
      decorations: true,
      zoom_default,
      zoom: zoom_default,

      exit_window: ExitWindow::default(),

      debug_window: DebugWindow::default(),
      show_debug_window: false,

      about_window: AboutWindow::new(),
      show_about_window: false,

      setting_window: SettingWindow::default(),
      show_setting_window: false,

      menu_bar: MenuBar::new(tx.clone(),),

      status_bar: StatusBar::new(tx.clone(),),
      show_status_bar: true,

      tool_bar: ToolBar::new(tx.clone(),),
      show_tool_bar: true,

      show_terminal: false,

      logo_image,

      open_dir: None,
      open_files: OpenFiles::new(tx.clone(),),
      vertical_tab_bar: false,

      tx,
      rx,
    }
  }
}

// ------------------------------------ UI

impl MyApp {
  /// 配置 Style
  fn configure_style(
    &mut self,
    ctx: &egui::Context,
  ) {
    let mut style = (*ctx.style()).clone();
    text::configure_style(&mut style,);
    self.debug_window.configure_style(&mut style,);
    ctx.set_style(style,);
  }

  /// 顶部
  fn show_top_menu_bar(
    &mut self,
    ui: &mut egui::Ui,
    _frame: &mut eframe::Frame,
  ) {
    ui::top_panel("top_menu_bar", ui.ctx(),)
      .exact_height(style::MENU_BAR_HEIGHT,)
      .show_inside(ui, |ui| {
        ui.horizontal_centered(|ui| {
          // logo
          {
            let logo_size = style::MENU_BAR_HEIGHT * 1.5;
            let margin_left = -logo_size / 16.;
            egui::Frame::none()
              .inner_margin(egui::Margin {
                left: margin_left,
                right: margin_left * 3.,
                top: margin_left * 3.,
                bottom: margin_left * 5.,
              },)
              .show(ui, |ui| {
                ui.add(
                  self
                    .logo_image
                    .as_image_widget(ui, egui::Vec2::splat(logo_size,),),
                );
              },);
          }
          // 顶部菜单栏
          self.menu_bar.show(ui, &self.key_actions,);
        },);
      },);
  }

  /// 中部
  fn show_center_panel(
    &mut self,
    ui: &mut egui::Ui,
  ) {
    ui::central_panel( ui.ctx())
    .show_inside(ui, |ui| {
      ui.horizontal_centered(|ui| {
        // 工具栏
        self.tool_bar.show(ui,self.show_tool_bar);
        //
        self.show_explorer_side_panel(ui);
        //
        ui::central_panel(ui.ctx())
          .show_inside(ui, |ui| {
            // terminal
            ui::bottom_panel("terminal_panel", ui.ctx())
            .resizable(true)
            .default_height(style::TERMINAL_PANEL_DEFAULT_HEIGHT)
            .max_height(style::TERMINAL_PANEL_MAX_HEIGHT)
            .show_animated_inside(ui, self.show_terminal, |ui| {
              ui::top_panel("terminal_tab_bar", ui.ctx())
                .exact_height(style::EXPLORER_TOP_HEIGHT)
                .show_inside(ui, |ui| {
                  ui.horizontal_centered(|ui| {
                    let spacing_size = ui.spacing().item_spacing;
                    ui.add_space(spacing_size.x);
                    if ui.button("问题").clicked() {
                    }
                    if ui.button("调试").clicked() {
                    }
                    if ui.button("终端").clicked() {
                    }
                  });
                });

                egui::ScrollArea::both().show(ui, |ui| {
                  ui.set_width(ui.available_width());
                  ui.set_height(ui.available_height());
                  ui.style_mut().wrap = Some(false);
                  ui.monospace(">>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>> ");
                });
            });

            ui::central_panel(ui.ctx())
              .show_inside(ui,|ui|{

              if self.open_files.is_empty(){
                return;
              }

              let open_dir = self
                .open_dir
                .as_ref()
                .map(|v| RefCell::borrow(v).path().to_owned());
              //
              // tab bar
              self.open_files.show_tab_bar(ui, self.vertical_tab_bar, open_dir.as_ref(),&self.key_actions)
              .response.context_menu(|ui|{
                if ui.button("vertical tab bar").clicked() {
                  ui.close_menu();
                  self.toggle_vertical_tab_bar();
                }
              });
              // 编辑器
              ui::central_panel(ui.ctx(),).show_inside(ui, |ui| {
                self.open_files.show_text_editor(ui);
              },);
              //
            });
          });
        });
    });
  }

  /// 探索面板
  fn show_explorer_side_panel(
    &mut self,
    ui: &mut egui::Ui,
  ) {
    ui::left_panel(id::EXPLORER, ui.ctx(),)
      .resizable(true,)
      .min_width(style::EXPLORER_MIN_WIDTH,)
      .show_animated_inside(ui, self.show_explorer(), |ui| {
        ui.style_mut().wrap = Some(false,);
        // 探索
        ui::top_panel("explorer_top", ui.ctx(),)
          .exact_height(style::EXPLORER_TOP_HEIGHT,)
          .show_inside(ui, |ui| {
            ui.horizontal_centered(|ui| {
              let spacing_size = ui.spacing().item_spacing;
              ui.add_space(spacing_size.x,);
              ui.heading("探索",);
            },);
          },);
        // 时间线
        ui::bottom_panel("timeline", ui.ctx(),)
          .resizable(true,)
          .show_inside(ui, |ui| {
            let (_state, _header_res, _body_res,) = ui::custom_collapsing(
              ui,
              "timeline_collapsing",
              "时间线",
              false,
              |_ui| {},
              |ui| {
                if ui.selectable_label(false, "文件保存",).clicked() {}
                if ui.selectable_label(false, "文件修改",).clicked() {}
                if ui.selectable_label(false, "文件打开",).clicked() {}
              },
            );
          },);
        // 大纲
        ui::bottom_panel("outline", ui.ctx(),)
          .resizable(true,)
          .show_inside(ui, |ui| {
            let (_state, _header_res, _body_res,) = ui::custom_collapsing(
              ui,
              "outline_collapsing",
              "大纲",
              false,
              |ui| {
                let more_action_button = egui::Button::new("",).frame(false,);
                if ui.add(more_action_button,).clicked() {
                  //
                }
                let collapse_button = egui::Button::new("",).frame(false,);
                if ui.add(collapse_button,).clicked() {
                  //
                }
              },
              |ui| {
                if ui.selectable_label(false, "function",).clicked() {}
                if ui.selectable_label(false, "struct",).clicked() {}
                if ui.selectable_label(false, "impl",).clicked() {}
              },
            );
          },);
        // 文件
        ui::central_panel(ui.ctx(),).show_inside(ui, |ui| {
          if self.open_dir.is_none() {
            ui.vertical_centered_justified(|ui| {
              ui.monospace("当前未打开文件夹",);
              if ui.button("打开文件夹",).clicked() {
                let _ = self.tx.send(Action::OpenFolder,);
              }
            },);
          } else {
            puffin::profile_scope!("show_tree");
            let node = self.open_dir.as_ref().unwrap();
            file_tree::show_tree(node, node, ui, &mut self.open_files, vec![],);
          }
        },);
        //
      },);
  }
}

// ------------------------------------ Data update

impl MyApp {
  pub fn exit_app(&mut self,) {
    self.exit_app = true;
  }
  pub fn toggle_fullscreen(&mut self,) {
    self.fullscreen = !self.fullscreen;
  }

  pub fn toggle_decorations(&mut self,) {
    self.decorations = !self.decorations;
  }

  pub fn zoom_in(&mut self,) {
    self.zoom = ui::zoom_clamp(self.zoom + 0.1,);
  }

  pub fn zoom_out(&mut self,) {
    self.zoom = ui::zoom_clamp(self.zoom - 0.1,);
  }

  pub fn zoom_reset(&mut self,) {
    self.zoom = self.zoom_default;
  }

  pub fn zoom_set(
    &mut self,
    new_zoom: f32,
  ) {
    self.zoom = ui::zoom_clamp(new_zoom,);
  }

  pub fn toggle_tool_bar(&mut self,) {
    self.show_tool_bar = !self.show_tool_bar;
  }

  pub fn toggle_status_bar(&mut self,) {
    self.show_status_bar = !self.show_status_bar;
  }

  pub fn toggle_vertical_tab_bar(&mut self,) {
    self.vertical_tab_bar = !self.vertical_tab_bar;
  }

  pub fn toggle_terminal(&mut self,) {
    self.show_terminal = !self.show_terminal;
  }

  pub fn show_explorer(&self,) -> bool {
    if let Some(item,) = self.tool_bar.current_item() {
      item.id == tool_bar::ToolId::Explorer
    } else {
      false
    }
  }

  pub fn open_about_window(&mut self,) {
    self.show_about_window = true;
  }

  pub fn open_setting_window(&mut self,) {
    self.show_setting_window = true;
  }

  pub fn open_debug_window(&mut self,) {
    self.show_debug_window = true;
  }

  fn set_open_dir(
    &mut self,
    dir_path: Option<std::path::PathBuf,>,
  ) {
    self.open_dir = dir_path.map(|v| {
      Rc::new(RefCell::new(Node::Dir {
        path: v,
        children: Default::default(),
        parent: None,
        expand: true,
      },),)
    },);
  }

  fn on_mode_changed(&mut self,) {
    self.tool_bar.reset();
  }
}

// ------------------------------------ Action handle

impl action::Handle for MyApp {
  fn handle(
    &mut self,
    action: &Action,
  ) {
    puffin::profile_function!();
    match action {
      Action::NoOp => {}
      Action::ExitApp => self.exit_app(),
      Action::ToggleFullScreen => self.toggle_fullscreen(),
      Action::ToggleDecorations => self.toggle_decorations(),
      Action::ToggleStatusBar => self.toggle_status_bar(),
      Action::ToggleToolBar => self.toggle_tool_bar(),
      Action::ToggleExplorer => { /*  此处不处理，交由 ToolBar 处理*/ }
      Action::ToggleTerminal => self.toggle_terminal(),
      Action::ToggleVerticalTabBar => self.toggle_vertical_tab_bar(),
      Action::ZoomIn => self.zoom_in(),
      Action::ZoomOut => self.zoom_out(),
      Action::ZoomReset => self.zoom_reset(),
      Action::ZoomSet(v,) => self.zoom_set(*v,),
      Action::OpenDebugWindow => self.open_debug_window(),
      Action::OpenPuffinViewer => dev_tool::open_puffin_viewer(),
      Action::OpenAboutWindow => self.open_about_window(),
      Action::OpenSettingWindow => self.open_setting_window(),
      Action::OpenFolder => {
        if let Some(dir_path,) = util::open_native_file_dialog() {
          let _ = self.tx.send(Action::SetOpenDir(dir_path,),);
        }
      }
      Action::SetOpenDir(dir_path,) => {
        self.set_open_dir(Some(dir_path.clone(),),);
        if !self.show_explorer() {
          let _ = self.tx.send(Action::ToggleExplorer,);
        }
      }
    }
  }
}
