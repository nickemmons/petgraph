use std::marker::{PhantomData, Sized};

use crate::prelude::*;

use crate::graph::Node;
use crate::graph::{Edge, IndexType};
// use crate::borsh_utils::CollectSeqWithLength;
// use crate::borsh_utilMappedSequenceVisitor;
use crate::borsh_utils::{FromDeserialized, IntoSerializable};
use crate::EdgeType;

use super::{EdgeIndex, NodeIndex};
use std::io::{Error, ErrorKind, Read, Write};
use std::collections::HashSet;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use borsh::{BorshDeserialize, BorshSerialize};
use std::convert::TryFrom;
use std::fmt::Debug;
use itertools::assert_equal;
use std::iter::FromIterator;

/// Serialization representation for Graph
/// Keep in sync with deserialization and StableGraph
///
/// The serialization format is as follows, in Pseudorust:
///
/// Graph {
///     nodes: [N],
///     node_holes: [NodeIndex<Ix>],
///     edge_property: EdgeProperty,
///     edges: [Option<(NodeIndex<Ix>, NodeIndex<Ix>, E)>]
/// }
///
/// The same format is used by both Graph and StableGraph.
///
/// For graph there are restrictions:
/// node_holes is always empty and edges are always Some
///
/// A stable graph serialization that obeys these restrictions
/// (effectively, it has no interior vacancies) can de deserialized
/// as a graph.
///
/// Node indices are serialized as integers and are fixed size for
/// binary formats, so the Ix parameter matters there.
// #[derive(BorshSerialize)]
// pub struct BorshSerGraph<'a, N, E, Ix> 
// {
//     nodes: &'a [Node<N, Ix>],
//     node_holes: &'a [NodeIndex<Ix>],
//     edge_property: EdgeProperty,
//     edges: &'a [Edge<E, Ix>],
// }

#[derive(BorshSerialize)]
pub struct BorshSerGraph<N, E, Ix> 
{
    nodes: Vec<Node<N, Ix>>,
    node_holes: Vec<NodeIndex<Ix>>,
    edge_property: EdgeProperty,
    edges: Vec<Edge<E, Ix>>,
}

// Deserialization representation for Graph
// Keep in sync with serialization and StableGraph
#[derive(BorshDeserialize)]
pub struct BorshDeserGraph<N, E, Ix> {
    nodes: Vec<Node<N, Ix>>,
    #[allow(unused)]
    node_holes: Vec<NodeIndex<Ix>>,
    edge_property: EdgeProperty,
    edges: Vec<Edge<E, Ix>>,
}

impl<Ix> BorshSerialize for NodeIndex<Ix>
where
    Ix: IndexType + BorshSerialize,
{
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        borsh::ser::BorshSerialize::serialize(&self.0, writer)?;
        Ok(())
    }
}

impl<Ix> BorshDeserialize for NodeIndex<Ix>
where
    Ix: IndexType + BorshDeserialize,
{
    fn deserialize<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let index = NodeIndex::try_from(<Ix as BorshDeserialize>::deserialize(reader)?)
            .map_err(|err| Error::new(ErrorKind::InvalidData, err.to_string()))?;
        Ok(index)
    }
}

impl<Ix> BorshSerialize for EdgeIndex<Ix>
where
    Ix: IndexType + BorshSerialize,
{
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        borsh::ser::BorshSerialize::serialize(&self.0, writer)?;
        Ok(())
    }
}

impl<Ix> BorshDeserialize for EdgeIndex<Ix>
where
    Ix: IndexType + BorshDeserialize,
{
    fn deserialize<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let index = EdgeIndex::try_from(<Ix as BorshDeserialize>::deserialize(reader)?)
            .map_err(|err| Error::new(ErrorKind::InvalidData, err.to_string()))?;
        
        Ok(index)
    }
}

#[derive(BorshSerialize, BorshDeserialize)]
pub enum EdgeProperty {
    Undirected,
    Directed,
}

impl EdgeProperty {
    pub fn is_directed(&self) -> bool {
        match *self {
            EdgeProperty::Directed => true,
            EdgeProperty::Undirected => false,
        }
    }
}

impl<Ty> From<PhantomData<Ty>> for EdgeProperty
where
    Ty: EdgeType,
{
    fn from(_: PhantomData<Ty>) -> Self {
        if Ty::is_directed() {
            EdgeProperty::Directed
        } else {
            EdgeProperty::Undirected
        }
    }
}

impl<Ty> FromDeserialized for PhantomData<Ty>
where
    Ty: EdgeType,
{
    type Input = EdgeProperty;
    fn from_deserialized(input: Self::Input) -> Result<Self, Error> {
        if input.is_directed() != Ty::is_directed() {
            Err(Error::new(ErrorKind::Other, "graph edge property mismatch"))
        } else {
            Ok(PhantomData)
        }
    }
}

impl<E, Ix> BorshSerialize for Edge<E, Ix>
where
    Ix: IndexType + BorshSerialize,
    E: BorshSerialize + Clone,
{
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        borsh::ser::BorshSerialize::serialize(&Some((self.source(), self.target(), self.weight.clone())), writer)?;
        Ok(())
    }
}

impl<E, Ix> BorshDeserialize for Edge<E, Ix>
where
    Ix: IndexType + BorshDeserialize,
    E: BorshDeserialize + Clone,
{
    fn deserialize<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let edge = <Option::<(NodeIndex::<Ix>, NodeIndex::<Ix>, E)> as BorshDeserialize>::deserialize(reader)?;
        match edge {
            Some((source, target, weight)) => {
                Ok(Edge {
                    weight: weight,
                    node: [source, target],
                    next: [EdgeIndex::end(); 2],
                })
            },
            None => Err(Error::new(ErrorKind::Other, "Graph can not have holes in the edge set, found None, expected edge"))
        }
    }
}

impl<N, Ix> BorshSerialize for Node<N, Ix>
where
    Ix: IndexType + BorshSerialize,
    N: BorshSerialize + Clone,
{
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        borsh::ser::BorshSerialize::serialize(&self.weight, writer)?;
        Ok(())
    }
}

impl<N, Ix> BorshDeserialize for Node<N, Ix>
where
    Ix: IndexType + BorshDeserialize,
    N: BorshDeserialize + Clone,
{
    fn deserialize<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let node_weight = <N as BorshDeserialize>::deserialize(reader)?;
        Ok(Node {
            weight: node_weight,
            next: [EdgeIndex::end(); 2],
        })
    }
}

impl<N, E, Ty, Ix> IntoSerializable for Graph<N, E, Ty, Ix>
where
    Ix: IndexType + BorshSerialize,
    Ty: EdgeType,
    N: BorshSerialize + Clone,
    E: BorshSerialize + Clone,
{
    type Output = BorshSerGraph<N, E, Ix>;
    fn into_serializable(&self) -> Self::Output {
        BorshSerGraph {
            nodes: self.nodes.clone(),
            node_holes: Vec::new(),
            edges: self.edges.clone(),
            edge_property: EdgeProperty::from(PhantomData::<Ty>),
        }
    }
}

impl<N, E, Ty, Ix> BorshSerialize for Graph<N, E, Ty, Ix>
where
    Ty: EdgeType,
    Ix: IndexType + BorshSerialize,
    N: BorshSerialize + Clone,
    E: BorshSerialize + Clone,
{
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.into_serializable().serialize(writer)?;
        Ok(())
    }
}

impl<'a, N, E, Ty, Ix> FromDeserialized for Graph<N, E, Ty, Ix>
where
    Ix: IndexType,
    Ty: EdgeType,
{
    type Input = BorshDeserGraph<N, E, Ix>;
    fn from_deserialized(input: Self::Input) -> Result<Self, Error> {
        let ty = PhantomData::<Ty>::from_deserialized(input.edge_property)?;
        let nodes = input.nodes;
        let edges = input.edges;
        if nodes.len() >= <Ix as IndexType>::max().index() {
            Err(Error::new(ErrorKind::Other, "invalid size"))?
        }

        if edges.len() >= <Ix as IndexType>::max().index() {
            Err(Error::new(ErrorKind::Other, "invalid size"))?
        }

        let mut gr = Graph {
            nodes: nodes,
            edges: edges,
            ty: ty,
        };
        let nc = gr.node_count();
        gr.link_edges()
            .map_err(|i| Error::new(ErrorKind::Other, "invalid node"));
        Ok(gr)
    }
}

impl<N, E, Ty, Ix> BorshDeserialize for Graph<N, E, Ty, Ix>
where
    Ty: EdgeType,
    Ix: IndexType + BorshDeserialize,
    N: BorshDeserialize + Clone,
    E: BorshDeserialize + Clone,
{
    fn deserialize<R: Read>(reader: &mut R) -> Result<Self, Error> {
        Self::from_deserialized(BorshDeserGraph::deserialize(reader)?)
    }
}