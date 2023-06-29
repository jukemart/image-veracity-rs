use crate::node::id::ID;
use crate::node::NodesRow;

pub struct Tile<'a> {
    id: ID,
    leaves: NodesRow<'a>,
}
