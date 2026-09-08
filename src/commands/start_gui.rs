use crate::config::get_dir;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use crossterm::event::{EventStream, Event, KeyCode};
use crossterm::terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::execute;
use futures::StreamExt;
use ratatui::{Terminal, backend::CrosstermBackend, widgets::{Block, Borders, Paragraph}};



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
    let stderr = cmd.stderr.take().expect("no stderr handle");

    // Merging stdout and stderr into a single stream
    let mut lines = BufReader::new(stdout).lines();
    let mut err_lines = BufReader::new(stderr).lines();
    let mut events = EventStream::new();

    //gui setup
    enable_raw_mode().expect("failed to enable raw mode");
    let mut screen = std::io::stdout();
    execute!(screen, EnterAlternateScreen).expect("failed to enter alternate screen");
    let backend = CrosstermBackend::new(screen);
    let mut terminal = Terminal::new(backend).expect("failed to create terminal");

    let mut log_lines: Vec<String> = Vec::new();

    // Main loop to read server output and handle user input
loop {
    tokio::select! {
        line = lines.next_line() => {
            match line {
                Ok(Some(l)) => log_lines.push(l),
                Ok(None) => break,
                Err(e) => { log_lines.push(format!("error reading line: {}", e)); break; }
            }
        }

        line = err_lines.next_line() => {
            match line {
                Ok(Some(l)) => log_lines.push(format!("[warn] {}", l)),
                Ok(None) => {}, // stderr closing isn't a reason to exit — stdout closing is what signals server exit
                Err(e) => log_lines.push(format!("error reading stderr: {}", e)),
            }
        }

        event = events.next() => {
            if let Some(Ok(Event::Key(key))) = event {
                if key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }
    }

        terminal.draw(|frame| {
            let text = log_lines.join("\n");
            let block = Block::default().title("BlockCommander").borders(Borders::ALL);
            let paragraph = Paragraph::new(text).block(block);
            frame.render_widget(paragraph, frame.area());
        }).expect("failed to draw");
    }


    disable_raw_mode().expect("failed to disable raw mode");
    execute!(std::io::stdout(), LeaveAlternateScreen).expect("failed to leave alternate screen");

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
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to start server process");

    println!("server spawned!");
    Some(cmd)
}