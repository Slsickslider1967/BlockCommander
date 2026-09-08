use crate::config::{load_config, save_config};

pub fn port(port: u16)
{
    let mut config = load_config();
    config.port = Some(port);
    save_config(&config);
}