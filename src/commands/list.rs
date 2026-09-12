use crate::config::*;
use std::fs;

pub fn list()
{
    let dir = match get_dir() {
        Some(d) => d,
        None => return,
    };

    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                let info_path = entry.path().join("server_info.toml");
                if let Ok(contents) = fs::read_to_string(&info_path) {
                    if let Ok(info) = toml::from_str::<ServerInfo>(&contents) {
                        println!(" - {} (version: {}, loader: {}, port: {})", info.name, info.version, info.loader, info.port);
                    }
                }
            }
        }
    }
}