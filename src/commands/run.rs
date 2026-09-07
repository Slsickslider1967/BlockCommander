
use crate::config::get_dir;

pub fn run(name: String)
{
    println!("Starting server '{}'", name);

    // load config to get servers directory
    let dir = match get_dir() {
        Some(d) => d,
        None => return,
    };
    let server_path = std::path::Path::new(&dir).join(&name);
    let is_first_time = !server_path.join("server.properties").exists();

    if is_first_time
    {
        println!("first time start up detected...");
        println!("creating server files...");
    }

    

}