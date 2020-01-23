extern crate petgraph;
#[macro_use]
extern crate quickcheck;
extern crate bincode;
extern crate itertools;
extern crate serde_json;
#[macro_use]
extern crate defmac;

extern crate borsh;

use std::collections::HashSet;
use std::fmt::Debug;
use std::iter::FromIterator;

use itertools::assert_equal;
use itertools::{repeat_n, Itertools};

use petgraph::graph::{edge_index, node_index, IndexType, Node, Edge};
use petgraph::prelude::*;
use petgraph::visit::EdgeRef;
use petgraph::visit::IntoEdgeReferences;
use petgraph::visit::NodeIndexable;
use petgraph::EdgeType;

use borsh::{BorshSerialize, BorshDeserialize};

// graphs are the equal, down to graph indices
// this is a strict notion of graph equivalence:
//
// * Requires equal node and edge indices, equal weights
// * Does not require: edge for node order
fn assert_graph_eq<N, N2, E, Ty, Ix>(g: &Graph<N, E, Ty, Ix>, h: &Graph<N2, E, Ty, Ix>)
where
    N: PartialEq<N2> + Debug,
    N2: PartialEq<N2> + Debug,
    E: PartialEq + Debug,
    Ty: EdgeType,
    Ix: IndexType,
{
    assert_eq!(g.node_count(), h.node_count());
    assert_eq!(g.edge_count(), h.edge_count());

    // same node weigths
    assert_equal(
        g.raw_nodes().iter().map(|n| &n.weight),
        h.raw_nodes().iter().map(|n| &n.weight),
    );

    // same edge weigths
    assert_equal(
        g.raw_edges().iter().map(|n| &n.weight),
        h.raw_edges().iter().map(|n| &n.weight),
    );

    for e1 in g.edge_references() {
        let (a2, b2) = h.edge_endpoints(e1.id()).unwrap();
        assert_eq!(e1.source(), a2);
        assert_eq!(e1.target(), b2);
    }

    for index in g.node_indices() {
        let outgoing1 = <HashSet<_>>::from_iter(g.neighbors(index));
        let outgoing2 = <HashSet<_>>::from_iter(h.neighbors(index));
        assert_eq!(outgoing1, outgoing2);
        let incoming1 = <HashSet<_>>::from_iter(g.neighbors_directed(index, Incoming));
        let incoming2 = <HashSet<_>>::from_iter(h.neighbors_directed(index, Incoming));
        assert_eq!(incoming1, incoming2);
    }
}

fn make_graph<Ty, Ix>() -> Graph<String, i32, Ty, Ix>
where
    Ty: EdgeType,
    Ix: IndexType,
{
    let mut g = Graph::default();
    let a = g.add_node("A".to_string());
    let b = g.add_node("B".to_string());
    let c = g.add_node("C".to_string());
    let d = g.add_node("D".to_string());
    let e = g.add_node("E".to_string());
    let f = g.add_node("F".to_string());
    g.extend_with_edges(&[
        (a, b, 7),
        (c, a, 9),
        (a, d, 14),
        (b, c, 10),
        (d, c, 2),
        (d, e, 9),
        (b, f, 15),
        (c, f, 11),
        (e, f, 6),
    ]);
    // Remove a node to make the structure a bit more interesting
    g.remove_node(d);
    g
}

// borsh macros
defmac!(encode ref g => g.try_to_vec().unwrap());
defmac!(decode ref mut data => BorshDeserialize::try_from_slice(data).unwrap());
defmac!(recode ref g => decode!(encode!(g)));

#[test]
fn borsh_edge_index_serialization() {
    let e1 = EdgeIndex::<u32>::new(7);
    let encoded_e1 = <EdgeIndex as borsh::BorshSerialize>::try_to_vec(&e1).unwrap();
    let decoded_e1 = <EdgeIndex as borsh::BorshDeserialize>::try_from_slice(&encoded_e1).unwrap();

    assert_eq!(&e1, &decoded_e1);
}

#[test]
fn borsh_node_index_serialization() {
    let e1 = NodeIndex::<u32>::new(7);
    let encoded_e1 = <NodeIndex as borsh::BorshSerialize>::try_to_vec(&e1).unwrap();
    let decoded_e1 = <NodeIndex as borsh::BorshDeserialize>::try_from_slice(&encoded_e1).unwrap();

    assert_eq!(&e1, &decoded_e1);
}

#[test]
fn borsh_node_serialization() {
    let mut g1 = Graph::<String, u32>::new();
    let x = "a node".to_string();
    g1.add_node(x);
    let n1 = &g1.raw_nodes()[0];
    let encoded_n1 = <Node<String> as borsh::BorshSerialize>::try_to_vec(&n1).unwrap();
    let decoded_n1 = <Node<String> as borsh::BorshDeserialize>::try_from_slice(&encoded_n1).unwrap();

    assert_eq!(&n1.weight, &decoded_n1.weight);
}

#[test]
fn borsh_edge_serialization() {
    let mut g1 = Graph::<String, u32>::new();
    let x = "a node".to_string();
    let y = "another node".to_string();
    let x_index = g1.add_node(x);
    let y_index = g1.add_node(y);
    g1.add_edge(x_index, y_index, 4);
    let e1 = &g1.raw_edges()[0];
    let encoded_e1 = <Edge<u32> as borsh::BorshSerialize>::try_to_vec(&e1).unwrap();
    let decoded_e1 = <Edge<u32> as borsh::BorshDeserialize>::try_from_slice(&encoded_e1).unwrap();

    assert_eq!(&e1.weight, &decoded_e1.weight);
}

#[test]
fn borsh_encode_graph_u32() {
    let mut g1 = Graph::<u32, u32>::new();
    let x = 1729;
    g1.add_node(x);
    let encoded_g1 = <petgraph::Graph<u32, u32> as borsh::BorshSerialize>::try_to_vec(&g1).unwrap();

    assert!(true);
}

#[test]
fn borsh_graph_to_graph_u32_1() {
    let mut g1 = Graph::<u32, u32>::new();
    let x = 1729;
    g1.add_node(x);
    let encoded_g1 = <petgraph::Graph<u32, u32> as borsh::BorshSerialize>::try_to_vec(&g1).unwrap();
    let decoded_g1 = <petgraph::Graph<u32, u32> as borsh::BorshDeserialize>::try_from_slice(&encoded_g1).unwrap();
    let g2: Graph<u32, u32> = recode!(g1);

    assert_graph_eq(&g1, &g2);
}

#[test]
fn borsh_graph_added2_removed2() {
    let mut g1 = Graph::<i32, i32>::new();
    let x = 1729;
    let a = g1.add_node(x);
    let b = g1.add_node(x + 1);
    g1.remove_node(a);
    g1.remove_node(b);
    let g2: Graph<i32, i32> = recode!(g1);

    assert_graph_eq(&g1, &g2);
}

#[test]
fn test_graph_borsh_serialization() {
    let graph: Graph<String, i32> = make_graph();
    let decoded_graph: Graph<String, i32> = recode!(graph);
    assert_graph_eq(&graph, &decoded_graph);
}