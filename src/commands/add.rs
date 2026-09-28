use std::path::Path;
use std::fs;
use std::io::Write;
use clap::ValueEnum;
use crate::Loader;
use crate::config::*;

pub fn Add_Extern_Server(extern_dir: String)
{
    let extern_path = match std::fs::canonicalize(&extern_dir)
    {
        Ok(p) if p.is_dir() => p,
        _ => {
            println!("external server folder does not exist");
            return;
        }
    };

    let dir = match get_dir() {
        Some(d) => d,
        None => return,
    };

    let name = prompt("enter the server name");
    let version = prompt("enter the server minecraft version");
    let loader = prompt_loader();
    let start_file;

    start_file =

    let destination_path = Path::new(&dir).join(&name);
    if destination_path.exists()
    {
        println!("a server named '{}' already exists", name);
        return;
    }

    // stops infinite recursion if the servers dir is inside the folder being copied
    if destination_path.starts_with(&extern_path)
    {
        println!("the servers directory can't be inside the server you're copying");
        return;
    }

    println!("copying server files, this can take a while for big worlds...");
    if let Err(e) = copy_dir_recursive(&extern_path, &destination_path)
    {
        println!("failed to copy server: {}", e);
        let _ = fs::remove_dir_all(&destination_path); // clean up the half-finished copy
        return;
    }

    let config = load_config();
    let mut list = load_server_list();
    let port = next_available_port(config.port.unwrap_or(25565), &list);
    let rcon_port = next_available_rcon_port(config.rcon_port.unwrap_or(25575), &list);

    let info = ServerInfo
    {
        name,
        version,
        loader: loader.to_string(),

        port,
        rcon_port,
        rcon_password: config.rcon_password.unwrap_or_else(|| "defaultpassword".to_string()),
        max_ram_mb: if config.max_ram_mb > 0 { config.max_ram_mb } else { 1024 },

        has_been_started: true,
        start_file: start_file,
    };

    save_server_info(&destination_path, &info);
    list.servers.push(info);
    save_server_list(&list);
}

fn prompt(label: &str) -> String
{
    print!("{}: ", label);
    std::io::stdout().flush().expect("Failed to flush stdout");

    let mut input = String::new();
    std::io::stdin().read_line(&mut input).expect("Failed to read input");
    input.trim().to_string()
}

fn prompt_loader() -> Loader
{
    loop
    {
        let input = prompt("enter the server loader (vanilla, fabric, forge, neoforge)");

        match Loader::from_str(&input, true)
        {
            Ok(loader) => return loader,
            Err(_) => println!("'{}' isn't a valid loader, try again.", input),
        }
    }
}

impl std::fmt::Display for Loader
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result
    {
        let name = match self
        {
            Loader::Vanilla => "Vanilla",
            Loader::Fabric => "Fabric",
            Loader::Forge => "Forge",
            Loader::Neoforge => "NeoForge",
        };
        write!(f, "{}", name)
    }
}