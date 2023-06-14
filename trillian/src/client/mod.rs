use std::process;
use std::time::Duration;

use eyre::{Report, Result};
use thiserror::Error;
use tonic::transport::{Channel, Endpoint, Uri};
use tonic::{Request, Status};
use tracing::{debug, error, instrument, trace};

use crate::{
    protobuf::trillian,
    protobuf::trillian::trillian_admin_client::TrillianAdminClient,
    protobuf::trillian::trillian_log_client::TrillianLogClient,
    protobuf::trillian::{
        CreateTreeRequest, ListTreesRequest, LogLeaf, QueueLeafRequest, Tree, TreeState, TreeType,
    },
};

#[derive(Builder)]
#[builder(
    custom_constructor,
    create_empty = "empty",
    build_fn(private, name = "fallible_build")
)]
pub struct TrillianClient {
    #[builder(setter(custom))]
    admin_client: TrillianAdminClient<Channel>,
    #[builder(setter(custom))]
    log_client: TrillianLogClient<Channel>,
}

impl Clone for TrillianClient {
    fn clone(&self) -> Self {
        // Cloning the clients should be lightweight https://github.com/hyperium/tonic/issues/33
        TrillianClient {
            log_client: self.log_client.clone(),
            admin_client: self.admin_client.clone(),
        }
    }
}

impl TrillianClient {
    #[instrument(skip(host))]
    pub async fn new(host: impl Into<String>) -> Result<TrillianClientBuilder> {
        let host = host.into();
        // Create Tonic endpoint
        trace!("Creating host uri from {}", &host);
        let host_uri = match Uri::try_from(host.clone()) {
            Ok(x) => x,
            Err(err) => {
                error!("Could not create URI: {}", err.to_string());
                process::exit(1);
            }
        };
        debug!("Connecting to host uri {}", &host_uri);
        let endpoint = Endpoint::from(host_uri);

        let admin_client = match TrillianAdminClient::connect(endpoint.clone()).await {
            Ok(x) => {
                trace!("Successfully connected Admin client");
                x
            }
            Err(err) => {
                error!("Could not connect Admin client");
                return Err(Report::from(err));
            }
        };
        let log_client =
            match trillian::trillian_log_client::TrillianLogClient::connect(endpoint.clone()).await
            {
                Ok(x) => {
                    trace!("Successfully connected Log client");
                    x
                }
                Err(err) => {
                    error!("Could not connect Admin client");
                    return Err(Report::from(err));
                }
            };
        trace!("Created Trillian client builder");
        Ok(TrillianClientBuilder {
            admin_client: Some(admin_client),
            log_client: Some(log_client),
        })
    }

    #[instrument(skip(self))]
    pub async fn list_trees(&mut self) -> Result<Vec<Tree>> {
        trace!("Creating list_tree_request");
        let request = list_tree_request();

        trace!("Sending request {:?}", request);
        let response = match self.admin_client.list_trees(request).await {
            Ok(x) => {
                trace!("Received response");
                x
            }
            Err(err) => {
                error!("Could not list trees {:?}", err);
                return Err(Report::from(err));
            }
        };

        let mut trees = vec![];
        for tree_response in response.into_inner().tree {
            // println!("{:#?}", tree_response);
            trees.push(tree_response);
        }
        debug! {"{trees:?}"}
        Ok(trees)
    }

    #[instrument(skip(self))]
    pub async fn create_tree(&mut self, name: &str, description: &str) -> Result<Tree> {
        trace!("Creating create_tree_request");
        let request = create_tree_request(name, description);

        trace!("Sending request {:?}", request);
        let response = match self.admin_client.create_tree(request).await {
            Ok(x) => {
                trace!("Received response");
                x
            }
            Err(err) => {
                error!("Could not create tree {:?}", err);
                return Err(Report::from(err));
            }
        };
        let tree = response.into_inner();
        trace!("Created tree {:?}", &tree);

        // New trees must be initialized by a log_client
        let request = Request::new(trillian::InitLogRequest {
            log_id: tree.tree_id,
            charge_to: None,
        });
        match self.log_client.init_log(request).await {
            Ok(x) => {
                debug!("Initialized the new tree");
                x
            }
            Err(err) => {
                error!("Could not initialize {:?}", err);
                return Err(Report::from(err));
            }
        };
        debug! {"{tree:?}"}
        Ok(tree)
    }

    #[instrument(skip(self))]
    pub async fn add_leaf(
        &mut self,
        tree_id: &i64,
        data: &[u8],
        extra_data: &[u8],
    ) -> Result<LogLeaf> {
        let request = form_leaf(*tree_id, data, extra_data);
        let response = match self.log_client.queue_leaf(request).await {
            Ok(x) => {
                trace!("Received response {:?}", x);
                x
            }
            Err(err) => {
                return Err(Report::from(TrillianClientError::BadStatus(err)));
            }
        };
        let leaf = response.into_inner().queued_leaf.unwrap().leaf.unwrap();

        debug!(
            "Queued leaf index: {}, Merkle hash:{:x?}, QueueTs:{:?} IntegrateTs:{:?}",
            &leaf.leaf_index,
            &leaf.leaf_identity_hash,
            &leaf.queue_timestamp,
            &leaf.integrate_timestamp
        );
        Ok(leaf)
    }
}

impl TrillianClientBuilder {
    #[instrument(skip(self))]
    pub fn build(&self) -> TrillianClient {
        trace!("Created Trillian client");
        self.fallible_build()
            .expect("All required fields were initialized")
    }
}

fn list_tree_request() -> Request<ListTreesRequest> {
    Request::new(ListTreesRequest { show_deleted: true })
}

fn create_tree_request(name: &str, description: &str) -> Request<CreateTreeRequest> {
    Request::new(CreateTreeRequest {
        tree: Option::from(Tree {
            tree_state: TreeState::Active.into(),
            tree_type: TreeType::Log.into(),
            display_name: name.to_string(),
            description: description.to_string(),
            max_root_duration: Option::from(
                prost_types::Duration::try_from(Duration::from_secs(3_600)).unwrap(),
            ),
            ..Tree::default()
        }),
    })
}

fn form_leaf(tree_id: i64, entry: &[u8], extra_data: &[u8]) -> Request<QueueLeafRequest> {
    let leaf = LogLeaf {
        leaf_value: entry.to_vec(),
        extra_data: extra_data.to_vec(),
        ..LogLeaf::default()
    };
    let queue = QueueLeafRequest {
        log_id: tree_id,
        leaf: Option::from(leaf),
        ..QueueLeafRequest::default()
    };
    Request::new(queue)
}

#[derive(Error, Debug)]
pub enum TrillianClientError {
    #[error(transparent)]
    BadStatus(#[from] Status),
}
