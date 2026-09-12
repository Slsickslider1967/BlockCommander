use crate::config::{ServerInfo, get_dir, save_server_info};

pub fn change_ports(name: String, port: Option<u16>, rcon_port: Option<u16>) 
{
    let dir = match get_dir() 
    {
        Some(dir) => dir,
        None => return,
    };

    let info_path = std::path::Path::new(&dir)
        .join(&name)
        .join("server_info.toml");

    let contents = match std::fs::read_to_string(&info_path) 
    {
        Ok(contents) => contents,
        Err(_) => 
        {
            eprintln!("Server with name '{}' not found.", name);
            return;
        }
    };

    let mut server: ServerInfo = match toml::from_str(&contents) 
    {
        Ok(server) => server,
        Err(error) => 
        {
            eprintln!("Could not read server '{}': {}", name, error);
            return;
        }
    };

    if let Some(port) = port 
    {
        server.port = port;
    }
    if let Some(rcon_port) = rcon_port 
    {
        server.rcon_port = rcon_port;
    }

    save_server_info(std::path::Path::new(&dir).join(&name).as_path(), &server);
    println!("Updated ports for server '{}'.", name);
}