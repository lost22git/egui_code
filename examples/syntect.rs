use syntect::{highlighting::ThemeSet, parsing::SyntaxSet};

fn main() {
  let ss = SyntaxSet::load_defaults_newlines();
  let ts = ThemeSet::load_defaults();

  println!("=============================================================");
  println!("== SyntaxSet");
  println!("=============================================================");
  for s in ss.syntaxes().iter() {
    let name = &s.name;
    let ext = &s.file_extensions;
    let first_line_match = &s.first_line_match;
    println!("-----------------------------------------");
    println!("name => {name:?}");
    println!("extensions => {ext:?}");
    println!("first_line_match => {first_line_match:?}");
  }

  println!("=============================================================");
  println!("== ThemeSet");
  println!("=============================================================");

  for (name, _theme,) in ts.themes.iter() {
    println!("------------------------------------------");
    println!("name => {name}");
    // let settings = &theme.settings;
    // println!("settings => {settings:?}");
    // let scopes = &theme.scopes;
    // println!("scopes => {scopes:?}");
  }
}

#[cfg(test)]
mod test {
  use syntect::{
    easy::HighlightLines,
    highlighting::{Style, Theme, ThemeSet},
    parsing::SyntaxSet,
    util::LinesWithEndings,
  };

  #[test]
  fn simple_hl() {
    let ss = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();

    let syntax = ss.find_syntax_for_file("../src/app.rs",).unwrap().unwrap();

    let mut hl = HighlightLines::new(syntax, &ts.themes["Solarized (light)"],);
    let text = include_str!("../src/app.rs");
    for line in LinesWithEndings::from(text,) {
      let ranges: Vec<(Style, &str,),> = hl.highlight_line(line, &ss,).unwrap();
      let escaped = syntect::util::as_24_bit_terminal_escaped(&ranges[..], true,);
      print!("{}", escaped);
    }
  }
}
