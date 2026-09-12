use serde::{Serialize, Deserialize};
use std::path::PathBuf;

// Configuration for BlockCommander, stored in a TOML file in the user's config directory.

#[derive(Serialize, Deserialize, Default)]
pub struct Config 
{
    pub servers_dir: Option<String>,
    pub port: Option<u16>,
    pub rcon_port: Option<u16>,
    pub rcon_password: Option<String>,
}

fn config_path() -> PathBuf 
{
    let mut path = dirs::config_dir().expect("couldn't find config dir");
    path.push("BlockCommander");
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

// Configure for indavidual server (Name, Port, etc.)

#[derive(Serialize, Deserialize, Clone)]
pub struct ServerInfo
{
    pub name: String,
    pub version: String,
    pub loader: String,
    pub port: u16,
    pub rcon_port: u16,
    pub rcon_password: String,
}

#[derive(Serialize, Deserialize, Default)]
pub struct ServerList
{
    pub servers: Vec<ServerInfo>,
}

pub fn save_server_list(list: &ServerList)
{
    let path = central_list_path();
    if let Some(parent) = path.parent()
    {
        std::fs::create_dir_all(parent).expect("couldn't create config dir");
    }

    let contents = toml::to_string(list).expect("couldn't serialize server list");
    std::fs::write(&path, contents).expect("couldn't write server list")
}

fn central_list_path() -> PathBuf
{
    let mut path = dirs::config_dir().expect("couldn't find config dir");
    path.push("BlockCommander");
    path.push("server_list.toml");
    path
}

pub fn save_server_info(server_path: &std::path::Path, info: &ServerInfo)
{
    let path = server_path.join("server_info.toml");
    let contents = toml::to_string(info).expect("couldn't serialize server info");
    std::fs::write(&path, contents).expect("couldn't write server info");
}

pub fn load_server_list() -> ServerList
{
    let path = central_list_path();
    match std::fs::read_to_string(&path)
    {
        Ok(contents) => toml::from_str(&contents).unwrap_or_default(),
        Err(_) => ServerList::default(),
    }
}

// Find the URL for a specific Minecraft version in Mojang's version manifest and download it. Returns None if the version isn't found or if the request fails.

#[derive(Deserialize)]
struct VersionManifest
{
    versions: Vec<VersionEntry>,
}
#[derive(Deserialize)]
struct VersionEntry
{
    id: String,
    url: String,
}

#[derive(Deserialize)]
struct VersionDetail {
    downloads: Downloads,
}

#[derive(Deserialize)]
struct Downloads {
    server: DownloadInfo,
}

#[derive(Deserialize)]
struct DownloadInfo {
    url: String,
}

pub fn find_version_url(version: &str) -> Option<String> {
    let response = reqwest::blocking::get(
        "https://launchermeta.mojang.com/mc/game/version_manifest.json"
    ).ok()?;

    let manifest: VersionManifest = response.json().ok()?;

    manifest.versions
        .into_iter()
        .find(|v| v.id == version)
        .map(|v| v.url)
}

pub fn download_server_jar(version_url: &str, target_path: &std::path::Path) -> Result<(), String> {
    let response = reqwest::blocking::get(version_url)
        .map_err(|e| e.to_string())?;

    let detail: VersionDetail = response.json()
        .map_err(|e| e.to_string())?;

    let jar_url = detail.downloads.server.url;

    let jar_bytes = reqwest::blocking::get(&jar_url)
        .map_err(|e| e.to_string())?
        .bytes()
        .map_err(|e| e.to_string())?;

    std::fs::write(target_path, jar_bytes)
        .map_err(|e| e.to_string())?;

    Ok(())
}

//Get the servers directory from the config, or return None if it's not set.
pub fn get_dir() -> Option<String>
{
    let config = load_config();

    let dir = match config.servers_dir 
    {
        Some(d) => d,
        None => {
            println!("no servers directory set — run `blockcom config servers-dir <path>` first");
            return None;
        }
    };
    Some(dir)
}

// RCON and Game port avalability 
pub fn next_available_port(base: u16, list: &ServerList) -> u16
{
    let mut candidate = base;
    loop {
        let taken = list.servers.iter().any(|s| s.port == candidate);
        if !taken {
            return candidate;
        }
        candidate = candidate.checked_add(1).expect("no available game port");
    }
}

pub fn next_available_rcon_port(base: u16, list: &ServerList) -> u16
{
    let mut candidate = base;
    loop {
        let taken = list.servers.iter().any(|s| s.rcon_port == candidate);
        if !taken {
            return candidate;
        }
        candidate = candidate.checked_add(1).expect("no available RCON port");
    }
}