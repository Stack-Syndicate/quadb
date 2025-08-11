use nalgebra::SVector;
use petgraph::{
    graph::{DiGraph, NodeIndex},
    visit::Dfs,
};
use std::{
    collections::HashMap,
    hash::Hash,
    iter::zip,
    ops::{Deref, DerefMut},
};

#[derive(Debug, Clone, Copy)]
pub struct Vector<const D: usize>(SVector<f64, D>);
impl<const D: usize> Deref for Vector<D> {
    type Target = SVector<f64, D>;
    fn deref(&self) -> &SVector<f64, D> {
        &self.0
    }
}
impl<const D: usize> DerefMut for Vector<D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl<const D: usize> PartialEq for Vector<D> {
    fn eq(&self, other: &Self) -> bool {
        zip(self.iter(), other.iter()).all(|(a, b)| a == b)
    }
}
impl<const D: usize> Hash for Vector<D> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for val in self.0.iter() {
            state.write_u64(val.to_bits());
        }
    }
}
impl<const D: usize> Eq for Vector<D> {}
impl<const D: usize> Vector<D> {
    pub fn from_element(value: f64) -> Self {
        Self(SVector::<f64, D>::from_element(value))
    }
}

pub trait QEntity: Sized + Clone {}

#[derive(Debug, Clone)]
pub struct Bounds<const D: usize> {
    min: Vector<D>,
    max: Vector<D>,
}
impl<const D: usize> Bounds<D> {
    fn new() -> Bounds<D> {
        let mut min = Vector::<D>::from_element(f64::MIN);
        let mut max = Vector::<D>::from_element(f64::MIN);
        for d in 0..D {
            min[d] = f64::MIN;
            max[d] = f64::MAX;
        }
        Self { min, max }
    }
    pub fn contains(&self, position: Vector<D>) -> bool {
        let mut within_bounds = true;
        for (p, (min, max)) in zip(
            position.iter(),
            zip(self.min.iter(), self.max.iter()).into_iter(),
        ) {
            if !(p > min && p <= max) {
                within_bounds = false;
                break;
            }
        }
        return within_bounds;
    }
    fn split(&self) -> Vec<Bounds<D>> {
        let mut new_bounds_vec = Vec::new();
        for mask in 0..(1 << D) {
            let mut new_bound = Bounds::<D>::new();
            for d in 0..D {
                let mid = (self.min[d] + self.max[d]) / 2.0;
                if ((mask >> d) & 1) == 0 {
                    new_bound.max[d] = mid;
                } else {
                    new_bound.min[d] = mid;
                }
            }
            new_bounds_vec.push(new_bound);
        }
        return new_bounds_vec;
    }
}

#[derive(Debug)]
pub struct Space<const D: usize, QE: QEntity> {
    pub bounds: Bounds<D>,
    pub entities: HashMap<Vector<D>, QE>,
}
impl<const D: usize, QE: QEntity> Space<D, QE> {
    fn new() -> Space<D, QE> {
        let bounds = Bounds::new();
        let entities = HashMap::new();
        Space::<D, QE> { bounds, entities }
    }
    fn from_bounds(bounds: Bounds<D>) -> Space<D, QE> {
        let entities = HashMap::new();
        Space::<D, QE> { bounds, entities }
    }
}

#[derive(Debug)]
pub struct SpaceTree<const D: usize, QE: QEntity> {
    pub graph: DiGraph<Space<D, QE>, ()>,
}
impl<const D: usize, QE: QEntity> SpaceTree<D, QE> {
    pub fn new() -> SpaceTree<D, QE> {
        let mut graph = DiGraph::new();
        graph.add_node(Space::<D, QE>::new());
        SpaceTree::<D, QE> { graph }
    }
    pub fn insert(&mut self, entity: QE, position: Vec<f64>) {
        let position = vec_to_svector(position).unwrap();
        let root = NodeIndex::new(0);
        let mut dfs = Dfs::new(&self.graph, root);
        let max_entities = usize::pow(2, D.try_into().unwrap());
        // While there exists a next node in the dfs
        while let Some(node_idx) = dfs.next(&self.graph) {
            let contains = {
                let space = self.graph.node_weight(node_idx).unwrap();
                space.bounds.contains(position)
            };
            if contains {
                let space = self.graph.node_weight_mut(node_idx).unwrap();
                let not_full = { space.entities.len() < max_entities };
                if not_full {
                    space.entities.insert(position, entity);
                    return;
                } else {
                    let mut entities: Vec<_> = space.entities.drain().collect();
                    entities.push((position, entity.clone()));
                    let old_bounds = space.bounds.clone();
                    // Split space into new nodes and insert them
                    let new_bounds_vec = old_bounds.split();
                    let mut new_node_idxs = Vec::new();
                    for b in new_bounds_vec {
                        new_node_idxs.push(self.graph.add_node(Space::from_bounds(b)));
                    }
                    for new_idx in new_node_idxs.iter() {
                        self.graph.add_edge(node_idx, *new_idx, ());
                    }
                    for (p, e) in entities.drain(..) {
                        for idx in &new_node_idxs {
                            let new_node = self.graph.node_weight_mut(*idx).unwrap();
                            if new_node.bounds.contains(p) {
                                new_node.entities.insert(p, e.clone());
                                break;
                            }
                        }
                    }
                    return;
                }
            } else {
                continue;
            }
        }
    }
}

fn vec_to_svector<const D: usize>(v: Vec<f64>) -> Option<Vector<D>> {
    if v.len() == D {
        Some(Vector(SVector::<f64, D>::from_column_slice(&v)))
    } else {
        None
    }
}
