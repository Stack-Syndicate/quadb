use bincode::{Decode, Encode};

#[derive(Debug, Clone, Encode, Decode)]
struct Bounds {
    min: Vec<f64>,
    max: Vec<f64>,
}
impl Bounds {
    pub fn new(dimensions: usize) -> Self {
        Bounds {
            min: vec![f64::NEG_INFINITY; dimensions],
            max: vec![f64::INFINITY; dimensions],
        }
    }
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct SpaceTree {
    bounds: Bounds,
    children: Vec<SpaceTree>,
    max_leaf_size: usize
}
impl SpaceTree {
    pub fn new(dimensions: usize, max_leaf_size: usize) -> Self {
        SpaceTree {
            bounds: Bounds::new(dimensions),
            children: Vec::new(),
            max_leaf_size,
        }
    }
}