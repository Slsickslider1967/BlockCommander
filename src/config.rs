use serde::{Serialize, Deserialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Default)]
pub struct Config 
{
    pub servers_dir: Option<String>,
}

fn config_path() -> PathBuf 
{
    let mut path = dirs::config_dir().expect("couldn't find config dir");
    path.push("blockcom");
    path.push("config.toml");
    path
}

pub fn load_config() -> Config 
{
    let path = config_path();
    match std::fs::read_to_string(&path) 
    {
        Ok(contents) => toml::from_str(&contents).unwrap_or_default(),
        Err(_) => Config::default(),
    }
}

pub fn save_config(config: &Config) 
{
    let path = config_path();
    if let Some(parent) = path.parent() 
    {
        std::fs::create_dir_all(parent).expect("couldn't create config dir");
    }
    let contents = toml::to_string(config).expect("couldn't serialize config");
    std::fs::write(&path, contents).expect("couldn't write config file");
}