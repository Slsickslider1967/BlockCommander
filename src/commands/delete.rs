use crate::config::get_dir;
use std::fs;

pub fn delete(name: String)
{
    let dir = match get_dir() {
        Some(d) => d,
        None => return,
    };

    let server_path = std::path::Path::new(&dir).join(&name);
    let path = std::path::PathBuf::from(&server_path);
    println!("deleting server '{}' at '{}'", name, path.display());

    if let Err(e) = fs::remove_dir_all(path) {
        println!("failed to remove server folder: {}", e);
    }

    // Remove server_info from server list
    println!("removing server info for '{}' from server list", name);
    let mut server_list = crate::config::load_server_list();
    server_list.servers.retain(|s| s.name != name);
    crate::config::save_server_list(&server_list);
}
