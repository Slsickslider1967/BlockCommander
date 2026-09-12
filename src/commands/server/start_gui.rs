use crate::config::get_dir;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use crossterm::event::{EventStream, Event, KeyCode};
use crossterm::terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::execute;
use futures::StreamExt;
use ratatui::{Terminal, backend::CrosstermBackend, widgets::{Block, Borders, Paragraph}};
use super::start::{ensure_server_initialized};

pub async fn start_gui(name: String)
{
    println!("Starting GUI...");

    // load config to get servers directory
    let dir = match get_dir() {
        Some(d) => d,
        None => return,
    };
    let server_path: std::path::PathBuf  = std::path::Path::new(&dir).join(&name);
    let name_clone = name.clone();
    let path_clone = server_path.clone();
    tokio::task::spawn_blocking(move || {
        ensure_server_initialized(&name_clone, &path_clone);
    }).await.expect("first-time setup task panicked");

    //Starting the server process
    let mut cmd = match start_async_server(name).await {
        Some(c) => c,
        None => return,
    };

    println!("got the child process, PID handle ready");
    
    //Reading output from the server process
    let mut stdin = cmd.stdin.take().expect("no stdin handle");
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
    run_gui_loop(
        &mut terminal,
        &mut lines,
        &mut err_lines,
        &mut events,
        &mut log_lines,
        &mut stdin,
        &mut cmd
    ).await;



    disable_raw_mode().expect("failed to disable raw mode");
    execute!(std::io::stdout(), LeaveAlternateScreen).expect("failed to leave alternate screen");

    match tokio::time::timeout(std::time::Duration::from_secs(30), cmd.wait()).await {
        Ok(Ok(_)) => {}
        Ok(Err(error)) => eprintln!("failed to wait on child: {error}"),
        Err(_) => {
            eprintln!("server did not exit after 30 seconds; terminating it");
            let _ = cmd.kill().await;
            let _ = cmd.wait().await;
        }
    }
}

async fn run_gui_loop
(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    lines: &mut tokio::io::Lines<BufReader<tokio::process::ChildStdout>>,
    err_lines: &mut tokio::io::Lines<BufReader<tokio::process::ChildStderr>>,
    events: &mut EventStream,
    log_lines: &mut Vec<String>,
    stdin: &mut tokio::process::ChildStdin,
    child: &mut tokio::process::Child
)
{

    // Initialize scroll_offset and auto_scroll variables
    let mut scroll_offset: u16 = 0; 
    let mut auto_scroll = true; // Enable auto-scrolling by default

    loop {
        tokio::select! {
            line = lines.next_line() => {
                if let Ok(Some(line)) = line {
                    log_lines.push(line);
                }
            }

            line = err_lines.next_line() => {
                if let Ok(Some(line)) = line {
                    log_lines.push(format!("[warn] {}", line));
                }
            }

            event = events.next() => {
                if let Some(Ok(Event::Key(key))) = event {
                    // if key.code == KeyCode::Char('q') {
                    //     break;
                    // }
                    match key.code 
                    {
                        KeyCode::Char('q') => 
                        {
                            stdin.write_all(&b"stop\n"[..]).await.expect("failed to write to stdin");

                            let shutdown = tokio::time::timeout(
                                std::time::Duration::from_secs(10),
                                async {
                                    let mut child_exited = false;
                                    let mut stdout_closed = false;
                                    let mut stderr_closed = false;

                                    while !child_exited || !stdout_closed || !stderr_closed {
                                        tokio::select! {
                                            result = child.wait(), if !child_exited => {
                                                child_exited = result.is_ok();
                                            }
                                            line = lines.next_line(), if !stdout_closed => {
                                                match line {
                                                    Ok(Some(line)) => log_lines.push(line),
                                                    _ => stdout_closed = true,
                                                }
                                            }
                                            line = err_lines.next_line(), if !stderr_closed => {
                                                match line {
                                                    Ok(Some(line)) => log_lines.push(format!("[warn] {}", line)),
                                                    _ => stderr_closed = true,
                                                }
                                            }
                                        }

                                        if auto_scroll {
                                            let visible_height = terminal.size().expect("failed to get size").height.saturating_sub(2);
                                            scroll_offset = (log_lines.len() as u16).saturating_sub(visible_height);
                                        }
                                        draw_terminal(terminal, log_lines, scroll_offset).await;
                                    }
                                }
                            ).await;

                            if shutdown.is_err() {
                                eprintln!("server did not finish shutting down after 10 seconds; terminating it");
                                let _ = child.kill().await;
                                let _ = child.wait().await;
                            }
                            break;
                        }
                        KeyCode::Up => 
                        {
                            auto_scroll = false;
                            scroll_offset = scroll_offset.saturating_sub(1);
                        }
                        KeyCode::Down =>
                        {
                            scroll_offset = scroll_offset.saturating_add(1);
                        }
                        _ => {} // Ignore other keys
                    }
                }
            }
        }

        if auto_scroll 
        {
            let visible_height = terminal.size().expect("failed to get size").height.saturating_sub(2) as u16;
            scroll_offset = (log_lines.len() as u16).saturating_sub(visible_height);
        }

        // Call draw terminal to update the UI with the latest log lines
        draw_terminal(terminal, log_lines, scroll_offset).await;
    }

}

async fn draw_terminal(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>, log_lines: &Vec<String>, scroll_offset: u16)
{
    // Initialize scroll_offset and auto_scroll variables
    let mut auto_scroll = true; // Enable auto-scrolling by default

    terminal.draw(|frame| {

        //Area 
        let area = frame.area();

        //Text 
        let text = log_lines.join("\n");
        let block = Block::default().title("BlockCommander").borders(Borders::ALL);

        // Create a paragraph widget with the log lines and render its
        let paragraph = Paragraph::new(text)
            .block(block)
            .scroll((scroll_offset, 0));

        frame.render_widget(paragraph, frame.area());
    }).expect("failed to draw");
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
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to start server process");

    println!("server spawned!");
    Some(cmd)
}