use std::{iter::zip, marker::PhantomData};

use bincode::{Decode, Encode};

#[derive(Debug, Clone, Encode, Decode)]
struct Bounds {
    min: Vec<f64>,
    max: Vec<f64>,
}
impl Bounds {
    pub fn new(dimensions: usize) -> Self {
        Bounds {
            min: vec![f64::MIN; dimensions],
            max: vec![f64::MAX; dimensions],
        }
    }
    pub fn is_inside(&self, position: impl AsRef<[f64]>) -> bool {
        let position: &[f64] = position.as_ref();
        if position.len() != self.min.len() {
            panic!("Position and bounds dimensions do not match.");
        } else {
            let mut inside = true;
            for ((min, max), coord) in zip(zip(&self.min, &self.max), position) {
                if !(coord <= max && coord >= min) {
                    inside = false;
                    break;
                }
            }
            return inside;
        }
    }
    pub fn bisect(&self) -> Vec<Self> {
        let midpoints: Vec<f64> = zip(&self.min, &self.max)
            .map(|(min, max)| (min + max) / 2.0)
            .collect();

        let mut bounds_list = Vec::new();
        let num_children = 1 << self.min.len(); // 2^n

        for i in 0..num_children {
            let mut min = Vec::with_capacity(self.min.len());
            let mut max = Vec::with_capacity(self.max.len());

            for d in 0..self.min.len() {
                if (i >> d) & 1 == 0 {
                    min.push(self.min[d]);
                    max.push(midpoints[d]);
                } else {
                    min.push(midpoints[d]);
                    max.push(self.max[d]);
                }
            }

            bounds_list.push(Bounds { min, max });
        }

        bounds_list
    }
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct SpaceTree<T: Encode + Decode<T> + Clone> {
    bounds: Bounds,
    children: Vec<SpaceTree<T>>,
    leaves: Vec<(Vec<f64>, T)>,
    entity_type: PhantomData<T>,
    dimensions: usize,
}
impl<T: Clone + Encode + Decode<T>> SpaceTree<T> {
    pub fn new(dimensions: usize) -> Self {
        SpaceTree {
            bounds: Bounds::new(dimensions),
            children: Vec::<SpaceTree<T>>::new(),
            entity_type: PhantomData,
            dimensions,
            leaves: Vec::new(),
        }
    }
    pub fn from_bounds(bounds: Bounds) -> Self {
        SpaceTree {
            bounds: bounds.clone(),
            children: Vec::<SpaceTree<T>>::new(),
            entity_type: PhantomData,
            dimensions: bounds.min.len(),
            leaves: Vec::new(),
        }
    }
    pub fn insert(&mut self, position: impl AsRef<[f64]>, entity: T) {
        let position: Vec<f64> = position.as_ref().to_vec();
        if self.children.is_empty() {
            if self.leaves.len() < 2usize.pow(self.dimensions as u32) {
                self.leaves.push((position, entity));
                return;
            }

            // Subdivide
            let new_bounds_list = self.bounds.bisect();
            println!("{:?}", new_bounds_list);
            let mut new_spacetrees: Vec<SpaceTree<T>> = new_bounds_list
                .iter()
                .map(|bounds| SpaceTree::from_bounds(bounds.clone()))
                .collect();

            for (pos, ent) in self.leaves.drain(..) {
                for st in new_spacetrees.iter_mut() {
                    let in_bounds = pos
                        .iter()
                        .enumerate()
                        .all(|(i, coord)| coord >= &st.bounds.min[i] && coord <= &st.bounds.max[i]);
                    if in_bounds {
                        st.insert(pos.clone(), ent.clone());
                        break;
                    }
                }
            }

            self.children = new_spacetrees;
        }
        for st in self.children.iter_mut() {
            let in_bounds = position
                .iter()
                .enumerate()
                .all(|(i, coord)| coord >= &st.bounds.min[i] && coord < &st.bounds.max[i]);
            if in_bounds {
                st.insert(position, entity);
                return;
            }
        }
        panic!("Position {:?} does not fit in any child bounds", position);
    }
    pub fn get_neighbours(&self, position: impl AsRef<[f64]>) -> Result<&Vec<(Vec<f64>, T)>, &str> {
        let position: &[f64] = position.as_ref();
        if self.children.len() == 0 {
            return Ok(&self.leaves);
        } else {
            for child in &self.children {
                if child.bounds.is_inside(position) {
                    return child.get_neighbours(position);
                }
            }
        }
        return Err("No neighbours found!");
    }
}

#[cfg(test)]
mod tests {
    use bincode::{Decode, Encode};
    use crate::spacetree::SpaceTree;
    use std::fs::File;
    use std::io::{self, BufWriter, Write};
    fn count_entities<T: Encode + Decode<T> + Clone>(tree: &SpaceTree<T>) -> usize {
        if tree.children.is_empty() {
            tree.leaves.len()
        } else {
            tree.children.iter().map(count_entities).sum()
        }
    }
    #[test]
    fn basic_insertion_and_subdivision() {
        let mut tree = SpaceTree::<f32>::new(2);
        for i in 0..10 {
            tree.insert(vec![i as f64, i as f64], i as f32);
        }
        assert!(!tree.children.is_empty(), "Tree should have subdivided");
        let total_entities = count_entities(&tree);
        assert_eq!(total_entities, 10, "All entities should be in the tree");
    }
    #[test]
    fn output_quadtree_structure() -> io::Result<()> {
        let mut tree = SpaceTree::<f32>::new(2);
        for i in -3..3 {
            tree.insert(vec![i as f64, i as f64], i as f32);
        }
        assert!(!tree.children.is_empty(), "Tree should have subdivided");
        let total_entities = count_entities(&tree);
        assert_eq!(total_entities, 6, "All entities should be in the tree");
        println!("{:?}", tree);
        let file = File::create("debug_output.txt")?;
        let mut writer = BufWriter::new(file);
        writeln!(writer, "{:#?}", tree)?;
        writer.flush()?;
        Ok(())
    }
}
