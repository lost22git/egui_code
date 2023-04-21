#[derive(Debug, Clone, PartialEq, Eq,)]
pub enum ItemId {
  Explorer,
  Search,
  Extension,
  Setting,
  Custom(String,),
}

fn main() {
  let a = ItemId::Custom("abc".to_string(),);
  let b = ItemId::Custom("ef".to_string(),);
  let c = ItemId::Custom("ef".to_string(),);

  assert!(a != b);
  assert!(b == c);
}
