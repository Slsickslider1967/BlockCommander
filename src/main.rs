use clap::{Parser, Subcommand};

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

    Server {
        name: String,
        #[command(subcommand)]
        command: ServerCommand,
    },
}

#[derive(Subcommand)]
enum ServerCommand {
    Start,
    StartGui,
    Stop,
    Port { port: u16 },
    RconPort { port: u16 },
}

#[derive(Subcommand)]
enum ConfigAction {
    ServersDir { dir: String },
    DefualtPort { port: u16 },
    DefaultRconPort { port: u16 },
    Sync,
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
        Commands::Config { action } => match action 
        {
            ConfigAction::ServersDir { dir } => commands::servers_dir(dir),
            ConfigAction::DefaultRconPort { port: defaultrconport } => commands::default_rcon_port(defaultrconport),
            ConfigAction::DefualtPort { port: defaultport } => commands::defualtport(defaultport),
            ConfigAction::Sync => commands::sync(),
        },

        Commands::Server { name, command } => handle_server_command(name, command).await,
    }
}

async fn handle_server_command(server_name: String, command: ServerCommand)
{
    match command {
        ServerCommand::Start => commands::start(server_name),
        ServerCommand::StartGui => commands::start_gui(server_name).await,
        ServerCommand::Stop => commands::stop(server_name),
        ServerCommand::Port { port } => {
            commands::change_ports(server_name, Some(port), None)
        }
        ServerCommand::RconPort { port } => {
            commands::change_ports(server_name, None, Some(port))
        }
    }
}
