use crate::config::ServerInfo;
use crate::config::get_dir;
use crate::config::save_server_list;
use crate::config::ServerList;

pub fn sync()
{
    println!("Syncing server info...");

    let dir = match get_dir()
    {
        Some(d) => d,
        None => return,
    };

    let mut list = ServerList::default();

    if let Ok(entries) = std::fs::read_dir(&dir)
    {
        for entry in entries.flatten()
        {
            if entry.path().is_dir()
            {
                let info_path = entry.path().join("server_info.toml");
                if let Ok(contents) = std::fs::read_to_string(&info_path)
                {
                    if let Ok(info) = toml::from_str::<ServerInfo>(&contents)
                    {
                        println!("Found server info: {}", info.name);
                        list.servers.push(info);
                    }
                }            
            }
        }
    }

    save_server_list(&list);

    println!("Synced server info: {}", list.servers.len());
}