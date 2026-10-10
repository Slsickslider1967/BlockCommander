use crate::config::{get_dir, ServerInfo};
use std::process::Stdio;
use std::io::Write;

pub fn start(name: String)
{
    println!("Starting server '{}'", name);

    let dir = match get_dir() {
        Some(d) => d,
        None => return,
    };
    let server_path: std::path::PathBuf = std::path::Path::new(&dir).join(&name);

    if !server_path.is_dir()
    {
        println!("{} is not a server", name);
        return;
    }

    let info_path = server_path.join("server_info.toml");
    let info = match std::fs::read_to_string(&info_path)
        .ok()
        .and_then(|contents| toml::from_str::<ServerInfo>(&contents).ok())
    {
        Some(info) => info,
        None => {
            eprintln!("Failed to read server info for '{}'.", name);
            return;
        }
    };

    ensure_server_initialized(&name, &server_path);

    let _cmd = start_server(&name, &server_path, &info.loader, &info.start_file, info.required_java);

    println!("{} server started at {}", info.loader, &name);
    println!("With RCON port: {}", info.port);
}

pub fn first_time_server_properties(name: &String, server_path: &std::path::Path)
{
    println!("updating server.properties for '{}'", name);
    let info_path = server_path.join("server_info.toml");
    let info = match std::fs::read_to_string(&info_path)
        .ok()
        .and_then(|contents| toml::from_str::<ServerInfo>(&contents).ok())
    {
        Some(info) => info,
        None => {
            eprintln!("Failed to read server info for '{}'.", name);
            return;
        }
    };
    let port = info.port;

    let properties_path = server_path.join("server.properties");
    let properties = match std::fs::read_to_string(&properties_path)
    {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Failed to read server.properties: {}", e);
            return;
        }
    };

    let mut updated = Vec::new();
    let mut portfound = false;
    let mut namefound = false;
    let mut rconfound = false;
    let mut enablefound = false;

    for line in properties.lines()
    {
        if line.starts_with("server-port=")
        {
            updated.push(format!("server-port={}", port));
            portfound = true;
        }
        else if line.starts_with("rcon.port=")
        {
            updated.push(format!("rcon.port={}", info.rcon_port));
            rconfound = true;
        }
        else if line.starts_with("enable-rcon=")
        {
            updated.push("enable-rcon=true".to_string());
            enablefound = true;
        }
        else if line.starts_with("rcon.password=")
        {
            updated.push(format!("rcon.password={}", info.rcon_password));
        }
        else if line.starts_with("level-name=")
        {
            updated.push(format!("level-name={}", name));
            namefound = true;
        }
        else
        {
            updated.push(line.to_string());
        }
    }

    if portfound { println!("updated server-port to {}", port); }
    if namefound { println!("updated server-name to {}", name); }
    if rconfound { println!("updated rcon-port to {}", port); }
    if enablefound { println!("enabled rcon-port"); }

    let new_contents = updated.join("\n");

    if let Err(e) = std::fs::write(&properties_path, new_contents)
    {
        eprintln!("failed to write server.properties: {}", e);
        return;
    }

    println!("updated server.properties for '{}'", name);
}

pub fn find_java_home(major_version: u32) -> Option<String>
{
    let entries = std::fs::read_dir("/usr/lib/jvm").ok()?;

    for entry in entries.flatten()
    {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.contains(&major_version.to_string())
        {
            return Some(entry.path().to_string_lossy().to_string());
        }
    }

    None
}

pub fn start_server(name: &String, server_path: &std::path::Path, loader: &String, start_file: &String, required_java: u32) -> std::process::Child
{
    println!("starting server '{}' from '{}'", name, server_path.display());

    let info_path = server_path.join("server_info.toml");
    let info = std::fs::read_to_string(&info_path)
        .ok()
        .and_then(|contents| toml::from_str::<ServerInfo>(&contents).ok());

    let max_ram = info.as_ref().map(|i| i.max_ram_mb).unwrap_or(1024);
    let java_home = find_java_home(required_java);

    if loader == "Vanilla" || loader == "Fabric"
    {
        let jar_file_name: String = if start_file == "defaultfile"
        {
            match loader.as_str()
            {
                "Vanilla" => "server.jar".to_string(),
                "Fabric" => "fabric-server-launch.jar".to_string(),
                _ => "server.jar".to_string(),
            }
        }
        else
        {
            start_file.clone()
        };

        let mut cmd = std::process::Command::new("java");
        cmd.arg(format!("-Xmx{}M", max_ram))
            .arg(format!("-Xms{}M", max_ram))
            .arg("-jar")
            .arg(server_path.join(jar_file_name))
            .arg("nogui")
            .current_dir(server_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        if let Some(home) = &java_home
        {
            cmd.env("JAVA_HOME", home);
            cmd.env("PATH", format!("{}/bin:{}", home, std::env::var("PATH").unwrap_or_default()));
        }

        cmd.spawn().expect("failed to start server process")
    }
    else
    {
        let arguments_path = server_path.join("user_jvm_args.txt");
        let arguments = format!(
            "-Xmx{}M\n-Xms{}M\n-XX:+UseG1GC\n-XX:MaxGCPauseMillis=200\n-XX:+ParallelRefProcEnabled\n-XX:+DisableExplicitGC\n-XX:MaxTenuringThreshold=1\n-XX:SurvivorRatio=32\n-Djava.awt.headless=true\n",
            max_ram, max_ram
        );

        if let Err(e) = std::fs::write(&arguments_path, arguments)
        {
            eprintln!("failed to write settings file: {}", e);
        }

        let start_file_name = if start_file != "defaultfile"
        {
            start_file.clone()
        }
        else
        {
            "run.sh".to_string()
        };

        let mut cmd = std::process::Command::new(format!("./{}", start_file_name));
        cmd.current_dir(server_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        if let Some(home) = &java_home
        {
            cmd.env("JAVA_HOME", home);
            cmd.env("PATH", format!("{}/bin:{}", home, std::env::var("PATH").unwrap_or_default()));
        }

        cmd.spawn().expect("Failed to spawn forge server")
    }
}

pub fn stop_server(cmd: &mut std::process::Child)
{
    let stdin = cmd.stdin.as_mut().expect("no stdin handle");
    stdin.write_all(b"stop\n").expect("failed to write to stdin");

    println!("please do not touch the console, if nothing happens in over a minute, then reset...");
}

pub fn ensure_server_initialized(name: &String, server_path: &std::path::Path)
{
    let info_path = server_path.join("server_info.toml");
    let info = match std::fs::read_to_string(&info_path)
        .ok()
        .and_then(|contents| toml::from_str::<ServerInfo>(&contents).ok())
    {
        Some(info) => info,
        None => {
            eprintln!("Failed to read server info for '{}'.", name);
            return;
        }
    };

    if (!server_path.join("server.properties").exists() || info.loader != "Vanilla") && !info.has_been_started
    {
        let info_file = match std::fs::read_to_string(&info_path)
        {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Failed to read server_info_toml: {}", e);
                return;
            }
        };

        println!("first time start up detected...");
        println!("creating server files...");

        let mut updated = Vec::new();
        for line in info_file.lines()
        {
            if line.starts_with("has_been_started =")
            {
                updated.push("has_been_started = true".to_string());
            }
            else
            {
                updated.push(line.to_string());
            }
        }
        let new_contents = updated.join("\n");

        if let Err(e) = std::fs::write(&info_path, new_contents)
        {
            eprintln!("failed to write server_info.toml: {}", e);
            return;
        }

        let mut cmd = start_server(name, server_path, &info.loader, &info.start_file, info.required_java);
        println!("stopping server...");
        stop_server(&mut cmd);
        cmd.wait().expect("failed to wait on child");

        first_time_server_properties(&name, &server_path);
    }
}