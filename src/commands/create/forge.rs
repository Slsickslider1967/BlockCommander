use crate::config::{load_config, find_version_url, get_dir, find_required_java_version, set_java_env};
use std::fs;
use std::io::Write;

pub fn forge(name: String, version: String)
{
    println!("creating forge server '{}' running Minecraft {}", name, version);

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

    let required_java = match find_version_url(&version)
    {
        Some(url) => find_required_java_version(&url),
        None => 17,
    };

    println!("please download the recommended forge installer for Minecraft {}", version);
    print!("enter the forge installer .jar directory: ");
    std::io::stdout().flush().expect("Failed to flush stdout");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).expect("Failed to read input");
    let installer_path = input.trim().to_string();

    if !std::path::Path::new(&installer_path).exists()
    {
        println!("that file doesn't exist");
        failsafe_remove(name, &server_path);
        return;
    }

    println!("running forge installer...");
    let mut installer = std::process::Command::new("java");
    installer
        .arg("-jar")
        .arg(&installer_path)
        .arg("--installServer")
        .current_dir(&server_path);
    set_java_env(&mut installer, required_java);
    installer.status().expect("Failed to run forge installer");

    if !server_path.join("run.sh").exists()
    {
        println!("forge installer failed to create run.sh");
        failsafe_remove(name, &server_path);
        return;
    }

    println!("running forge once to generate eula.txt...");
    let mut first_run = std::process::Command::new("./run.sh");
    first_run.current_dir(&server_path);
    set_java_env(&mut first_run, required_java);
    first_run.status().expect("Failed to run run.sh");

    let eula_path = server_path.join("eula.txt");
    if !eula_path.exists()
    {
        println!("forge failed to create eula.txt");
        failsafe_remove(name, &server_path);
        return;
    }

    let mut input = String::new();
    print!("Do you accept the Minecraft EULA? (https://account.mojang.com/documents/minecraft_eula) (y/n): ");
    std::io::stdout().flush().expect("Failed to flush stdout");
    std::io::stdin().read_line(&mut input).expect("Failed to read input");

    if input.trim().to_lowercase() != "y"
    {
        println!("EULA not accepted. Server creation aborted.");
        failsafe_remove(name, &server_path);
        return;
    }

    if let Err(e) = fs::write(&eula_path, "eula=true\n")
    {
        println!("failed to write eula.txt: {}", e);
        failsafe_remove(name, &server_path);
        return;
    }

    println!("adding server info...");
    let config = load_config();
    let mut server_list = crate::config::load_server_list();
    let port = crate::config::next_available_port(config.port.unwrap_or(25565), &server_list);
    let rcon_port = crate::config::next_available_rcon_port(config.rcon_port.unwrap_or(25575), &server_list);

    let server_info = crate::config::ServerInfo
    {
        name: name.clone(),
        version: version.clone(),
        loader: "Forge".to_string(),
        required_java,

        port,
        rcon_port,
        rcon_password: config.rcon_password.unwrap_or_else(|| "defaultpassword".to_string()),
        max_ram_mb: if config.max_ram_mb > 0 { config.max_ram_mb } else { 1024 },

        has_been_started: false,
        start_file: "defaultfile".to_string(),
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