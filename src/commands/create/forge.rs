use crate::config::{load_config, find_version_url, download_server_jar, get_dir};
use std::fs;
use std::io::Write;

pub fn forge(name: String, version: String)
{

}

fn failsafe_remove(name: String, path: &std::path::Path)
{
    println!("removing server folder '{}' due to previous errors...", name);

    if let Err(e) = fs::remove_dir_all(path) {
        println!("failed to remove server folder: {}", e);
    }
}