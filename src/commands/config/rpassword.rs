use crate::config::{load_config, save_config};

pub fn change_rcon_password(password: String)
{
    let mut config = load_config();
    config.rcon_password = Some(password);
    save_config(&config);
}