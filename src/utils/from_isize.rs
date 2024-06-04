#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FromIsize {
    Positive(usize),
    Negative(usize),
}

impl FromIsize {
    pub fn new(src: isize) -> Self {
        if src.is_positive() {
            Self::Positive(src as usize)
        } else {
            Self::Negative((-src) as usize)
        }
    }

    pub fn add(&self, target: usize) -> usize {
        match self {
            FromIsize::Positive(number) => target + number,
            FromIsize::Negative(number) => target - number,
        }
    }
}
