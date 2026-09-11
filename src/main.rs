use clap::{Command, Parser, Subcommand};

mod config;    // src/config.rs
mod commands;  // src/commands/ folder, which itself declares its own submodules


 #[derive(Parser)]
 #[command(name = "BlockCommander")]
 #[command(about = "Manage Minecraft servers from the terminal")]
 struct CLI 
 {
    #[command(subcommand)]
    command: Commands,
 }

#[derive(clap::ValueEnum, Clone, Debug)]
pub(crate) enum Loader 
{
    Vanilla,
    Fabric,
    Forge,
    NeoForge,
}

#[derive(Subcommand)]
enum Commands
{
    Create 
    {
        name: String,
        version: String,
        #[arg(long, value_enum, default_value = "vanilla")]
        loader: Loader,
    },
    Delete { name: String },
    List,

    Config 
    {
        #[command(subcommand)]
        action: ConfigAction,
    },

    #[command(external_subcommand)]
    Server(Vec<String>),
}

#[derive(Subcommand)]
enum ConfigAction {
    ServersDir { dir: String },
    DefualtPort { port: u16 },
}

#[tokio::main]
async fn main()
{
    let cli = CLI::parse();

    match cli.command
    {
        Commands::Create { name, version, loader } => {tokio::task::spawn_blocking(move ||commands::create(name, version, loader)).await.expect("create task panicked");},
        Commands::Delete { name } => commands::delete(name),
        Commands::List => commands::list(),
        Commands::StartGui { target } => commands::start_gui(target).await,
        Commands::Config { action } => match action 
        {
            ConfigAction::ServersDir { dir } => commands::servers_dir(dir),
            ConfigAction::DefualtPort { port: defaultport } => commands::defualtport(defaultport),
        },

        Commands::Server(args) => handle_server_command(args).await,
    }
}

async fn handle_server_command(args: Vec<String>)
{
    if args.len() < 2 {
        eprintln!("Usage: BlockCommander <server> <command> [input]");
        return;
    }

    let server_name = args[0].clone();
    let command = args[1].as_str();

    match command
    {
        "start" => commands::start(server_name),
        "start-gui" => commands::start_gui(server_name).await,
        "stop" => commands::stop(server_name),
        "rename" => {
            if args.len() < 3 {
                eprintln!("Usage: BlockCommander <server> rename <new-name>");
                return;
            }
            commands::rename(server_name, args[2].clone());
        }
        _ => eprintln!("unknown server command '{}'", command),
    }
}