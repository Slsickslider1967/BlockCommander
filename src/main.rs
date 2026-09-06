 use clap::{Parser, Subcommand};

 mod config;
 mod commands;

 #[derive(Parser)]
 #[command(name = "BlockCommander")]
 #[command(about = "Manage Minecraft servers from the terminal")]
 struct CLI 
 {
    #[command(subcommand)]
    command: Commands,
 }

#[derive(clap::ValueEnum, Clone, Debug)]
pub(crate) enum Loader {
    Vanilla,
    Fabric,
    Forge,
    NeoForge,
}

 #[derive(Subcommand)]
 enum Commands
 {
    Create {
        name: String,
        version: String,
        #[arg(long, value_enum, default_value = "vanilla")]
        loader: Loader,
    },
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
}

fn main()
{
    let cli = CLI::parse();

    match cli.command
    {
        Commands::Create { name, version, loader } => commands::create(name, version, loader),
        Commands::List => commands::list(),
        Commands::Config { action } => match action {
            ConfigAction::ServersDir { dir } => commands::servers_dir(dir),
        },
        Commands::Run { target } => {
            println!("Running server '{}'", target);
        }
    }
}