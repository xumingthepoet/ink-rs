#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FlowLevel {
    Story,
    Knot,
    Stitch,
    WeavePoint,
}
