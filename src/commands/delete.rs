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
}