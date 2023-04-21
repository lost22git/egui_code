use std::{collections::HashMap, path::PathBuf, str::FromStr, sync::mpsc::SyncSender};

use eframe::egui;

// ------------------------------------ Action

#[allow(clippy::enum_variant_names)]
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, PartialOrd,)]
pub enum Action {
  NoOp,
  ExitApp,
  ToggleFullScreen,
  ToggleDecorations,
  ToggleStatusBar,
  ToggleToolBar,
  ToggleExplorer,
  ToggleTerminal,
  ToggleVerticalTabBar,
  ZoomIn,
  ZoomOut,
  ZoomReset,
  ZoomSet(f32,),
  OpenDebugWindow,
  OpenPuffinViewer,
  OpenAboutWindow,
  OpenSettingWindow,
  OpenFolder,
  SetOpenDir(PathBuf,),
}

impl Action {
  pub fn name(&self,) -> &'static str {
    action_name(self,)
  }
}

impl FromStr for &'static Action {
  type Err = String;

  fn from_str(s: &str,) -> Result<Self, Self::Err,> {
    parse_action(s,)
  }
}

pub fn action_name(action: &Action,) -> &'static str {
  match action {
    Action::NoOp => "NoOp",
    Action::ExitApp => "ExitApp",
    Action::ToggleFullScreen => "ToggleFullScreen",
    Action::ToggleDecorations => "ToggleDecorations",
    Action::ToggleStatusBar => "ToggleStatusBar",
    Action::ToggleToolBar => "ToggleToolBar",
    Action::ToggleExplorer => "ToggleExplorer",
    Action::ToggleTerminal => "ToggleTerminal",
    Action::ToggleVerticalTabBar => "ToggleVerticalTabBar",
    Action::ZoomIn => "ZoomIn",
    Action::ZoomOut => "ZoomOut",
    Action::ZoomReset => "ZoomReset",
    Action::ZoomSet(_,) => "ZoomSet",
    Action::OpenDebugWindow => "OpenDebugWindow",
    Action::OpenPuffinViewer => "OpenPuffinViewer",
    Action::OpenAboutWindow => "OpenAboutWindow",
    Action::OpenSettingWindow => "OpenSettingWindow",
    Action::OpenFolder => "OpenFolder",
    Action::SetOpenDir(_,) => "SetOpenDir",
  }
}

static ACTION_MAP: once_cell::sync::Lazy<HashMap<&str, Action,>,> =
  once_cell::sync::Lazy::new(|| {
    [
      Action::ToggleFullScreen,
      Action::ToggleDecorations,
      Action::ToggleStatusBar,
      Action::ToggleToolBar,
      Action::ToggleExplorer,
      Action::ToggleTerminal,
      Action::ToggleVerticalTabBar,
      Action::ZoomIn,
      Action::ZoomOut,
      Action::ZoomReset,
      Action::OpenDebugWindow,
      Action::OpenPuffinViewer,
      Action::OpenFolder,
    ]
    .into_iter()
    .map(|v| (v.name(), v,),)
    .collect::<HashMap<&str, Action,>>()
  },);

pub fn parse_action(action_name: &str,) -> Result<&'static Action, String,> {
  ACTION_MAP
    .get(action_name,)
    .ok_or(format!("解析 action => {} 失败", action_name,),)
}

// ------------------------------------ Handle

pub trait Handle {
  fn handle(
    &mut self,
    action: &Action,
  );
}

// ------------------------------------ KeyActions

#[derive(Debug,)]
pub struct KeyActions {
  tx: SyncSender<Action,>,
  map: HashMap<egui::KeyboardShortcut, Action,>,
}

impl KeyActions {
  pub fn new(tx: SyncSender<Action,>,) -> Self {
    Self {
      tx,
      map: HashMap::new(),
    }
  }

  pub fn init(&mut self,) -> Result<(), String,> {
    self.load_default()?;
    Ok((),)
  }

  pub fn insert(
    &mut self,
    key: egui::KeyboardShortcut,
    action: Action,
  ) -> Result<(), String,> {
    if let Some(act,) = self.map.get(&key,) {
      let key_name = key.format(&egui::ModifierNames::NAMES, cfg!(target_os = "macos"),);
      let action_name = act.name();

      return Err(format!("key冲突，已绑定 {} => {} ", key_name, action_name),);
    }

    self.map.insert(key, action,);
    Ok((),)
  }

  pub fn load_default(&mut self,) -> Result<(), String,> {
    self.insert(parse_shortcut("Alt+Enter",)?, Action::ToggleFullScreen,)?;
    self.insert(parse_shortcut("F11",)?, Action::ToggleFullScreen,)?;
    self.insert(
      parse_shortcut("Alt+Shift+Enter",)?,
      Action::ToggleDecorations,
    )?;
    self.insert(parse_shortcut("Ctrl+Minus",)?, Action::ZoomOut,)?;
    self.insert(parse_shortcut("Ctrl+Plus",)?, Action::ZoomIn,)?;
    self.insert(parse_shortcut("Ctrl+0",)?, Action::ZoomReset,)?;
    self.insert(parse_shortcut("F12",)?, Action::OpenDebugWindow,)?;
    self.insert(parse_shortcut("Alt+F12",)?, Action::OpenPuffinViewer,)?;
    self.insert(parse_shortcut("Ctrl+F12",)?, Action::OpenAboutWindow,)?;
    self.insert(parse_shortcut("Ctrl+Shift+Q",)?, Action::ExitApp,)?;
    self.insert(parse_shortcut("Ctrl+Shift+O",)?, Action::OpenFolder,)?;
    self.insert(parse_shortcut("Ctrl+Shift+S",)?, Action::OpenSettingWindow,)?;
    self.insert(parse_shortcut("Alt+1",)?, Action::ToggleExplorer,)?;
    self.insert(parse_shortcut("Alt+3",)?, Action::ToggleTerminal,)?;
    self.insert(parse_shortcut("Alt+4",)?, Action::ToggleStatusBar,)?;
    self.insert(parse_shortcut("Alt+5",)?, Action::ToggleToolBar,)?;

    Ok((),)
  }

  pub fn get_action_keys<'a,>(
    &'a self,
    action: &'a Action,
  ) -> Vec<&'a egui::KeyboardShortcut,> {
    let mut ret = vec![];
    for (key, act,) in self.map.iter() {
      if act == action {
        ret.push(key,);
      }
    }
    ret
  }
}

impl KeyActions {
  pub fn bind_to_context(
    &self,
    ctx: &egui::Context,
  ) {
    ctx.input_mut(|input| {
      for (key, act,) in self.map.iter() {
        if input.consume_shortcut(key,) {
          let _ = self.tx.send(act.clone(),);
        }
      }
    },);
  }
}

// ------------------------------------ Parsing egui [Key] [Modifier] [KeyboardShortcut]

static NAME_TO_KEY: once_cell::sync::Lazy<HashMap<&'static str, egui::Key,>,> =
  once_cell::sync::Lazy::new(|| {
    HashMap::from([
      (egui::Key::A.name(), egui::Key::A,),
      (egui::Key::B.name(), egui::Key::B,),
      (egui::Key::C.name(), egui::Key::C,),
      (egui::Key::D.name(), egui::Key::D,),
      (egui::Key::E.name(), egui::Key::E,),
      (egui::Key::F.name(), egui::Key::F,),
      (egui::Key::G.name(), egui::Key::G,),
      (egui::Key::H.name(), egui::Key::H,),
      (egui::Key::I.name(), egui::Key::I,),
      (egui::Key::J.name(), egui::Key::J,),
      (egui::Key::K.name(), egui::Key::K,),
      (egui::Key::L.name(), egui::Key::L,),
      (egui::Key::M.name(), egui::Key::M,),
      (egui::Key::N.name(), egui::Key::N,),
      (egui::Key::O.name(), egui::Key::O,),
      (egui::Key::P.name(), egui::Key::P,),
      (egui::Key::Q.name(), egui::Key::Q,),
      (egui::Key::R.name(), egui::Key::R,),
      (egui::Key::S.name(), egui::Key::S,),
      (egui::Key::T.name(), egui::Key::T,),
      (egui::Key::U.name(), egui::Key::U,),
      (egui::Key::V.name(), egui::Key::V,),
      (egui::Key::W.name(), egui::Key::W,),
      (egui::Key::X.name(), egui::Key::X,),
      (egui::Key::Y.name(), egui::Key::Y,),
      (egui::Key::Z.name(), egui::Key::Z,),
      (egui::Key::ArrowDown.name(), egui::Key::ArrowDown,),
      (egui::Key::ArrowUp.name(), egui::Key::ArrowUp,),
      (egui::Key::ArrowLeft.name(), egui::Key::ArrowLeft,),
      (egui::Key::ArrowRight.name(), egui::Key::ArrowRight,),
      (egui::Key::Backspace.name(), egui::Key::Backspace,),
      (egui::Key::Delete.name(), egui::Key::Delete,),
      (egui::Key::End.name(), egui::Key::End,),
      (egui::Key::Enter.name(), egui::Key::Enter,),
      (egui::Key::Escape.name(), egui::Key::Escape,),
      (egui::Key::F1.name(), egui::Key::F1,),
      (egui::Key::F2.name(), egui::Key::F2,),
      (egui::Key::F3.name(), egui::Key::F3,),
      (egui::Key::F4.name(), egui::Key::F4,),
      (egui::Key::F5.name(), egui::Key::F5,),
      (egui::Key::F6.name(), egui::Key::F6,),
      (egui::Key::F7.name(), egui::Key::F7,),
      (egui::Key::F8.name(), egui::Key::F8,),
      (egui::Key::F9.name(), egui::Key::F9,),
      (egui::Key::F10.name(), egui::Key::F10,),
      (egui::Key::F11.name(), egui::Key::F11,),
      (egui::Key::F12.name(), egui::Key::F12,),
      (egui::Key::F13.name(), egui::Key::F13,),
      (egui::Key::F14.name(), egui::Key::F14,),
      (egui::Key::F15.name(), egui::Key::F15,),
      (egui::Key::F16.name(), egui::Key::F16,),
      (egui::Key::F17.name(), egui::Key::F17,),
      (egui::Key::F18.name(), egui::Key::F18,),
      (egui::Key::F19.name(), egui::Key::F19,),
      (egui::Key::F20.name(), egui::Key::F20,),
      (egui::Key::Home.name(), egui::Key::Home,),
      (egui::Key::Insert.name(), egui::Key::Insert,),
      (egui::Key::Minus.name(), egui::Key::Minus,),
      (egui::Key::Num0.name(), egui::Key::Num0,),
      (egui::Key::Num1.name(), egui::Key::Num1,),
      (egui::Key::Num2.name(), egui::Key::Num2,),
      (egui::Key::Num3.name(), egui::Key::Num3,),
      (egui::Key::Num4.name(), egui::Key::Num4,),
      (egui::Key::Num5.name(), egui::Key::Num5,),
      (egui::Key::Num6.name(), egui::Key::Num6,),
      (egui::Key::Num7.name(), egui::Key::Num7,),
      (egui::Key::Num8.name(), egui::Key::Num8,),
      (egui::Key::Num9.name(), egui::Key::Num9,),
      (egui::Key::PageDown.name(), egui::Key::PageDown,),
      (egui::Key::PageUp.name(), egui::Key::PageUp,),
      (egui::Key::PlusEquals.name(), egui::Key::PlusEquals,),
      (egui::Key::Space.name(), egui::Key::Space,),
      (egui::Key::Tab.name(), egui::Key::Tab,),
    ],)
  },);

pub fn parse_key(key_name: &str,) -> Result<&'static egui::Key, String,> {
  NAME_TO_KEY
    .get(key_name,)
    .ok_or(format!("无法解析 key => {}", key_name),)
}

pub fn parse_modifier(modifier_name: &str,) -> Result<egui::Modifiers, String,> {
  match modifier_name {
    "Ctrl" => Ok(egui::Modifiers::CTRL,),
    "Alt" => Ok(egui::Modifiers::ALT,),
    "Shift" => Ok(egui::Modifiers::SHIFT,),
    "Cmd" if cfg!(target_os = "macos") => Ok(egui::Modifiers::MAC_CMD,),
    "Cmd" if cfg!(not(target_os = "macos")) => Ok(egui::Modifiers::COMMAND,),
    _ => Err(format!("无法解析 modifier => {}", modifier_name),),
  }
}

pub fn parse_shortcut(key_name: &str,) -> Result<egui::KeyboardShortcut, String,> {
  let tokens = key_name.split('+',).collect::<Vec<&str,>>();
  let len = tokens.len();
  if len == 0 {
    Err(format!("无法解析 shortcut => {}", key_name),)
  } else if len == 1 {
    let key = *parse_key(tokens[0],)?;
    Ok(egui::KeyboardShortcut {
      modifiers: egui::Modifiers::NONE,
      key,
    },)
  } else if len == 2 {
    let key = *parse_key(tokens[len - 1],)?;
    let m = parse_modifier(tokens[0],)?;
    Ok(egui::KeyboardShortcut {
      modifiers: m,
      key,
    },)
  } else if len == 3 {
    let key = *parse_key(tokens[len - 1],)?;
    let m1 = parse_modifier(tokens[0],)?;
    let m2 = parse_modifier(tokens[1],)?;
    Ok(egui::KeyboardShortcut {
      modifiers: m1 | m2,
      key,
    },)
  } else if len == 4 {
    let key = *parse_key(tokens[len - 1],)?;
    let m1 = parse_modifier(tokens[0],)?;
    let m2 = parse_modifier(tokens[1],)?;
    let m3 = parse_modifier(tokens[2],)?;
    Ok(egui::KeyboardShortcut {
      modifiers: m1 | m2 | m3,
      key,
    },)
  } else {
    Err(format!("无法解析 shortcut => {}", key_name),)
  }
}

pub fn format_key(key: &egui::KeyboardShortcut,) -> String {
  key.format(&egui::ModifierNames::NAMES, cfg!(target_os = "macos"),)
}

#[cfg(test)]
mod test {
  use std::sync::mpsc::sync_channel;

  use eframe::egui::{self};

  use crate::action::{format_key, parse_key, parse_modifier, parse_shortcut, Action};

  use super::KeyActions;

  #[test]
  fn print_keyactions() {
    let (tx, _,) = sync_channel(10,);
    let mut ka = KeyActions::new(tx,);
    let _ = ka.init();
    for (key, act,) in ka.map.iter() {
      let key_name = format_key(key,);
      let action_name = act.name();
      println!("{} => {:?}:{}", key_name, act, action_name);
    }
  }
  #[test]
  fn test_parse_key() {
    assert_eq!(parse_key("0").unwrap(), &egui::Key::Num0);
    assert_eq!(parse_key("A").unwrap(), &egui::Key::A);
    assert_eq!(parse_key("F1").unwrap(), &egui::Key::F1);
    assert_eq!(parse_key("Space").unwrap(), &egui::Key::Space);
    assert_eq!(parse_key("Backspace").unwrap(), &egui::Key::Backspace);
    assert_eq!(parse_key("Escape").unwrap(), &egui::Key::Escape);
    assert_eq!(parse_key("Plus").unwrap(), &egui::Key::PlusEquals);
    assert_eq!(parse_key("Minus").unwrap(), &egui::Key::Minus);
    assert_eq!(parse_key("Tab").unwrap(), &egui::Key::Tab);
    assert_eq!(parse_key("Enter").unwrap(), &egui::Key::Enter);
    assert_eq!(parse_key("End").unwrap(), &egui::Key::End);
    assert_eq!(parse_key("PageUp").unwrap(), &egui::Key::PageUp);
    assert_eq!(parse_key("Up").unwrap(), &egui::Key::ArrowUp);
  }

  #[test]
  fn test_parse_modifier() {
    assert_eq!(parse_modifier("Cmd").unwrap(), egui::Modifiers::COMMAND);
    assert_eq!(parse_modifier("Ctrl").unwrap(), egui::Modifiers::CTRL);
    assert_eq!(parse_modifier("Shift").unwrap(), egui::Modifiers::SHIFT);
    assert_eq!(parse_modifier("Alt").unwrap(), egui::Modifiers::ALT);
  }

  #[test]
  fn test_parse_shortcut() {
    assert_eq!(
      parse_shortcut("Alt+Shift+Ctrl+K").unwrap(),
      egui::KeyboardShortcut {
        modifiers: egui::Modifiers::ALT | egui::Modifiers::SHIFT | egui::Modifiers::CTRL,
        key: egui::Key::K
      }
    );
  }

  #[test]
  fn test_action_compare() {
    assert_eq!(Action::ToggleFullScreen, Action::ToggleFullScreen);
  }

  #[test]
  fn test_parse_action() {
    assert_eq!(
      &Action::ToggleFullScreen,
      "ToggleFullScreen".parse::<&'static Action>().unwrap()
    );
  }
}
