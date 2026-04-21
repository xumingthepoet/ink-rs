#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FlowLevel {
    Story,
    Knot,
    Stitch,
    WeavePoint,
}

impl FlowLevel {
    pub fn next(self) -> Option<Self> {
        match self {
            Self::Story => Some(Self::Knot),
            Self::Knot => Some(Self::Stitch),
            Self::Stitch => Some(Self::WeavePoint),
            Self::WeavePoint => None,
        }
    }
}
