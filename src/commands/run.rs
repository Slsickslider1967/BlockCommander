
use crate::config::get_dir;
use std::process::Stdio;
use std::io::Write;
use std::process::Command;

pub fn run(name: String)
{
    println!("Starting server '{}'", name);

    // load config to get servers directory
    let dir = match get_dir() {
        Some(d) => d,
        None => return,
    };
    let server_path: std::path::PathBuf  = std::path::Path::new(&dir).join(&name);
    let is_first_time = !server_path.join("server.properties").exists();

    if is_first_time
    {
        println!("first time start up detected...");
        println!("creating server files...");

        let mut cmd = run_server(&name, &server_path);
        end_server(&mut cmd);
        cmd.wait().expect("failed to wait on child");

        firsttime_server_properties(&name, &server_path);
    }

    
    let mut cmd = run_server(&name, &server_path);
}

fn firsttime_server_properties(name: &String, server_path: &std::path::Path)
{

}

fn run_server(name: &String, server_path: &std::path::Path) -> std::process::Child
{
    println!("running server '{}' from '{}'", name, server_path.display());

    // Run the server from server.jar
    std::process::Command::new("java")
        .arg("-jar")
        .arg(server_path.join("server.jar"))
        .arg("nogui")
        .current_dir(server_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to start server process")

}

fn end_server(cmd: &mut std::process::Child)
{
    let stdin = cmd.stdin.as_mut().expect("no stdin handle");
    stdin.write_all(b"stop\n").expect("failed to write to stdin");
}