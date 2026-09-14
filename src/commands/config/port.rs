use crate::config::{load_config, save_config};

pub fn defualtport(port: u16)
{
    println!("changig default server port to '{}'", port);
    
    let mut config = load_config();
    config.port = Some(port);
    save_config(&config);
}