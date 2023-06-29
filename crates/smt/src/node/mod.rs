use std::cmp::Ordering;
use std::sync::Arc;

use crate::node::id::ID;

pub(crate) mod id;

#[derive(Debug, Default, Eq, PartialEq)]
pub struct Node {
    pub(crate) id: ID,
    // Using fixed-size hash value instead of generic type or GAT
    hash: Arc<[u8; 32]>,
}

impl Node {
    pub fn new_from_id(id: ID) -> Self {
        Node {
            id,
            ..Node::default()
        }
    }
    pub fn new(id: ID, hash: [u8; 32]) -> Self {
        Node {
            id,
            hash: Arc::from(hash),
        }
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.id.partial_cmp(&other.id)
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct NodesRow(pub Vec<Arc<Node>>);

impl NodesRow {
    pub fn try_new(mut nodes: Vec<Arc<Node>>) -> Result<Self, String> {
        if nodes.is_empty() {
            Ok(NodesRow(nodes))
        } else {
            let depth = &nodes.first().unwrap().id.bit_length();
            // we can unwrap immediately after checking the empty case
            prepare(&mut nodes, *depth)?;
            Ok(NodesRow(nodes))
        }
    }

    /// in_subtree returns whether all nodes in this row are strictly under the node with the given ID
    pub fn in_subtree(&self, root: &ID) -> bool {
        let root_length = root.bit_length();
        if self
            .0
            .first()
            .is_some_and(|n| n.id.bit_length() <= root_length)
        {
            return false;
        }
        if self
            .0
            .first()
            .is_some_and(|n| n.id.prefix(root_length) != *root)
        {
            return false;
        }
        // It's enough to only check first and last because the list is sorted
        self.0.len() == 1
            || self
                .0
                .last()
                .is_some_and(|n| n.id.prefix(root_length) == *root)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

/// mutably filters, sorts, and de-dupes in preparation for HStar3 algorithm
pub fn prepare(nodes: &mut Vec<Arc<Node>>, depth: usize) -> Result<(), String> {
    for (index, node) in nodes.iter().enumerate() {
        if node.id.bit_length() != depth {
            return Err(format!(
                "node {} invalid depth {}, want {}",
                index,
                node.id.bit_length(),
                depth
            ));
        }
    }
    nodes.sort_unstable();
    nodes.dedup();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_node_row() {
        let test_cases = vec![
            // empty
            (vec![], NodesRow(vec![]), false, "no error"),
            // sorted
            (
                vec![
                    Node::new_from_id(ID::new_id(b"\x00\x01", 16)),
                    Node::new_from_id(ID::new_id(b"\x00\x00", 16)),
                    Node::new_from_id(ID::new_id(b"\x00\x02", 16)),
                ],
                NodesRow::try_new(vec![
                    Arc::from(Node::new_from_id(ID::new_id(b"\x00\x00", 16))),
                    Arc::from(Node::new_from_id(ID::new_id(b"\x00\x01", 16))),
                    Arc::from(Node::new_from_id(ID::new_id(b"\x00\x02", 16))),
                ])
                .expect("creating NodesRow"),
                false,
                "no error",
            ),
            // depth error
            (
                vec![
                    Node::new_from_id(ID::new_id(b"\x00\x00", 16)),
                    Node::new_from_id(ID::new_id(b"\x00\x00\x01", 24)),
                ],
                NodesRow(vec![]),
                true,
                "invalid depth",
            ),
            // dupe ID
            (
                vec![
                    Node::new_from_id(ID::new_id(b"\x00\x01", 16)),
                    Node::new_from_id(ID::new_id(b"\x00\x01", 16)),
                ],
                NodesRow::try_new(vec![Arc::from(Node::new_from_id(ID::new_id(
                    b"\x00\x01",
                    16,
                )))])
                .expect("creating NodesRow"),
                false,
                "no error",
            ),
        ];

        for (nodes, want, want_error, want_err_str) in test_cases {
            let mut arc_nodes = Vec::new();
            for node in nodes {
                arc_nodes.push(Arc::from(node));
            }
            match NodesRow::try_new(arc_nodes) {
                Ok(got) => {
                    assert_eq!(
                        got, want,
                        "NodesRow::try_new got {:?}, want {:?}",
                        got, want
                    )
                }
                Err(got_err_str) => {
                    assert!(
                        want_error,
                        "NodesRow::try_new got error {}, want error {}",
                        got_err_str, want_error
                    );
                    assert!(
                        got_err_str.contains(want_err_str),
                        "NodesRow::try_new got error {}, wanted substring {}",
                        got_err_str,
                        want_err_str
                    );
                }
            }
        }
    }

    #[test]
    fn node_row_prepare<'a>() {
        const TEST_BYTES_1: &[u8; 32] = &[
            0_u8, 1_u8, 2_u8, 3_u8, 4_u8, 5_u8, 6_u8, 7_u8, 8_u8, 9_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 0_u8, 1_u8,
        ];

        const TEST_BYTES_2: &[u8; 32] = &[
            0_u8, 1_u8, 2_u8, 3_u8, 4_u8, 5_u8, 6_u8, 7_u8, 8_u8, 9_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 0_u8, 2_u8,
        ];

        const TEST_BYTES_3: &[u8; 32] = &[
            0_u8, 1_u8, 2_u8, 3_u8, 4_u8, 5_u8, 6_u8, 7_u8, 8_u8, 9_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 0_u8, 3_u8,
        ];

        const TEST_BYTES_4: &[u8; 32] = &[
            0_u8, 1_u8, 2_u8, 3_u8, 4_u8, 5_u8, 6_u8, 7_u8, 8_u8, 9_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 1_u8, 1_u8, 1_u8,
            1_u8, 1_u8, 1_u8, 1_u8,
        ];

        let test_cases = vec![
            (
                "depth-err",
                vec![Node::new_from_id(ID::new_id(TEST_BYTES_1, 256).prefix(10))],
                vec![],
                "invalid depth",
            ),
            (
                "dupe-1",
                vec![
                    Node::new_from_id(ID::new_id(TEST_BYTES_1, 256)),
                    Node::new_from_id(ID::new_id(TEST_BYTES_1, 256)),
                ],
                vec![Node::new_from_id(ID::new_id(TEST_BYTES_1, 256))],
                "",
            ),
            (
                "dupe-2",
                vec![
                    Node::new_from_id(ID::new_id(TEST_BYTES_1, 256)),
                    Node::new_from_id(ID::new_id(TEST_BYTES_2, 256)),
                    Node::new_from_id(ID::new_id(TEST_BYTES_2, 256)),
                ],
                vec![
                    Node::new_from_id(ID::new_id(TEST_BYTES_1, 256)),
                    Node::new_from_id(ID::new_id(TEST_BYTES_2, 256)),
                ],
                "",
            ),
            ("ok-empty", vec![], vec![], ""),
            (
                "ok-1",
                vec![
                    Node::new_from_id(ID::new_id(TEST_BYTES_4, 256)),
                    Node::new_from_id(ID::new_id(TEST_BYTES_3, 256)),
                    Node::new_from_id(ID::new_id(TEST_BYTES_2, 256)),
                    Node::new_from_id(ID::new_id(TEST_BYTES_1, 256)),
                ],
                vec![
                    Node::new_from_id(ID::new_id(TEST_BYTES_1, 256)),
                    Node::new_from_id(ID::new_id(TEST_BYTES_2, 256)),
                    Node::new_from_id(ID::new_id(TEST_BYTES_3, 256)),
                    Node::new_from_id(ID::new_id(TEST_BYTES_4, 256)),
                ],
                "",
            ),
            (
                "ok-2",
                vec![
                    Node::new_from_id(ID::new_id(TEST_BYTES_2, 256)),
                    Node::new_from_id(ID::new_id(TEST_BYTES_1, 256)),
                    Node::new_from_id(ID::new_id(TEST_BYTES_3, 256)),
                    Node::new_from_id(ID::new_id(TEST_BYTES_4, 256)),
                ],
                vec![
                    Node::new_from_id(ID::new_id(TEST_BYTES_1, 256)),
                    Node::new_from_id(ID::new_id(TEST_BYTES_2, 256)),
                    Node::new_from_id(ID::new_id(TEST_BYTES_3, 256)),
                    Node::new_from_id(ID::new_id(TEST_BYTES_4, 256)),
                ],
                "",
            ),
        ];

        for (desc, mut nodes, want, want_err) in test_cases {
            let mut arc_nodes = Vec::new();
            for node in nodes {
                arc_nodes.push(Arc::from(node));
            }
            match prepare(&mut arc_nodes, 256) {
                Ok(_) => {
                    let mut arc_want = Vec::new();
                    for node in want {
                        arc_want.push(Arc::from(node));
                    }
                    assert!(
                        want_err.is_empty(),
                        "{} NodesRow prepare expected error {}",
                        desc,
                        want_err
                    );
                    assert_eq!(
                        arc_nodes, arc_want,
                        "{} NodesRow prepare got {:?}, want {:?}",
                        desc, arc_nodes, arc_want
                    );
                }
                Err(err) => {
                    assert!(
                        !want_err.is_empty(),
                        "{} NodesRow prepare did not expect error, got err {}",
                        desc,
                        err
                    );
                    assert!(
                        err.contains(want_err),
                        "{} NodesRow prepare got err {}, want substring {}",
                        desc,
                        err,
                        want_err
                    );
                }
            }
        }
    }
}
