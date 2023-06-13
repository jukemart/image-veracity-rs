use std::any::Any;
use std::convert::Infallible;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::process;

use clap::builder::Str;
use clap::{Args, Command, FromArgMatches, Parser, Subcommand, ValueEnum};
use tokio;
use tonic::transport::{Channel, Endpoint, Error};
use tonic::{Request, Response, Status};
use tracing::{debug, error, info, span, trace, warn, Level};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use trillian_client::api::trillian;
use trillian_client::api::trillian::trillian_admin_client::TrillianAdminClient;
use trillian_client::api::trillian::trillian_log_client::TrillianLogClient;
use trillian_client::api::trillian::{
    InitLogResponse, ListTreesRequest, ListTreesResponse, QueueLeafResponse, Tree,
};

/// Simple Trillian Client CLI
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Address of Trillian instance
    #[arg(short, long)]
    address: String,

    /// Turn debugging information on. Use multiple to increase verbosity level
    #[arg(short, action = clap::ArgAction::Count)]
    verbose: u8,

    #[command(subcommand)]
    submodule: Submodules,
}

#[derive(Subcommand)]
enum Submodules {
    /// Trillian Admin Client
    Admin(AdminArgs),
    /// Trillian Log Client
    Client(ClientArgs),
}

#[derive(Clone, Args)]
struct AdminArgs {
    #[command(subcommand)]
    admin_commands: AdminCommands,
}

#[derive(Clone, Subcommand)]
enum AdminCommands {
    /// List all trees
    ListTrees,
    /// Create a new tree
    CreateTree(CreateTreeArgs),
}

#[derive(Clone, Args)]
struct CreateTreeArgs {
    #[arg(short, long)]
    /// Name of the new tree
    name: String,
    #[arg(short, long)]
    /// Short description of the tree
    description: String,
}

#[derive(Clone, Args)]
struct ClientArgs {
    #[command(subcommand)]
    client_commands: ClientCommands,
}

#[derive(Clone, Subcommand)]
enum ClientCommands {
    /// Add new leaf to tree
    AddLeaf(AddLeafArgs),
}

#[derive(Clone, Args)]
struct AddLeafArgs {
    #[arg(short, long)]
    /// Tree ID to add new leaf
    tree_id: i64,
    #[arg(short, long)]
    /// Data to add in leaf
    data: String,
    #[arg(short, long)]
    /// Extra data to add with leaf
    extra_data: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    // Set verbosity level
    let tracing_name = "trillian_client_cli";
    let verbosity_level = match args.verbose {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{tracing_name}={verbosity_level}").into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    info!("Verbosity level: {verbosity_level}");

    let trillian_address = args.address.clone();
    info!("Connecting to {}!", &trillian_address);

    let trillian_endpoint = match Endpoint::from_shared(args.address.to_string()) {
        Ok(x) => x,
        Err(_) => {
            error!(
                "Could not create endpoint from address {}",
                &trillian_address
            );
            process::exit(1);
        }
    };

    match &args.submodule {
        Submodules::Admin(admin_args) => {
            debug!("Creating Admin client");
            let mut admin_client =
                match trillian::trillian_admin_client::TrillianAdminClient::connect(
                    trillian_endpoint,
                )
                .await
                {
                    Ok(x) => {
                        trace!("Successfully connected Admin client");
                        x
                    }
                    Err(_) => {
                        error!("Could not connect to address {}", &trillian_address);
                        process::exit(1);
                    }
                };

            match &admin_args.admin_commands {
                AdminCommands::ListTrees => {
                    let span = span!(Level::TRACE, "list_tree_request");
                    let _enter = span.enter();

                    trace!("Creating list_tree_request");
                    let request = trillian_client::list_tree_request();

                    trace!("Sending request {:?}", request);
                    let response = match admin_client.list_trees(request).await {
                        Ok(x) => {
                            trace!("Received response");
                            x
                        }
                        Err(err) => err_response(err),
                    };

                    for tree_response in response.into_inner().tree {
                        println!("{:#?}", tree_response);
                    }
                }
                AdminCommands::CreateTree(CreateTreeArgs { name, description }) => {
                    let span = span!(Level::TRACE, "create_tree_request");
                    let _enter = span.enter();

                    trace!("Creating create_tree_request");
                    let request = trillian_client::create_tree_request(&name, &description);

                    trace!("Sending request {:?}", request);
                    let response = match admin_client.create_tree(request).await {
                        Ok(x) => {
                            trace!("Received response");
                            x
                        }
                        Err(err) => err_response(err),
                    };
                    let tree = response.into_inner();
                    println!("New Tree ID: {}", tree.tree_id);

                    // New trees must be initialized by a log client
                    let trillian_endpoint =
                        Endpoint::from_shared(args.address.to_string()).unwrap();
                    let mut log_client =
                        match trillian::trillian_log_client::TrillianLogClient::connect(
                            trillian_endpoint,
                        )
                        .await
                        {
                            Ok(x) => {
                                trace!("Successfully connected Admin client");
                                x
                            }
                            Err(_) => {
                                error!("Could not connect to address {}", &trillian_address);
                                process::exit(1);
                            }
                        };
                    let request = tonic::Request::new(trillian::InitLogRequest {
                        log_id: tree.tree_id,
                        charge_to: None,
                    });
                    match log_client.init_log(request).await {
                        Ok(x) => {
                            debug!("Initialized the new tree");
                            x
                        }
                        Err(err) => {
                            error!("Could not initialize {}", err.to_string());
                            process::exit(1);
                        }
                    };
                }
            }
        }
        Submodules::Client(client_args) => {
            debug!("Creating Log client");
            let mut log_client =
                match trillian::trillian_log_client::TrillianLogClient::connect(trillian_endpoint)
                    .await
                {
                    Ok(x) => {
                        trace!("Successfully connected Log client");
                        x
                    }
                    Err(_) => {
                        error!("Could not connect to address {}", &trillian_address);
                        process::exit(1);
                    }
                };
            match &client_args.client_commands {
                ClientCommands::AddLeaf(AddLeafArgs {
                    tree_id,
                    data,
                    extra_data,
                }) => {
                    let span = span!(Level::TRACE, "add_leaf_request");
                    let _enter = span.enter();

                    let request = trillian_client::form_leaf(
                        tree_id.clone(),
                        data.as_bytes(),
                        extra_data.as_bytes(),
                    );
                    let response = match log_client.queue_leaf(request).await {
                        Ok(x) => {
                            trace!("Received response");
                            x
                        }
                        Err(err) => err_response(err),
                    };
                    let leaf = response.into_inner().queued_leaf.unwrap().leaf.unwrap();
                    println!(
                        "Queued leaf index {} and hash {:x?}",
                        leaf.leaf_index, leaf.leaf_identity_hash
                    )
                }
            }
        }
    }

    Ok(())
}

fn err_response(err: Status) -> ! {
    error!("Error receiving response: {}", err.to_string());
    process::exit(1);
}
