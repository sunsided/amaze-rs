use crate::grid_coord_2d::GridCoord2D;
use crate::wall4_grid::Wall4Grid;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Edge {
    pub from: GridCoord2D,
    pub to: GridCoord2D,
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EdgeList {
    pub width: usize,
    pub height: usize,
    pub edges: Vec<Edge>,
}

impl EdgeList {
    #[inline]
    pub fn iter_edges(&self) -> impl Iterator<Item = &Edge> {
        self.edges.iter()
    }
}

impl From<&Wall4Grid> for EdgeList {
    fn from(value: &Wall4Grid) -> Self {
        let mut edges = Vec::new();

        for from in value.coords() {
            for to in value.open_neighbors(from) {
                if from < to {
                    edges.push(Edge { from, to });
                }
            }
        }

        Self {
            width: value.width(),
            height: value.height(),
            edges,
        }
    }
}

#[cfg(feature = "petgraph")]
#[derive(Clone, Copy)]
pub struct PetgraphEdgeRef {
    source: GridCoord2D,
    target: GridCoord2D,
    edge_id: usize,
}

#[cfg(feature = "petgraph")]
impl petgraph::visit::EdgeRef for PetgraphEdgeRef {
    type NodeId = GridCoord2D;
    type EdgeId = usize;
    type Weight = ();

    fn source(&self) -> Self::NodeId {
        self.source
    }

    fn target(&self) -> Self::NodeId {
        self.target
    }

    fn weight(&self) -> &Self::Weight {
        static UNIT: () = ();
        &UNIT
    }

    fn id(&self) -> Self::EdgeId {
        self.edge_id
    }
}

#[cfg(feature = "petgraph")]
#[derive(Clone)]
pub struct NodeIdentifiers {
    width: usize,
    next: usize,
    total: usize,
}

#[cfg(feature = "petgraph")]
impl Iterator for NodeIdentifiers {
    type Item = GridCoord2D;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next >= self.total {
            return None;
        }

        let index = self.next;
        self.next += 1;
        Some(GridCoord2D::new(index % self.width, index / self.width))
    }
}

#[cfg(feature = "petgraph")]
pub struct EdgeReferences<'a> {
    iter: std::iter::Enumerate<std::slice::Iter<'a, Edge>>,
}

#[cfg(feature = "petgraph")]
impl<'a> Iterator for EdgeReferences<'a> {
    type Item = PetgraphEdgeRef;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(edge_id, edge)| PetgraphEdgeRef {
            source: edge.from,
            target: edge.to,
            edge_id,
        })
    }
}

#[cfg(feature = "petgraph")]
impl petgraph::visit::GraphBase for EdgeList {
    type NodeId = GridCoord2D;
    type EdgeId = usize;
}

#[cfg(feature = "petgraph")]
impl petgraph::visit::Data for EdgeList {
    type NodeWeight = ();
    type EdgeWeight = ();
}

#[cfg(feature = "petgraph")]
impl petgraph::visit::GraphProp for EdgeList {
    type EdgeType = petgraph::Undirected;
}

#[cfg(feature = "petgraph")]
impl petgraph::visit::NodeCount for EdgeList {
    fn node_count(&self) -> usize {
        self.width * self.height
    }
}

#[cfg(feature = "petgraph")]
impl petgraph::visit::EdgeCount for EdgeList {
    fn edge_count(&self) -> usize {
        self.edges.len()
    }
}

#[cfg(feature = "petgraph")]
impl<'a> petgraph::visit::IntoNodeIdentifiers for &'a EdgeList {
    type NodeIdentifiers = NodeIdentifiers;

    fn node_identifiers(self) -> Self::NodeIdentifiers {
        NodeIdentifiers {
            width: self.width,
            next: 0,
            total: self.width * self.height,
        }
    }
}

#[cfg(feature = "petgraph")]
impl<'a> petgraph::visit::IntoNeighbors for &'a EdgeList {
    type Neighbors = std::vec::IntoIter<GridCoord2D>;

    fn neighbors(self, a: Self::NodeId) -> Self::Neighbors {
        self.edges
            .iter()
            .filter_map(|edge| {
                if edge.from == a {
                    Some(edge.to)
                } else if edge.to == a {
                    Some(edge.from)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .into_iter()
    }
}

#[cfg(feature = "petgraph")]
impl<'a> petgraph::visit::IntoEdgeReferences for &'a EdgeList {
    type EdgeRef = PetgraphEdgeRef;
    type EdgeReferences = EdgeReferences<'a>;

    fn edge_references(self) -> Self::EdgeReferences {
        EdgeReferences {
            iter: self.edges.iter().enumerate(),
        }
    }
}

#[cfg(feature = "petgraph")]
impl<'a> petgraph::visit::IntoEdges for &'a EdgeList {
    type Edges = std::vec::IntoIter<PetgraphEdgeRef>;

    fn edges(self, a: Self::NodeId) -> Self::Edges {
        self.edges
            .iter()
            .enumerate()
            .filter_map(|(edge_id, edge)| {
                if edge.from == a {
                    Some(PetgraphEdgeRef {
                        source: a,
                        target: edge.to,
                        edge_id,
                    })
                } else if edge.to == a {
                    Some(PetgraphEdgeRef {
                        source: a,
                        target: edge.from,
                        edge_id,
                    })
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .into_iter()
    }
}

#[cfg(feature = "petgraph")]
impl petgraph::visit::Visitable for EdgeList {
    type Map = std::collections::HashSet<GridCoord2D>;

    fn visit_map(&self) -> Self::Map {
        std::collections::HashSet::with_capacity(self.width * self.height)
    }

    fn reset_map(&self, map: &mut Self::Map) {
        map.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generators::RecursiveBacktracker4;

    #[test]
    fn edge_count_matches_door_count() {
        let maze = RecursiveBacktracker4::new_from_seed(7).generate(8, 8);
        let list = EdgeList::from(&maze);

        let directed_door_count: usize = maze
            .coords()
            .map(|coord| maze.open_neighbors(coord).len())
            .sum();

        assert_eq!(list.edges.len() * 2, directed_door_count);
    }
}
