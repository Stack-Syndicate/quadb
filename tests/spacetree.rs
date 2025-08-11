use quadb::prelude::*;
use quadb_macros::QEntity;

#[derive(Debug, Clone, QEntity)]
struct Float(f64);

#[test]
fn multiple_insertion() {
    let mut st = SpaceTree::<2, Float>::new();
    
    let points = vec![
        vec![-1e3, 500.0],
        vec![1e2, -2e2],
        vec![-7.5, 9e2],
        vec![1e3, -1e3],
        vec![123.4, -567.8],
    ];
    
    for (i, pos) in points.into_iter().enumerate() {
        st.insert(Float(i as f64), pos);
    }
    
    for node_idx in st.graph.node_indices() {
        let space = st.graph.node_weight(node_idx).unwrap();
        for e in &space.entities {
            assert_eq!(true, space.bounds.contains(*e.0))
        }
    }
}