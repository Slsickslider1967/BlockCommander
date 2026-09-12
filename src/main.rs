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

 #[derive(Parser)]
struct ServerCli {
    #[command(subcommand)]
    command: ServerCommand,
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
    ListActive,

    Config 
    {
        #[command(subcommand)]
        action: ConfigAction,
    },

    #[command(external_subcommand)]
    Server(Vec<String>),
}

#[derive(Subcommand)]
enum ServerCommand {
    Start,
    StartGui,
    Stop,
    Port { port: u16 },
    RconPort { port: u16 },
    RconPassword { password: String },
}

#[derive(Subcommand)]
enum ConfigAction {
    ServersDir { dir: String },
    DefualtPort { port: u16 },
    DefaultRconPort { port: u16 },
    Sync,
    SetMaxRam { ram_mb: u32 },
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
        Commands::ListActive => commands::list_active().await,
        Commands::Config { action } => match action 
        {
            ConfigAction::ServersDir { dir } => commands::servers_dir(dir),
            ConfigAction::DefaultRconPort { port: defaultrconport } => commands::default_rcon_port(defaultrconport),
            ConfigAction::DefualtPort { port: defaultport } => commands::defualtport(defaultport),
            ConfigAction::Sync => commands::sync(),
            ConfigAction::SetMaxRam { ram_mb } => commands::set_max_ram(ram_mb),
        },

        Commands::Server(args) => handle_server_command(args).await,
    }
}

async fn handle_server_command(args: Vec<String>)
{
    if args.is_empty() {
        eprintln!("Usage: BlockCommander <server-name> <command> [args]");
        return;
    }

    let server_name = args[0].clone();
    let rest = &args[1..];

    // clap needs a "program name" as the first element when parsing manually
    let full_args = std::iter::once("blockcommander".to_string())
        .chain(rest.iter().cloned());

    let parsed = match ServerCli::try_parse_from(full_args) {
        Ok(p) => p,
        Err(e) => {
            e.print().expect("failed to print error");
            return;
        }
    };

    match parsed.command {
        ServerCommand::Start => commands::start(server_name),
        ServerCommand::StartGui => commands::start_gui(server_name).await,
        ServerCommand::Stop => commands::stop(server_name).await,
        ServerCommand::Port { port } => commands::change_ports(server_name, Some(port), None),
        ServerCommand::RconPort { port } => commands::change_ports(server_name, None, Some(port)),
        ServerCommand::RconPassword { password } => commands::change_rcon_password(server_name, password),
    }
}