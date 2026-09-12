use crate::config::{load_config, save_config};

pub fn default_rcon_port(port: u16)
{
    let mut config = load_config();
    config.rcon_port = Some(port);
    save_config(&config);
}