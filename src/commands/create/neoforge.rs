use crate::config::{load_config, find_version_url, get_dir, version_is_greater_than, neoforge_version_getter, find_neoforge_installer_version, find_required_java_version, set_java_env};
use std::fs;
use std::io::Write;

pub fn neoforge(name: String, version: String)
{
    if !version_is_greater_than(&version, "1.20.2")
    {
        println!("NeoForge is only supported past 1.20.2, '{}' is too old", version);
        return;
    }

    println!("creating neoforge server '{}' running Minecraft {}", name, version);

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
    if let Err(e) = fs::create_dir_all(&server_path)
    {
        println!("failed to create server folder: {}", e);
        return;
    }

    let required_java = match find_version_url(&version)
    {
        Some(url) => find_required_java_version(&url),
        None => 17,
    };

    println!("getting NeoForge version...");
    let prefix = match neoforge_version_getter(&version)
    {
        Some(v) => format!("{}.", v),
        None => {
            println!("could not work out the NeoForge version prefix for '{}'", version);
            failsafe_remove(name, &server_path);
            return;
        }
    };

    let neoforge_version = match find_neoforge_installer_version(&prefix)
    {
        Some(v) => v,
        None => {
            println!("couldn't find a NeoForge release for '{}'", version);
            failsafe_remove(name, &server_path);
            return;
        }
    };

    let installer_name = format!("neoforge-{}-installer.jar", neoforge_version);
    let installer_url = format!(
        "https://maven.neoforged.net/releases/net/neoforged/neoforge/{v}/neoforge-{v}-installer.jar",
        v = neoforge_version
    );

    println!("downloading {}...", installer_name);
    let bytes = match reqwest::blocking::get(&installer_url)
        .and_then(|r| r.error_for_status())
        .and_then(|r| r.bytes())
    {
        Ok(b) => b,
        Err(e) => {
            println!("failed to download NeoForge installer: {}", e);
            failsafe_remove(name, &server_path);
            return;
        }
    };

    if let Err(e) = fs::write(server_path.join(&installer_name), bytes)
    {
        println!("failed to save NeoForge installer: {}", e);
        failsafe_remove(name, &server_path);
        return;
    }

    println!("running installer...");
    let mut installer = std::process::Command::new("java");
    installer
        .arg("-jar")
        .arg(&installer_name)
        .arg("--installServer")
        .current_dir(&server_path);
    set_java_env(&mut installer, required_java);
    installer.status().expect("Failed to run NeoForge installer");

    if !server_path.join("run.sh").exists()
    {
        println!("neoforge installer failed to create run.sh");
        failsafe_remove(name, &server_path);
        return;
    }

    let mut first_run = std::process::Command::new("./run.sh");
    first_run.current_dir(&server_path);
    set_java_env(&mut first_run, required_java);
    first_run.status().expect("Failed to run run.sh");

    let eula_path = server_path.join("eula.txt");
    if !eula_path.exists()
    {
        println!("neoforge failed to create eula.txt");
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
        loader: "NeoForge".to_string(),
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

fn get_neoforge_version(version: String) -> Option<String>
{

    let minecraft_prefix = match neoforge_version_getter(&version)
    {
        Some(v) => format!("{}.", v),
        None =>
            {
                println!("could not determie NeoForge prefect for '{}", version);
                return None;
            }
    };

    let neoforge_version = match find_neoforge_installer_version(&minecraft_prefix)
    {
        Some(v) => v,
        None => {
            println!("couldn't determine NeoForge version for '{}'", version);
            return None;
        }
    };

    let installer_url = format!(
        "https://maven.neoforged.net/releases/net/neoforged/neoforge/{v}/neoforge-{v}-installer.jar",
        v = neoforge_version
    );

    Some(installer_url)
}