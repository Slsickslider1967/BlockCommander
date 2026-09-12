use crate::Loader;
use crate::config::load_config;
use crate::config::find_version_url;
use crate::config::download_server_jar;
use crate::config::get_dir;
use crate::commands::config::sync;
use std::fs;
use std::io::Write;

pub fn create(name: String, version: String, loader: Loader) 
{
    match loader
    {
        Loader::Vanilla => vanilla(name, version),
        Loader::Fabric => fabric(name, version),
        Loader::Forge => forge(name, version),
        Loader::NeoForge => neoforge(name, version),
    }

    //sync();
}

fn vanilla(name: String, version: String) 
{
    println!("creating vanilla server '{}' running Minecraft {}", name, version);

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
    println!("getting version manifest from Mojang...");

    let version_url = match find_version_url(&version) 
    {
        Some(url) => url,
        None => {
            println!("couldn't find version '{}' in Mojang's manifest", version);
            failsafe_remove(name, &server_path);
            return;
        }
    };

    println!("found version url: {}", version_url);
    println!("downloading server jar...");

    let jar_path = server_path.join("server.jar");
    if let Err(e) = download_server_jar(&version_url, &jar_path) 
    {
        println!("failed to download server jar: {}", e);
        failsafe_remove(name, &server_path);
        return;
    }

    println!("downloaded server jar to '{}'", jar_path.display());

    let eula_path = server_path.join("eula.txt");
    let mut input = String::new();

    // EULA acceptance prompt with flish so y/n appears on the same line
    print!("Do you accept the Minecraft EULA? (https://account.mojang.com/documents/minecraft_eula) (y/n): ");
    std::io::stdout().flush().expect("Failed to flush stdout");
    std::io::stdin().read_line(&mut input).expect("Failed to read input");

    if input.trim().to_lowercase() != "y" {
        println!("EULA not accepted. Server creation aborted.");
        failsafe_remove(name, &server_path);
        return;
    }

    if let Err(e) = std::fs::write(&eula_path, "eula=true\n") {
        println!("failed to write eula.txt: {}", e);
        failsafe_remove(name, &server_path);
        return;
    }
    println!("server '{}' created successfully at '{}'", name, server_path.display());

    // Add server_info file
    let server_info = crate::config::ServerInfo 
    {
        name: name.clone(),
        version: version.clone(),
        loader: "Vanilla".to_string(),
        port: load_config().port.unwrap_or(25565),
    };
    crate::config::save_server_info(&server_path, &server_info);  
}

fn fabric(name: String, version: String) 
{
    println!("creating fabric server '{}' running Minecraft {}", name, version);
}

fn forge(name: String, version: String) 
{
    println!("creating forge server '{}' running Minecraft {}", name, version);
}

fn neoforge(name: String, version: String)
{
    println!("creating neoforge server '{}' running Minecraft {}", name, version);
}


fn failsafe_remove(name: String, path: &std::path::Path)
{
    println!("removing server folder '{}' due to previous errors...", name);

    if let Err(e) = fs::remove_dir_all(path) {
        println!("failed to remove server folder: {}", e);
    }
}