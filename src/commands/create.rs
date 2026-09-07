use crate::Loader;
use crate::config::load_config;
use crate::config::find_version_url;
use crate::config::download_server_jar;
use crate::config::get_dir;
use std::fs;

pub fn create(name: String, version: String, loader: Loader) 
{
    match loader
    {
        Loader::Vanilla => vanilla(name, version),
        Loader::Fabric => fabric(name, version),
        Loader::Forge => forge(name, version),
        Loader::NeoForge => neoforge(name, version),
    }
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
            return;
        }
    };

    println!("found version url: {}", version_url);
    println!("downloading server jar...");

    let jar_path = server_path.join("server.jar");
    if let Err(e) = download_server_jar(&version_url, &jar_path) 
    {
        println!("failed to download server jar: {}", e);
        return;
    }

    println!("downloaded server jar to '{}'", jar_path.display());

    let eula_path = server_path.join("eula.txt");
    if let Err(e) = std::fs::write(&eula_path, "eula=true\n") {
        println!("failed to write eula.txt: {}", e);
        return;
    }

    println!("server '{}' created successfully at '{}'", name, server_path.display());
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


