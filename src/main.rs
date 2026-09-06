 use clap::{Parser, Subcommand};

 #[derive(Parser)]
 #[command(name = "BlockCommander")]
 #[command(about = "Manage Minecraft servers from the terminal")]
 struct CLI 
 {
    #[command(subcommand)]
    command: Commands,
 }

 #[derive(Subcommand)]
 enum Commands
 {
    Create { name: String, version: String },
    List,
    Setwfolder { dir: String },
    Run { target: String },
 }

fn main()
{
    let cli = CLI::parse();

    match cli.command
    {
        Commands::Create { name, version } => {
            println!("Creating server '{}' with version '{}'", name, version);
        }
        Commands::List => {
            println!("Listing all servers...");
        }
        Commands::Setwfolder { dir } => {
            println!("Setting working folder to '{}'", dir);
        }
        Commands::Run { target } => {
            println!("Running server '{}'", target);
        }
    }
}