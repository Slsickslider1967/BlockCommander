use std::path::Path;
use std::fs;
use std::io::Write;
use clap::ValueEnum;
use crate::Loader;
use crate::config::*;

const KNOWN_LAUNCHERS: [&str; 3] = ["server.jar", "fabric-server-launch.jar", "run.sh"];

pub fn add_extern_server(extern_dir: String)
{
    let extern_path = match std::fs::canonicalize(&extern_dir)
    {
        Ok(p) if p.is_dir() => p,
        _ => {
            println!("external server folder does not exist");
            return;
        }
    };

    let dir = match get_dir() {
        Some(d) => d,
        None => return,
    };

    let name = prompt("enter the server name");
    let version = prompt("enter the server minecraft version");
    let loader = prompt_loader();

    let destination_path = Path::new(&dir).join(&name);
    if destination_path.exists()
    {
        println!("a server named '{}' already exists", name);
        return;
    }

    if destination_path.starts_with(&extern_path)
    {
        println!("the servers directory can't be inside the server you're copying");
        return;
    }

    println!("copying server files, this can take a while for big worlds...");
    if let Err(e) = copy_dir_recursive(&extern_path, &destination_path)
    {
        println!("failed to copy server: {}", e);
        let _ = fs::remove_dir_all(&destination_path);
        return;
    }

    if let Some(setup_script) = find_setup_script(&destination_path)
    {
        run_setup_script(&destination_path, &setup_script);
    }

    let start_file = match find_start_file(&destination_path)
    {
        Some(f) => f,
        None => {
            println!("couldn't find a known start file (server.jar, fabric-server-launch.jar, run.sh)");
            let _ = fs::remove_dir_all(&destination_path);
            return;
        }
    };
    println!("start file found: '{}'", start_file);

    let eula_path = destination_path.join("eula.txt");
    let mut input = String::new();

    print!("Do you accept the Minecraft EULA? (https://account.mojang.com/documents/minecraft_eula) (y/n): ");
    std::io::stdout().flush().expect("Failed to flush stdout");
    std::io::stdin().read_line(&mut input).expect("Failed to read input");

    if input.trim().to_lowercase() != "y"
    {
        println!("EULA not accepted. Server add aborted.");
        let _ = fs::remove_dir_all(&destination_path);
        return;
    }

    if let Err(e) = std::fs::write(&eula_path, "eula=true\n")
    {
        println!("failed to write eula.txt: {}", e);
        let _ = fs::remove_dir_all(&destination_path);
        return;
    }

    println!("adding server to list");

    make_executable(&destination_path.join(&start_file));

    let config = load_config();
    let mut list = load_server_list();
    let port = next_available_port(config.port.unwrap_or(25565), &list);
    let rcon_port = next_available_rcon_port(config.rcon_port.unwrap_or(25575), &list);

    let info = ServerInfo
    {
        name,
        version,
        loader: loader.to_string(),

        port,
        rcon_port,
        rcon_password: config.rcon_password.unwrap_or_else(|| "defaultpassword".to_string()),
        max_ram_mb: if config.max_ram_mb > 0 { config.max_ram_mb } else { 1024 },

        has_been_started: true,
        start_file,
    };

    save_server_info(&destination_path, &info);
    list.servers.push(info);
    save_server_list(&list);
}

fn find_start_file(dir: &Path) -> Option<String>
{
    for name in KNOWN_LAUNCHERS
    {
        if dir.join(name).exists()
        {
            return Some(name.to_string());
        }
    }
    None
}

fn find_setup_script(dir: &Path) -> Option<String>
{
    let entries = std::fs::read_dir(dir).ok()?;

    for entry in entries.flatten()
    {
        let file_name = entry.file_name();
        let Some(file_name) = file_name.to_str() else { continue; };

        let is_script_or_jar = file_name.ends_with(".sh") || file_name.ends_with(".jar");
        let is_known_launcher = KNOWN_LAUNCHERS.contains(&file_name);

        if is_script_or_jar && !is_known_launcher
        {
            return Some(file_name.to_string());
        }
    }

    None
}

fn run_setup_script(server_path: &Path, script_name: &str)
{
    let script_path = server_path.join(script_name);

    if !script_path.exists()
    {
        println!("'{}' doesn't exist, skipping setup step", script_name);
        return;
    }

    make_executable(&script_path);

    println!("running setup script...");
    let mut child = std::process::Command::new(format!("./{}", script_name))
        .current_dir(server_path)
        .spawn()
        .expect("Failed to spawn setup script");

    let eula_path = server_path.join("eula.txt");
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(300);

    loop
    {
        if eula_path.exists()
        {
            println!("setup finished, eula.txt found");
            let _ = child.kill();
            let _ = child.wait();
            return;
        }

        if let Ok(Some(status)) = child.try_wait()
        {
            println!("setup script exited on its own with status {:?}", status);
            return;
        }

        if std::time::Instant::now() >= deadline
        {
            println!("setup script didn't finish within 5 minutes, giving up");
            let _ = child.kill();
            let _ = child.wait();
            return;
        }

        std::thread::sleep(std::time::Duration::from_millis(500));
    }
}

fn make_executable(path: &Path)
{
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = std::fs::metadata(path)
        {
            let mut perms = metadata.permissions();
            perms.set_mode(0o755);
            let _ = std::fs::set_permissions(path, perms);
        }
    }
}

fn prompt(label: &str) -> String
{
    print!("{}: ", label);
    std::io::stdout().flush().expect("Failed to flush stdout");

    let mut input = String::new();
    std::io::stdin().read_line(&mut input).expect("Failed to read input");
    input.trim().to_string()
}

fn prompt_loader() -> Loader
{
    loop
    {
        let input = prompt("enter the server loader (vanilla, fabric, forge, neoforge)");

        match Loader::from_str(&input, true)
        {
            Ok(loader) => return loader,
            Err(_) => println!("'{}' isn't a valid loader, try again.", input),
        }
    }
}

impl std::fmt::Display for Loader
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result
    {
        let name = match self
        {
            Loader::Vanilla => "Vanilla",
            Loader::Fabric => "Fabric",
            Loader::Forge => "Forge",
            Loader::Neoforge => "NeoForge",
        };
        write!(f, "{}", name)
    }
}