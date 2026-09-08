use crate::config::get_dir;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use crossterm::event::{EventStream, Event, KeyCode};
use futures::StreamExt;


pub async fn start_gui(name: String)
{
    println!("Starting GUI...");

    //Starting the server process
    let mut cmd = match start_async_server(name).await {
        Some(c) => c,
        None => return,
    };

    println!("got the child process, PID handle ready");
    
    //Reading output from the server process
    let stdout = cmd.stdout.take().expect("no stdout handle");
    let mut lines = BufReader::new(stdout).lines();
    let mut events = EventStream::new();

    crossterm::terminal::enable_raw_mode().expect("failed to enable raw mode");

    loop {
        tokio::select! {
            line = lines.next_line() => {
                match line {
                    Ok(Some(l)) => println!("[server] {}\r\n", l),
                    Ok(None) => break,
                    Err(e) => { println!("error reading line: {}", e); break; }
                }
            }

            // Handling user input events
            event = events.next() => {
                if let Some(Ok(Event::Key(key))) = event {
                    if key.code == KeyCode::Char('q') {
                        break;
                    }
                }
            }
        }
    }

    crossterm::terminal::disable_raw_mode().expect("failed to disable raw mode");


    cmd.wait().await.expect("failed to wait on child");
}

async fn start_async_server(name: String) -> Option<tokio::process::Child>
{
    let dir = match get_dir() {
        Some(d) => d,
        None => return None,
    };
    let server_path = std::path::Path::new(&dir).join(&name);

    let cmd = tokio::process::Command::new("java")
        .arg("-jar")
        .arg(server_path.join("server.jar"))
        .current_dir(&server_path)
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to start server process");

    println!("server spawned!");
    Some(cmd)
}