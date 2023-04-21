use std::time::Duration;

fn main() {
  let bytes = include_bytes!("c:/Users/zzz/Desktop/wallpaper/4gha4c7w7gt91.png");
  let encoding = detect_encoding(bytes,);
  let (str, _, _,) = encoding.decode(bytes,);
  let string: String = str.into();
  println!("{string}");
  loop {
    let _count = string.lines().count();
    std::thread::sleep(Duration::from_millis(50,),);
  }
}

pub fn detect_encoding(text: &[u8],) -> &'static encoding_rs::Encoding {
  let len = text.len();
  let to = std::cmp::min(1000, len,);
  let mut encoding_detector = chardetng::EncodingDetector::new();
  encoding_detector.feed(&text[0..to], to == len,);
  encoding_detector.guess(None, true,)
}
