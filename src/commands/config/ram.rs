use crate::config::{load_config, save_config};

pub fn set_max_ram(ram_mb: u32) 
{
    let mut config = load_config();
    config.max_ram_mb = ram_mb;
    save_config(&config);
    println!("Max RAM set to {} MB", ram_mb);
}