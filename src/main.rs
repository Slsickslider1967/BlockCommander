use clap::{Parser, Subcommand};

mod config;    // src/config.rs
mod commands;  // src/commands/ folder, which itself declares its own submodules


 #[derive(Parser)]
 #[command(name = "BlockCommander")]
 #[command(about = "Manage Minecraft servers from the terminal")]
 #[command(after_help = "For more information, see the documentation at https://github.com/yourusername/BlockCommander")]
#[command(help_template = "\
{about-with-newline}
{usage-heading} {usage}

{subcommands}

Usage: BlockCommander config <SUBCOMMAND>

  servers-dir <dir>         Set the folder where servers are created and stored
  default-port <port>       Set the default port for new servers
  default-rcon-port <port>  Set the default RCON port for new servers
  sync                      Sync the configuration file with the current state of the servers folder
  set-max-ram <ram_mb>      Set the maximum RAM for new servers

Usage: BlockCommander <server-name> <COMMAND>

  start                 Start in the background
  start-gui             Start with the live dashboard
  stop                  Stop the server via RCON
  port <port>           Change the game port
  rcon-port <port>      Change the RCON port
  rcon-password <pass>  Change the RCON password

{options}{after-help}")]
 
 struct CLI 
 {
    #[command(subcommand)]
    command: Commands,
 }

 #[derive(Parser)]
struct ServerCli 
{
    #[command(subcommand)]
    command: ServerCommands,
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
    /// create <name> <version> [--loader <loader>]
    Create 
    {
        name: String,
        version: String,
        #[arg(long, value_enum, default_value = "vanilla")]
        loader: Loader,
    },
    /// Delete <name>
    Delete { name: String },
    /// List all servers.
    List,
    /// List all active servers.
    ListActive,

    /// Configure BlockCommander settings.
    Config 
    {
        #[command(subcommand)]
        action: ConfigAction,
    },

    /// Run a command on a server. The first argument is the server name, followed by the command and its arguments.
    #[command(external_subcommand)]
    Server(Vec<String>),
}

#[derive(Subcommand)]
enum ServerCommands
{
    Start,
    StartGui,
    Stop,
    Port { port: u16 },
    RconPort { port: u16 },
    RconPassword { password: String },
}

#[derive(Subcommand)]
enum ConfigAction 
{
    /// Set the folder where servers are created and stored
    ServersDir { dir: String },
    /// Set the default port for new servers
    DefualtPort { port: u16 },
    /// Set the default RCON port for new servers
    DefaultRconPort { port: u16 },
    /// Sync the configuration file with the current state of the servers folder
    Sync,
    /// Set the maximum RAM for new servers
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
    if args.is_empty() 
    {
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

    match parsed.command 
    {
        ServerCommands::Start => commands::start(server_name),
        ServerCommands::StartGui => commands::start_gui(server_name).await,
        ServerCommands::Stop => commands::stop(server_name).await,
        ServerCommands::Port { port } => commands::change_ports(server_name, Some(port), None),
        ServerCommands::RconPort { port } => commands::change_ports(server_name, None, Some(port)),
        ServerCommands::RconPassword { password } => commands::change_rcon_password(server_name, password),
    }
}