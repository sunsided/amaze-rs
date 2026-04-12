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
impl petgraph::visit::IntoNodeIdentifiers for &EdgeList {
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
impl petgraph::visit::IntoNeighbors for &EdgeList {
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
impl petgraph::visit::IntoEdges for &EdgeList {
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
    use std::collections::HashSet;

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

    #[test]
    fn edges_are_deduplicated() {
        let maze = RecursiveBacktracker4::new_from_seed(7).generate(8, 8);
        let list = EdgeList::from(&maze);

        let mut seen = HashSet::new();
        for edge in &list.edges {
            assert!(edge.from < edge.to);
            assert!(seen.insert((edge.from, edge.to)));
        }
    }

    #[test]
    fn edges_connect_adjacent_cells_only() {
        let maze = RecursiveBacktracker4::new_from_seed(7).generate(8, 8);
        let list = EdgeList::from(&maze);

        for edge in &list.edges {
            let dx = edge.from.x.abs_diff(edge.to.x);
            let dy = edge.from.y.abs_diff(edge.to.y);
            assert_eq!(dx + dy, 1);
        }
    }

    #[test]
    fn empty_grid_produces_empty_edge_list() {
        let maze = RecursiveBacktracker4::new_from_seed(7).generate(0, 0);
        let list = EdgeList::from(&maze);

        assert_eq!(list.width, 0);
        assert_eq!(list.height, 0);
        assert!(list.edges.is_empty());
    }

    #[test]
    fn iter_edges_yields_all() {
        let maze = RecursiveBacktracker4::new_from_seed(7).generate(8, 8);
        let list = EdgeList::from(&maze);

        let from_iter: Vec<_> = list.iter_edges().copied().collect();
        assert_eq!(from_iter, list.edges);
    }

    #[test]
    fn width_height_preserved() {
        let maze = RecursiveBacktracker4::new_from_seed(7).generate(8, 8);
        let list = EdgeList::from(&maze);

        assert_eq!(list.width, maze.width());
        assert_eq!(list.height, maze.height());
    }
}

#[cfg(all(test, feature = "petgraph"))]
mod petgraph_tests {
    use super::*;
    use crate::generators::RecursiveBacktracker4;
    use petgraph::algo::astar;
    use petgraph::visit::{
        Bfs, EdgeCount, EdgeRef, IntoEdgeReferences, IntoEdges, IntoNeighbors, IntoNodeIdentifiers,
        NodeCount, VisitMap, Visitable,
    };
    use std::collections::HashSet;

    #[test]
    fn node_count_matches_grid_cells() {
        let maze = RecursiveBacktracker4::new_from_seed(7).generate(8, 8);
        let list = EdgeList::from(&maze);

        assert_eq!(list.node_count(), maze.width() * maze.height());
    }

    #[test]
    fn edge_count_matches_edges_len() {
        let maze = RecursiveBacktracker4::new_from_seed(7).generate(8, 8);
        let list = EdgeList::from(&maze);

        assert_eq!(list.edge_count(), list.edges.len());
    }

    #[test]
    fn node_identifiers_enumerates_all_cells() {
        let maze = RecursiveBacktracker4::new_from_seed(7).generate(8, 8);
        let list = EdgeList::from(&maze);

        let nodes: Vec<_> = (&list).node_identifiers().collect();
        let unique: HashSet<_> = nodes.iter().copied().collect();
        assert_eq!(nodes.len(), maze.width() * maze.height());
        assert_eq!(unique.len(), nodes.len());
    }

    #[test]
    fn neighbors_returns_open_connections() {
        let maze = RecursiveBacktracker4::new_from_seed(7).generate(8, 8);
        let list = EdgeList::from(&maze);

        for coord in maze.coords() {
            let expected: HashSet<_> = maze.open_neighbors(coord).into_iter().collect();
            let actual: HashSet<_> = (&list).neighbors(coord).collect();
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn edges_returns_incident_edges() {
        let maze = RecursiveBacktracker4::new_from_seed(7).generate(8, 8);
        let list = EdgeList::from(&maze);

        for coord in maze.coords() {
            let expected: HashSet<_> = maze.open_neighbors(coord).into_iter().collect();
            let incident: Vec<_> = (&list).edges(coord).collect();
            let actual: HashSet<_> = incident.iter().map(|edge| edge.target()).collect();

            assert_eq!(incident.len(), expected.len());
            assert_eq!(actual, expected);
            assert!(incident.iter().all(|edge| edge.source() == coord));
        }
    }

    #[test]
    fn edge_references_enumerates_all() {
        let maze = RecursiveBacktracker4::new_from_seed(7).generate(8, 8);
        let list = EdgeList::from(&maze);

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
        let maze = RecursiveBacktracker4::new_from_seed(7).generate(8, 8);
        let list = EdgeList::from(&maze);

        let mut map = list.visit_map();
        let node = maze.coords().next().unwrap_or(GridCoord2D::new(0, 0));
        assert!(map.visit(node));
        assert!(map.is_visited(&node));
        assert!(!map.visit(node));

        list.reset_map(&mut map);
        assert!(!map.is_visited(&node));
    }

    #[test]
    fn bfs_traversal_visits_all_reachable() {
        let maze = RecursiveBacktracker4::new_from_seed(7).generate(8, 8);
        let list = EdgeList::from(&maze);
        let start = GridCoord2D::new(0, 0);

        let mut bfs = Bfs::new(&list, start);
        let mut visited = HashSet::new();
        while let Some(node) = bfs.next(&list) {
            visited.insert(node);
        }

        assert_eq!(visited.len(), maze.width() * maze.height());
    }

    #[test]
    fn astar_finds_path() {
        let maze = RecursiveBacktracker4::new_from_seed(7).generate(8, 8);
        let list = EdgeList::from(&maze);
        let start = GridCoord2D::new(0, 0);
        let goal = GridCoord2D::new(maze.width() - 1, maze.height() - 1);

        let path = astar(&list, start, |node| node == goal, |_| 1usize, |_| 0usize);
        assert!(path.is_some());

        let (_, nodes) = path.unwrap();
        assert_eq!(nodes.first().copied(), Some(start));
        assert_eq!(nodes.last().copied(), Some(goal));
    }
}
