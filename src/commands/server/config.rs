use crate::config::get_dir;
use std::process::Stdio;
use std::io::Write;
use std::os::unix::prelude::CommandExt;

pub fn update_server_config(server_name: String, config_name: String, config_change: String)
{
    // Getting server path
    let dir = match get_dir() {
        Some(d) => d,
        None => return,
    };
    let server_path: std::path::PathBuf = std::path::Path::new(&dir).join(&server_name);

    // Reading server properties
    let properties_path = server_path.join("server.properties");
    let properties = match std::fs::read_to_string(&properties_path)
    {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Failed to read server.properties: {}", e);
            return;
        }
    };

    let mut updated: Vec<String> = Vec::new();

    for line in properties.lines()
    {
        if line.starts_with(config_name.as_str())
        {
            updated.push(format!("{}={}", config_name.as_str(), config_change.as_str()));
        }
        else
        {
            updated.push(line.to_string());
        }
    }

    if let Err(e) = std::fs::write(&properties_path, updated.join("\n"))
    {
        eprintln!("failed to write server.properties: {}", e);
        return;
    }
}