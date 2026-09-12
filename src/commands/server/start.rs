use crossterm::event::PopKeyboardEnhancementFlags;

use crate::config::get_dir;
use crate::config;
use std::process::Stdio;
use std::io::Write;
use std::process::Command;

pub fn start(name: String)
{
    println!("Starting server '{}'", name);
 
    // load config to get servers directory
    let dir = match get_dir() {
        Some(d) => d,
        None => return,
    };
    let server_path: std::path::PathBuf  = std::path::Path::new(&dir).join(&name);

    ensure_server_initialized(&name, &server_path);
    
    let mut cmd = start_server(&name, &server_path);
}

pub fn first_time_server_properties(name: &String, server_path: &std::path::Path)
{
    println!("updating server.properties for '{}'", name);
    let info_path = server_path.join("server_info.toml");
    let info = match std::fs::read_to_string(&info_path)
        .ok()
        .and_then(|contents| toml::from_str::<crate::config::ServerInfo>(&contents).ok())
    {
        Some(info) => info,
        None => {
            eprintln!("Failed to read server info for '{}'.", name);
            return;
        }
    };
    let port = info.port;

    //Read the server.properties file
    let properties_path = server_path.join("server.properties");
    let mut properties = 
        match std::fs::read_to_string(&properties_path)
        {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Failed to read server.properties: {}", e);
                return;
            }
        };

    let mut updated = Vec::new();
    let mut portfound = false;
    let mut namefound = false;
    let mut rconfound = false;
    let mut enablefound = false;

    // Read through the file to fine needed changes
    for line in properties.lines()
    {
        if line.starts_with("server-port=") 
        {
            updated.push(format!("server-port={}", port));
            portfound = true;
        } 
        else if line.starts_with("rcon.port=")
        {
            updated.push(format!("rcon.port={}", info.rcon_port));
        }
        else if line.starts_with("enable-rcon=")
        {
            updated.push("enable-rcon=true".to_string());
        }
        else if line.starts_with("rcon.password=")
        {
            updated.push(format!("rcon.password={}", info.rcon_password));
        }
        else if line.starts_with("level-name=") 
        {
            updated.push(format!("level-name={}", name));
            namefound = true;
        }
        else 
        {
            updated.push(line.to_string());
        }  
    }

    if portfound 
    {
        println!("updated server-port to {}", port);
    }
    if namefound 
    {
        println!("updated server-name to {}", name);
    }
    

    let new_contents = updated.join("\n");

    if let Err(e) = std::fs::write(&properties_path, new_contents) 
    {
        eprintln!("failed to write server.properties: {}", e);
        return;
    }

    println!("updated server.properties for '{}'", name);

}

pub fn start_server(name: &String, server_path: &std::path::Path) -> std::process::Child
{
    println!("starting server '{}' from '{}'", name, server_path.display());

    let info_path = server_path.join("server_info.toml");
    let info = std::fs::read_to_string(&info_path)
        .ok()
        .and_then(|contents| toml::from_str::<crate::config::ServerInfo>(&contents).ok());

    let max_ram = info.as_ref().map(|i| i.max_ram_mb).unwrap_or(1024);

    // Run the server from server.jar
    std::process::Command::new("java")
        .arg(format!("-Xmx{}M", max_ram))
        .arg(format!("-Xms{}M", max_ram))
        .arg("-jar")
        .arg(server_path.join("server.jar"))
        .arg("nogui")
        .current_dir(server_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to start server process")

}

pub fn stop_server(cmd: &mut std::process::Child)
{
    let stdin = cmd.stdin.as_mut().expect("no stdin handle");
    stdin.write_all(b"stop\n").expect("failed to write to stdin");
}

pub fn ensure_server_initialized(name: &String, server_path: &std::path::Path)
{
    if !server_path.join("server.properties").exists()
    {
        println!("first time start up detected...");
        println!("creating server files...");

        let mut cmd = start_server(name, server_path);
        stop_server(&mut cmd);
        cmd.wait().expect("failed to wait on child");

        first_time_server_properties(&name, &server_path);
    }
}