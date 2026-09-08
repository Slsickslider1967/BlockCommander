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
    Run { target: String },

    Config 
    {
        #[command(subcommand)]
        action: ConfigAction,
    }
}

#[derive(Subcommand)]
enum ConfigAction {
    ServersDir { dir: String },
    Port { port: u16 },
}

fn main()
{
    let cli = CLI::parse();

    match cli.command
    {
        Commands::Create { name, version, loader } => commands::create(name, version, loader),
        Commands::Delete { name } => commands::delete(name),
        Commands::List => commands::list(),
        Commands::Run { target } => commands::run(target),

        Commands::Config { action } => match action 
        {
            ConfigAction::ServersDir { dir } => commands::servers_dir(dir),
            ConfigAction::Port { port } => commands::port(port),
        },
    }
}