#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FromI32 {
    Positive(u32),
    Negative(u32),
}

impl FromI32 {
    pub fn new(src: i32) -> Self {
        if src.is_positive() {
            Self::Positive(src as u32)
        } else {
            Self::Negative((-src) as u32)
        }
    }
}
