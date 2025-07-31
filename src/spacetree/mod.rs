use crate::utils::BoxedError;

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


pub struct SpaceTree {
    bounds: Bounds,
    children: Vec<SpaceTree>,
    max_entities_per_node: usize
}
impl SpaceTree {
    pub fn new(dimensions: usize, max_entities_per_node: usize) -> Self {
        SpaceTree {
            bounds: Bounds::new(dimensions),
            children: Vec::new(),
            max_entities_per_node,
        }
    }
}