use syntect::{
  easy::HighlightLines,
  highlighting::{Theme, ThemeSet},
  parsing::{SyntaxReference, SyntaxSet},
};
//
static SYNTAX_SET: once_cell::sync::Lazy<SyntaxSet,> =
  once_cell::sync::Lazy::new(SyntaxSet::load_defaults_newlines,);

static THEME_SET: once_cell::sync::Lazy<ThemeSet,> =
  once_cell::sync::Lazy::new(ThemeSet::load_defaults,);

pub fn syntax_set() -> &'static SyntaxSet {
  &SYNTAX_SET
}

pub fn get_syntax(file_ext: &str,) -> &SyntaxReference {
  let fallback_syntax = || SYNTAX_SET.find_syntax_plain_text();
  SYNTAX_SET
    .find_syntax_by_extension(file_ext,)
    .unwrap_or_else(fallback_syntax,)
}

pub fn get_theme(
  theme_name: Option<&str,>,
  dark_mode: bool,
) -> &Theme {
  let fallback_theme = || {
    if dark_mode {
      THEME_SET.themes.get("Solarized (dark)",).unwrap()
      // THEME_SET.themes.get("base16-ocean.dark",).unwrap()
    } else {
      THEME_SET.themes.get("Solarized (light)",).unwrap()
      // THEME_SET.themes.get("base16-ocean.light",).unwrap()
    }
  };

  match theme_name {
    Some(v,) => THEME_SET.themes.get(v,).unwrap_or_else(fallback_theme,),
    None => fallback_theme.call_once((),),
  }
}

#[derive(Debug, Hash,)]
pub struct HlKey<'a,> {
  theme_name: Option<&'a str,>,
  dark_mode: bool,
  file_ext: &'a str,
}

impl<'a,> HlKey<'a,> {
  pub fn new(
    theme_name: Option<&'a str,>,
    dark_mode: bool,
    file_ext: &'a str,
  ) -> Self {
    Self {
      theme_name,
      dark_mode,
      file_ext,
    }
  }
  pub fn new_hl(&self,) -> HighlightLines {
    let syntax = get_syntax(self.file_ext,);
    let theme = get_theme(self.theme_name, self.dark_mode,);
    HighlightLines::new(syntax, theme,)
  }
}

pub mod layout {
  use eframe::egui;
  use syntect::{highlighting::FontStyle, util::LinesWithEndings};

  use crate::{hl::syntax_set, text};

  use super::HlKey;

  #[derive(Clone, Default,)]
  struct LayoutJobLoader {}

  impl egui::util::cache::ComputerMut<(&HlKey<'_,>, &str,), egui::text::LayoutJob,>
    for LayoutJobLoader
  {
    fn compute(
      &mut self,
      (hl_key, text,): (&HlKey<'_,>, &str,),
    ) -> egui::text::LayoutJob {
      get_layout_job(hl_key, text,)
    }
  }

  type LayoutJobCache = egui::util::cache::FrameCache<egui::text::LayoutJob, LayoutJobLoader,>;

  pub fn get_layout_job_from_cache(
    ctx: &egui::Context,
    hl_key: &HlKey,
    text: &str,
  ) -> egui::text::LayoutJob {
    puffin::profile_function!();
    ctx.memory_mut(|mem| mem.caches.cache::<LayoutJobCache>().get((hl_key, text,),),)
  }

  fn get_layout_job(
    hl_key: &HlKey,
    text: &str,
  ) -> egui::text::LayoutJob {
    puffin::profile_function!();

    let mut hl = hl_key.new_hl();

    let mut job = egui::text::LayoutJob {
      text: text.into(),
      ..Default::default()
    };

    job.wrap.max_width = f32::INFINITY; // no wrap

    for line in LinesWithEndings::from(text,) {
      for (style, range,) in hl.highlight_line(line, syntax_set(),).unwrap() {
        let format = convert_to_text_format(style,);
        //
        job.sections.push(egui::text::LayoutSection {
          leading_space: 0.0,
          byte_range: as_byte_range(text, range,),
          format,
        },);
      }
    }

    job
  }

  /// syntect style => egui TextFormat
  fn convert_to_text_format(style: syntect::highlighting::Style,) -> egui::TextFormat {
    let fg = style.foreground;
    let text_color = egui::Color32::from_rgb(fg.r, fg.g, fg.b,);
    let italics = style.font_style.contains(FontStyle::ITALIC,);
    let underline = style.font_style.contains(FontStyle::UNDERLINE,);
    let underline = if underline {
      egui::Stroke::new(1.0, text_color,)
    } else {
      egui::Stroke::NONE
    };

    egui::TextFormat {
      font_id: text::text_editor_font(),
      color: text_color,
      italics,
      underline,
      ..Default::default()
    }
  }

  fn as_byte_range(
    whole: &str,
    range: &str,
  ) -> std::ops::Range<usize,> {
    let whole_start = whole.as_ptr() as usize;
    let range_start = range.as_ptr() as usize;
    assert!(whole_start <= range_start);
    assert!(range_start + range.len() <= whole_start + whole.len());
    let offset = range_start - whole_start;
    offset..(offset + range.len())
  }
}
