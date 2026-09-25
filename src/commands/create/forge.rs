use crate::config::{load_config, find_version_url, download_server_jar, get_dir};
use std::fs;
use std::io::Write;

pub fn forge(name: String, version: String)
{
    println!("creating forge server '{}' running Minecraft {}", name, version);

    // Get general server path
    let dir = match get_dir() {
        Some(d) => d,
        None => return,
    };

    let server_path = std::path::Path::new(&dir).join(&name);
    if server_path.exists()
    {
        println!("server '{}' already exists in '{}'", name, dir);
        return;
    }
    if let Err(e) = fs::create_dir_all(&server_path) {
        println!("failed to create server folder: {}", e);
        return;
    }

    println!("created server folder at '{}'", server_path.display());

    // Get user installed Fabric loader .jar path
    println!("plase install the recommended version of forge for the game version you've selected");
    print!("enter the forge installer .jar directory: ");
    std::io::stdout().flush().expect("Failed to flush stdout");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).expect("Failed to read input");
    let forge_loader_path = input.trim().to_string();

    if !std::path::Path::new(&forge_loader_path).exists()
    {
        println!("that file doesn't exist");
        failsafe_remove(name, &server_path);
        return;
    }

    println!("running forge installer");

    let mut child = std::process::Command::new("java")
        .arg("-jar")
        .arg(forge_loader_path)
        .arg("--installServer")
        .current_dir(&server_path)
        .spawn()
        .expect("Failed to spawn forge installer");

    child.wait().expect("Failed to wait on forge installer");

    if !server_path.join("run.sh").exists()
    {
        println!("forge installer failed to create run.sh");
        failsafe_remove(name, &server_path);
        return;
    }
    else
    {
        println!("forge server created successfully at '{}'", server_path.display());
    }

    // Start server .sh for acepting the eula
    let mut child = std::process::Command::new("./run.sh")
        .current_dir(&server_path)
        .spawn()
        .expect("Failed to spawn forge installer");

    child.wait().expect("failed to wait on Fabric server");

    // EULA acceptance prompt with flish so y/n appears on the same line

    let eula_path = server_path.join("eula.txt");
    let mut input = String::new();

    if (!eula_path.exists())
    {
        println!("forge installer failed to create eula");
        failsafe_remove(name, &server_path);
        return;
    }

    print!("Do you accept the Minecraft EULA? (https://account.mojang.com/documents/minecraft_eula) (y/n): ");
    std::io::stdout().flush().expect("Failed to flush stdout");
    std::io::stdin().read_line(&mut input).expect("Failed to read input");

    if input.trim().to_lowercase() != "y"
    {
        println!("EULA not accepted. Server creation aborted.");
        failsafe_remove(name, &server_path);
        return;
    }

    if let Err(e) = std::fs::write(&eula_path, "eula=true\n")
    {
        println!("failed to write eula.txt: {}", e);
        failsafe_remove(name, &server_path);
        return;
    }

    // Add server_info file
    println!("adding server info...");
    let config = load_config();
    let mut server_list = crate::config::load_server_list();
    let base_game_port = config.port.unwrap_or(25565);
    let game_port = crate::config::next_available_port(base_game_port, &server_list);
    let base_rcon_port = config.rcon_port.unwrap_or(25575);
    let rcon_port = crate::config::next_available_rcon_port(base_rcon_port, &server_list);

    let max_ram_mb;
    if config.max_ram_mb > 0
    {
        max_ram_mb = config.max_ram_mb;
    }
    else
    {
        println!("Warning: max_ram_mb is not set in the config. Defaulting to 1024 MB.");
        max_ram_mb = 1024;
    };

    let server_info = crate::config::ServerInfo
    {
        name: name.clone(),
        version: version.clone(),
        loader: "Forge".to_string(),
        port: game_port,
        rcon_port,
        rcon_password: config.rcon_password.unwrap_or_else(|| "defaultpassword".to_string()),

        max_ram_mb: max_ram_mb,

        has_been_started: false,
    };
    crate::config::save_server_info(&server_path, &server_info);
    server_list.servers.push(server_info);
    crate::config::save_server_list(&server_list);
}

fn failsafe_remove(name: String, path: &std::path::Path)
{
    println!("removing server folder '{}' due to previous errors...", name);

    if let Err(e) = fs::remove_dir_all(path) {
        println!("failed to remove server folder: {}", e);
    }
}