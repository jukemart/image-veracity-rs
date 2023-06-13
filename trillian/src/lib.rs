use crate::api::trillian;
use crate::api::trillian::{
    CreateTreeRequest, ListTreesRequest, LogLeaf, QueueLeafRequest, Tree, TreeState, TreeType,
};
use std::time::Duration;
use tonic::Request;

pub mod api;

pub fn list_tree_request() -> Request<ListTreesRequest> {
    Request::new(ListTreesRequest { show_deleted: true })
}

pub fn create_tree_request(name: &str, description: &str) -> Request<CreateTreeRequest> {
    Request::new(CreateTreeRequest {
        tree: Option::from(Tree {
            tree_id: 0,
            tree_state: TreeState::Active.into(),
            tree_type: TreeType::Log.into(),
            display_name: name.to_string(),
            description: description.to_string(),
            storage_settings: None,
            max_root_duration: Option::from(
                prost_types::Duration::try_from(Duration::from_secs(3_600)).unwrap(),
            ),
            create_time: None,
            update_time: None,
            deleted: false,
            delete_time: None,
        }),
    })
}

pub fn form_leaf(tree_id: i64, entry: &[u8], extra_data: &[u8]) -> Request<QueueLeafRequest> {
    let leaf = LogLeaf {
        merkle_leaf_hash: vec![],
        leaf_value: entry.to_vec(),
        extra_data: extra_data.to_vec(),
        leaf_index: 0,
        leaf_identity_hash: vec![],
        queue_timestamp: None,
        integrate_timestamp: None,
    };
    let queue = QueueLeafRequest {
        log_id: tree_id,
        leaf: Option::from(leaf),
        charge_to: None,
    };
    Request::new(queue)
}
