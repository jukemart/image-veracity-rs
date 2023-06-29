use std::sync::Arc;

use itertools::{EitherOrBoth, Itertools};

use crate::node::id::ID;
use crate::node::{Node, NodesRow};

#[derive(Debug, Eq, PartialEq)]
pub struct Tile {
    id: ID,
    leaves: NodesRow,
}

impl Tile {
    /// Take the updates nodes in the NodesRow and update the Tile leaves
    pub fn merge(&mut self, updates: NodesRow) -> Result<(), String> {
        // Do nothing if there's no update
        if updates.0.is_empty() {
            return Ok(());
        }

        // If the tile's leaves is empty, just take the updates
        if self.leaves.0.is_empty() {
            self.leaves = updates;
            return Ok(());
        }

        // Check the depths
        let got = updates.0.first().unwrap().id.bit_length();
        let want = self.leaves.0.first().unwrap().id.bit_length();
        if got != want {
            Err(format!(
                "Updates are at depth {got} but this tile's depth is {want}"
            ))
        } else if !updates.in_subtree(&self.id) {
            Err("Updates are not entirely in this tile".to_string())
        } else {
            let merged = merge(&self.leaves, &updates)?;
            self.leaves = merged;
            Ok(())
        }
    }
}

/// Merge two sorted NodesRow into a new, sorted, NodesRow, taking updated values
fn merge(nodes: &NodesRow, update: &NodesRow) -> Result<NodesRow, String> {
    let merged = nodes
        .0
        .iter()
        .merge_join_by(update.0.iter(), |orig, upd| orig.id.cmp(&upd.id))
        .map(|which| match which {
            // Take the update
            EitherOrBoth::Both(_, upd) => upd,
            EitherOrBoth::Left(x) | EitherOrBoth::Right(x) => x,
        })
        .cloned()
        .collect();
    NodesRow::try_new(merged)
}

mod test {
    use std::sync::Arc;

    use crate::node::Node;

    use super::*;

    const TEST_IDS: [&[u8; 2]; 6] = [
        b"\xAB\x00",
        b"\xAB\x10",
        b"\xAB\x20",
        b"\xAB\x30",
        b"\xAB\x40",
        b"\xAC\x00", // In another tile
    ];

    fn test_node(index: usize, hash: &str) -> Node {
        let bytes = hash.bytes();
        let mut hash = [0_u8; 32];
        // This expects less than 32 bytes in the hash string
        for (i, byte) in bytes.enumerate() {
            hash[i] = byte;
        }
        Node::new(ID::new_id(TEST_IDS[index], 15), hash)
    }

    fn test_id() -> ID {
        ID::new_id(TEST_IDS[0], 15).prefix(8)
    }

    fn arc_node(nodes: Vec<Node>) -> Vec<Arc<Node>> {
        let mut arc_nodes = Vec::with_capacity(nodes.len());
        for node in nodes {
            arc_nodes.push(Arc::from(node));
        }
        arc_nodes
    }

    macro_rules! tile_merge_tests {
        ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (was, upd, want, want_err) = $value;

                let was_nodes = arc_node(was);
                let update = arc_node(upd);
                let want = arc_node(want);

                let mut was_tile = Tile {
                    id: test_id(),
                    leaves: NodesRow(was_nodes),
                };
                let update = NodesRow::try_new(update).unwrap();
                let want_tile = Tile {
                    id: test_id(),
                    leaves: NodesRow(want),
                };

                match was_tile.merge(update) {
                    Ok(_) => {
                        assert!(want_err.is_empty(), "Expected error {}, but got Ok result", want_err);
                        assert_eq!(was_tile, want_tile, "Merge result mismatch: got {:#?}, want {:#?}", was_tile.leaves, want_tile.leaves);
                    }
                    Err(err) => {
                        assert!(!want_err.is_empty(), "Unexpected error {}", err);
                        assert!(
                            err.contains(want_err),
                            "Expected error to contain substring {}, got {}",
                            want_err,
                            err
                        );
                    }
                }
            }
        )*
        }
    }

    tile_merge_tests! {
        empty_merge_empty: (vec![], vec![], vec![], ""),
        no_updates: (vec![test_node(3, "h")], vec![], vec![test_node(3, "h")], ""),
        add_to_empty: (vec![], vec![test_node(0, "h")], vec![test_node(0, "h")], ""),
        override_one: (
            vec![test_node(0, "old")],
            vec![test_node(0, "new")],
            vec![test_node(0, "new")],
            ""
        ),
        add_multiple: (
            vec![test_node(0, "old0"), test_node(3, "old3")],
            vec![test_node(1, "new1"), test_node(2, "new2"), test_node(4, "new4")],
            vec![test_node(0, "old0"), test_node(1, "new1"), test_node(2, "new2"), test_node(3, "old3"), test_node(4, "new4")],
            ""
        ),
        override_some: (
            vec![test_node(0, "old0"), test_node(1, "old1"), test_node(2, "old2"), test_node(3, "old3")],
            vec![test_node(1, "new1"), test_node(2, "new2")],
            vec![test_node(0, "old0"), test_node(1, "new1"), test_node(2, "new2"), test_node(3, "old3")],
            ""
        ),
        override_and_add: (
            vec![test_node(0, "old0"), test_node(1, "old1"), test_node(3, "old3"), test_node(4, "old4")],
            vec![test_node(1, "new1"), test_node(2, "new2")],
            vec![test_node(0, "old0"), test_node(1, "new1"), test_node(2, "new2"), test_node(3, "old3"), test_node(4, "old4")],
            ""
        ),
        wrong_depth: (vec![test_node(0, "old0")], vec![Node::new(test_id(), [0; 32])], vec![], "Updates are at depth"),
        wrong_tile: (vec![test_node(0, "old")], vec![test_node(5, "new")], vec![], "Updates are not entirely in this tile"),
    }
}
