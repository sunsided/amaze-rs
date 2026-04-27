use crate::hex_coord::HexCoord;
use crate::wall6_grid::Wall6Grid;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HexEdge {
    pub from: HexCoord,
    pub to: HexCoord,
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HexEdgeList {
    pub width: usize,
    pub height: usize,
    pub edges: Vec<HexEdge>,
}

impl HexEdgeList {
    #[inline]
    pub fn iter_edges(&self) -> impl Iterator<Item = &HexEdge> {
        self.edges.iter()
    }
}

impl From<&Wall6Grid> for HexEdgeList {
    fn from(value: &Wall6Grid) -> Self {
        let mut edges = Vec::new();

        for from in value.coords() {
            for to in value.open_neighbors(from) {
                if from < to {
                    edges.push(HexEdge { from, to });
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
pub struct HexPetgraphEdgeRef {
    source: HexCoord,
    target: HexCoord,
    edge_id: usize,
}

#[cfg(feature = "petgraph")]
impl petgraph::visit::EdgeRef for HexPetgraphEdgeRef {
    type NodeId = HexCoord;
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
pub struct HexNodeIdentifiers {
    width: usize,
    next: usize,
    total: usize,
}

#[cfg(feature = "petgraph")]
impl Iterator for HexNodeIdentifiers {
    type Item = HexCoord;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next >= self.total {
            return None;
        }

        let index = self.next;
        self.next += 1;
        Some(HexCoord::new(
            (index % self.width) as isize,
            (index / self.width) as isize,
        ))
    }
}

#[cfg(feature = "petgraph")]
pub struct HexEdgeReferences<'a> {
    iter: std::iter::Enumerate<std::slice::Iter<'a, HexEdge>>,
}

#[cfg(feature = "petgraph")]
impl<'a> Iterator for HexEdgeReferences<'a> {
    type Item = HexPetgraphEdgeRef;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(edge_id, edge)| HexPetgraphEdgeRef {
            source: edge.from,
            target: edge.to,
            edge_id,
        })
    }
}

#[cfg(feature = "petgraph")]
impl petgraph::visit::GraphBase for HexEdgeList {
    type NodeId = HexCoord;
    type EdgeId = usize;
}

#[cfg(feature = "petgraph")]
impl petgraph::visit::Data for HexEdgeList {
    type NodeWeight = ();
    type EdgeWeight = ();
}

#[cfg(feature = "petgraph")]
impl petgraph::visit::GraphProp for HexEdgeList {
    type EdgeType = petgraph::Undirected;
}

#[cfg(feature = "petgraph")]
impl petgraph::visit::NodeCount for HexEdgeList {
    fn node_count(&self) -> usize {
        self.width * self.height
    }
}

#[cfg(feature = "petgraph")]
impl petgraph::visit::EdgeCount for HexEdgeList {
    fn edge_count(&self) -> usize {
        self.edges.len()
    }
}

#[cfg(feature = "petgraph")]
impl petgraph::visit::IntoNodeIdentifiers for &HexEdgeList {
    type NodeIdentifiers = HexNodeIdentifiers;

    fn node_identifiers(self) -> Self::NodeIdentifiers {
        HexNodeIdentifiers {
            width: self.width,
            next: 0,
            total: self.width * self.height,
        }
    }
}

#[cfg(feature = "petgraph")]
impl petgraph::visit::IntoNeighbors for &HexEdgeList {
    type Neighbors = std::vec::IntoIter<HexCoord>;

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
impl<'a> petgraph::visit::IntoEdgeReferences for &'a HexEdgeList {
    type EdgeRef = HexPetgraphEdgeRef;
    type EdgeReferences = HexEdgeReferences<'a>;

    fn edge_references(self) -> Self::EdgeReferences {
        HexEdgeReferences {
            iter: self.edges.iter().enumerate(),
        }
    }
}

#[cfg(feature = "petgraph")]
impl petgraph::visit::IntoEdges for &HexEdgeList {
    type Edges = std::vec::IntoIter<HexPetgraphEdgeRef>;

    fn edges(self, a: Self::NodeId) -> Self::Edges {
        self.edges
            .iter()
            .enumerate()
            .filter_map(|(edge_id, edge)| {
                if edge.from == a {
                    Some(HexPetgraphEdgeRef {
                        source: a,
                        target: edge.to,
                        edge_id,
                    })
                } else if edge.to == a {
                    Some(HexPetgraphEdgeRef {
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
impl petgraph::visit::Visitable for HexEdgeList {
    type Map = std::collections::HashSet<HexCoord>;

    fn visit_map(&self) -> Self::Map {
        std::collections::HashSet::with_capacity(self.width * self.height)
    }

    fn reset_map(&self, map: &mut Self::Map) {
        map.clear();
    }
}

#[cfg(all(test, feature = "generator-hex-recursive-backtracker"))]
mod tests {
    use super::*;
    use crate::generators::RecursiveBacktracker6;
    use std::collections::HashSet;

    fn make_hex_maze() -> Wall6Grid {
        RecursiveBacktracker6::new_from_seed(42).generate(5, 5)
    }

    #[test]
    fn edge_count_matches_door_count() {
        let maze = make_hex_maze();
        let list = HexEdgeList::from(&maze);

        let directed_door_count: usize = maze
            .coords()
            .map(|coord| maze.open_neighbors(coord).count())
            .sum();

        assert_eq!(list.edges.len() * 2, directed_door_count);
    }

    #[test]
    fn edges_are_deduplicated() {
        let maze = make_hex_maze();
        let list = HexEdgeList::from(&maze);

        let mut seen = HashSet::new();
        for edge in &list.edges {
            assert!(edge.from < edge.to);
            assert!(seen.insert((edge.from, edge.to)));
        }
    }

    #[test]
    fn empty_grid_produces_empty_edge_list() {
        let maze = RecursiveBacktracker6::new_from_seed(42).generate(0, 0);
        let list = HexEdgeList::from(&maze);

        assert_eq!(list.width, 0);
        assert_eq!(list.height, 0);
        assert!(list.edges.is_empty());
    }

    #[test]
    fn iter_edges_yields_all() {
        let maze = make_hex_maze();
        let list = HexEdgeList::from(&maze);

        let from_iter: Vec<_> = list.iter_edges().copied().collect();
        assert_eq!(from_iter, list.edges);
    }

    #[test]
    fn width_height_preserved() {
        let maze = make_hex_maze();
        let list = HexEdgeList::from(&maze);

        assert_eq!(list.width, maze.width());
        assert_eq!(list.height, maze.height());
    }
}

#[cfg(all(
    test,
    feature = "petgraph",
    feature = "generator-hex-recursive-backtracker"
))]
mod petgraph_tests {
    use super::*;
    use crate::generators::RecursiveBacktracker6;
    use petgraph::algo::astar;
    use petgraph::visit::{
        Bfs, EdgeCount, EdgeRef, IntoEdgeReferences, IntoEdges, IntoNeighbors, IntoNodeIdentifiers,
        NodeCount, VisitMap, Visitable,
    };
    use std::collections::HashSet;

    fn make_hex_maze() -> Wall6Grid {
        RecursiveBacktracker6::new_from_seed(42).generate(5, 5)
    }

    #[test]
    fn node_count_matches_grid_cells() {
        let maze = make_hex_maze();
        let list = HexEdgeList::from(&maze);
        assert_eq!(list.node_count(), maze.width() * maze.height());
    }

    #[test]
    fn edge_count_matches_edges_len() {
        let maze = make_hex_maze();
        let list = HexEdgeList::from(&maze);
        assert_eq!(list.edge_count(), list.edges.len());
    }

    #[test]
    fn node_identifiers_enumerates_all_cells() {
        let maze = make_hex_maze();
        let list = HexEdgeList::from(&maze);

        let nodes: Vec<_> = (&list).node_identifiers().collect();
        let unique: HashSet<_> = nodes.iter().copied().collect();
        assert_eq!(nodes.len(), maze.width() * maze.height());
        assert_eq!(unique.len(), nodes.len());
    }

    #[test]
    fn neighbors_returns_open_connections() {
        let maze = make_hex_maze();
        let list = HexEdgeList::from(&maze);

        for coord in maze.coords() {
            let expected: HashSet<_> = maze.open_neighbors(coord).collect();
            let actual: HashSet<_> = (&list).neighbors(coord).collect();
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn edges_returns_incident_edges() {
        let maze = make_hex_maze();
        let list = HexEdgeList::from(&maze);

        for coord in maze.coords() {
            let expected: HashSet<_> = maze.open_neighbors(coord).collect();
            let incident: Vec<_> = (&list).edges(coord).collect();
            let actual: HashSet<_> = incident.iter().map(|edge| edge.target()).collect();

            assert_eq!(incident.len(), expected.len());
            assert_eq!(actual, expected);
            assert!(incident.iter().all(|edge| edge.source() == coord));
        }
    }

    #[test]
    fn edge_references_enumerates_all() {
        let maze = make_hex_maze();
        let list = HexEdgeList::from(&maze);

        let refs: Vec<_> = (&list).edge_references().collect();
        assert_eq!(refs.len(), list.edges.len());

        for edge_ref in refs {
            let edge = &list.edges[edge_ref.id()];
            assert_eq!(edge_ref.source(), edge.from);
            assert_eq!(edge_ref.target(), edge.to);
        }
    }

    #[test]
    fn visitable_map_works() {
        let maze = make_hex_maze();
        let list = HexEdgeList::from(&maze);
        let node = maze.coords().next().unwrap_or(HexCoord::new(0, 0));

        let mut map = list.visit_map();
        assert!(map.visit(node));
        assert!(map.is_visited(&node));
        assert!(!map.visit(node));

        list.reset_map(&mut map);
        assert!(!map.is_visited(&node));
    }

    #[test]
    fn bfs_traversal_visits_all_reachable() {
        let maze = make_hex_maze();
        let list = HexEdgeList::from(&maze);
        let start = HexCoord::new(0, 0);

        let mut bfs = Bfs::new(&list, start);
        let mut visited = HashSet::new();
        while let Some(node) = bfs.next(&list) {
            visited.insert(node);
        }

        assert_eq!(visited.len(), maze.width() * maze.height());
    }

    #[test]
    fn astar_finds_path() {
        let maze = make_hex_maze();
        let list = HexEdgeList::from(&maze);
        let start = HexCoord::new(0, 0);
        let goal = HexCoord::new((maze.width() - 1) as isize, (maze.height() - 1) as isize);

        let path = astar(&list, start, |node| node == goal, |_| 1usize, |_| 0usize);
        assert!(path.is_some());

        let (_, nodes) = path.unwrap();
        assert_eq!(nodes.first().copied(), Some(start));
        assert_eq!(nodes.last().copied(), Some(goal));
    }
}
