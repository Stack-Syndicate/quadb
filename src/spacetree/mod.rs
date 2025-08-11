use std::collections::HashMap;
use petgraph::graph::DiGraph;
use nalgebra::SVector;

pub trait Entity: Sized {
    fn jolt(velocity: dyn Asref<Vec<f64>>);
}

struct Bounds<const D: usize> {
    min: SVector<f64, D>,
    max: SVector<f64, D>
}
impl<const D: usize> Bounds<D> {
    fn new() -> Bounds<D> {
        let mut min = SVector::<f64, D>::from_element(f64::MIN);
        let mut max = SVector::<f64, D>::from_element(f64::MAX);
        for d in 0..D {
            min[d] = f64::MIN;
            max[d] = f64::MIN;
        }
        Self {
            min, max
        }
    }
}

struct Space<const D: usize, E: Entity> {
    bounds: Bounds<D>,
    entities: HashMap<SVector<f64, D>, E>
}
impl<const D: usize, E: Entity> Space<D, E> {
    fn new() -> Space::<D, E> {
        let bounds = Bounds::new();
        let entities = HashMap::new();
        Space::<D, E> {
            bounds, entities
        }
    }
}

pub struct SpaceTree<const D: usize, E: Entity> {
    graph: DiGraph<Space<D, E>, usize>,
}
impl<const D: usize, E: Entity> SpaceTree<D, E> {
    pub fn new() -> SpaceTree<D, E> {
        let mut graph = DiGraph::new();
        graph.add_node(Space::<D, E>::new());
        SpaceTree::<D, E> { 
            graph
        }
    }
    pub fn insert(entity: E, position: Vec<f64>) {
                
    }
}