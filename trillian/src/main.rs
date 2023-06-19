use clap::{Args, Parser, Subcommand};
use eyre::Result;
use tracing::debug;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use trillian::client::{TrillianClient, TrillianClientApiMethods};

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

#[derive(Clone, Debug, Subcommand)]
enum AdminCommands {
    /// List all trees
    ListTrees,
    /// Create a new tree
    CreateTree(CreateTreeArgs),
}

#[derive(Clone, Debug, Args)]
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

#[derive(Clone, Debug, Subcommand)]
enum ClientCommands {
    /// Add new leaf to tree
    AddLeaf(AddLeafArgs),
}

#[derive(Clone, Debug, Args)]
struct AddLeafArgs {
    #[arg(short, long)]
    /// Tree ID to add new leaf
    tree_id: i64,
    #[arg(short, long)]
    /// Data to add in leaf
    data: String,
    #[arg(short, long)]
    /// Optional extra data to add with leaf
    extra_data: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();

    // Set verbosity level
    let verbosity_level = match args.verbose {
        0 => "warn",
        1 => "debug",
        _ => "trace",
    };

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("trillian_client_cli={verbosity_level},trillian_client={verbosity_level}")
                    .into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    debug!("Verbosity level: {verbosity_level}");

    let mut trillian = TrillianClient::new(args.address).await?.build();
    debug!("Created Trillian client");

    match &args.submodule {
        Submodules::Admin(admin_args) => {
            let admin_command = &admin_args.admin_commands;
            debug!("Admin client command {:?}", admin_command);

            match admin_command {
                AdminCommands::ListTrees => {
                    let trees = trillian.list_trees().await?;
                    for tree in trees {
                        println!("{tree:#?}")
                    }
                }
                AdminCommands::CreateTree(CreateTreeArgs { name, description }) => {
                    let tree = trillian.create_tree(&name, &description).await?;
                    println!("New Tree ID: {}", &tree.tree_id);
                }
            }
        }
        Submodules::Client(client_args) => {
            let client_command = &client_args.client_commands;
            debug!("Log client command {:?}", client_command);

            match &client_args.client_commands {
                ClientCommands::AddLeaf(AddLeafArgs {
                    tree_id,
                    data,
                    extra_data,
                }) => {
                    let extra_data_bytes = if let Some(extra) = extra_data {
                        extra.as_bytes()
                    } else {
                        &[]
                    };
                    let leaf = trillian
                        .add_leaf(tree_id, data.as_bytes(), extra_data_bytes)
                        .await?;
                    println!(
                        "Queued leaf index {} and hash {:x?}",
                        &leaf.leaf_index, &leaf.leaf_identity_hash
                    );
                }
            }
        }
    }

    Ok(())
}
