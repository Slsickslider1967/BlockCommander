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
    let mut child = std::process::Command::new("./")
        .arg("run.sh")
        .current_dir(&server_path)
        .spawn()
        .expect("Failed to spawn forge installer");

    child.wait().expect("failed to wait on Fabric server");
}

fn failsafe_remove(name: String, path: &std::path::Path)
{
    println!("removing server folder '{}' due to previous errors...", name);

    if let Err(e) = fs::remove_dir_all(path) {
        println!("failed to remove server folder: {}", e);
    }
}