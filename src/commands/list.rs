use crate::config::load_config;
use std::fs;

pub fn list()
{
    let config = load_config();

    let dir = match config.servers_dir 
    {
        Some(d) => d,
        None => {
            println!("no servers directory set — run `blockcom config servers-dir <path>` first");
            return;
        }
    };

    let entries = match fs::read_dir(&dir)
    {
        Ok(entries) => entries,
        Err(e) => {
            println!("couldn't read servers directory '{}': {}", dir, e);
            return;
        }
    };

    let mut found_any = false;
    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        // only show folders (each server should live in its own folder)
        if entry.path().is_dir() {
            if let Some(name) = entry.file_name().to_str() {
                println!("{}", name);
                found_any = true;
            }
        }
    }

    if found_any == false {
        println!("no servers found in '{}'", dir);
    }
}