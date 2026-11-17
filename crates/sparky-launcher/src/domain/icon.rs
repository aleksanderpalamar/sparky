#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Icon {
    Named(String),
    Symbolic(&'static [&'static str]),
}
