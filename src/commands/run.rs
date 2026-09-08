
use crossterm::event::PopKeyboardEnhancementFlags;

use crate::config::get_dir;
use crate::config;
use std::process::Stdio;
use std::io::Write;
use std::process::Command;

pub fn run(name: String)
{
    println!("Starting server '{}'", name);
 
    // load config to get servers directory
    let dir = match get_dir() {
        Some(d) => d,
        None => return,
    };
    let server_path: std::path::PathBuf  = std::path::Path::new(&dir).join(&name);
    let is_first_time = !server_path.join("server.properties").exists();

    if is_first_time
    {
        println!("first time start up detected...");
        println!("creating server files...");

        let mut cmd = run_server(&name, &server_path);
        end_server(&mut cmd);
        cmd.wait().expect("failed to wait on child");

        firsttime_server_properties(&name, &server_path);
    }

    
    let mut cmd = run_server(&name, &server_path);
}

fn firsttime_server_properties(name: &String, server_path: &std::path::Path)
{
    // port configuration
    let config = crate::config::load_config();
    let port = config.port.unwrap_or(25565);

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

    // Read through the file to fine needed changes
    for line in properties.lines()
    {
        if line.starts_with("server-port=") 
        {
            updated.push(format!("server-port={}", port));
            portfound = true;
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

fn run_server(name: &String, server_path: &std::path::Path) -> std::process::Child
{
    println!("running server '{}' from '{}'", name, server_path.display());

    // Run the server from server.jar
    std::process::Command::new("java")
        .arg("-jar")
        .arg(server_path.join("server.jar"))
        //.arg("nogui")
        .current_dir(server_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to start server process")

}

fn end_server(cmd: &mut std::process::Child)
{
    let stdin = cmd.stdin.as_mut().expect("no stdin handle");
    stdin.write_all(b"stop\n").expect("failed to write to stdin");
}