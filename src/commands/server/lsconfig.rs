use crate::config::get_dir;

pub fn list_server_config(server_name: String)
{
    // Getting server path
    let dir = match get_dir() {
        Some(d) => d,
        None => return,
    };
    let server_path: std::path::PathBuf = std::path::Path::new(&dir).join(&server_name);

    // Reading server properties
    let properties_path = server_path.join("server.properties");
    let properties = match std::fs::read_to_string(&properties_path)
    {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Failed to read server.properties: {}", e);
            return;
        }
    };

    // printing server prop
    for line in properties.lines()
    {
        if (line.contains("="))
        {
            let column_width = 33;
            let smaller_width = 10;

            if let Some((key, value)) = line.split_once('=')
            {
                println!("{:<w$} = {:>ws$}", key.trim(), value.trim(), w = column_width, ws = smaller_width);
            }

        }
    }

    for line in properties.lines()
    {

    }
}