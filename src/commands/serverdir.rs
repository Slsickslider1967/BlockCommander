use crate::config::{load_config, save_config};

pub fn servers_dir(dir: String) {
    let mut config = load_config();
    config.servers_dir = Some(dir.clone());
    save_config(&config);
    println!("servers directory set to '{}'", dir);
}