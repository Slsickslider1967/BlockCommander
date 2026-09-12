use crate::config::get_dir;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use crossterm::event::{EventStream, Event, KeyCode};
use crossterm::terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::execute;
use futures::StreamExt;
use ratatui::{Terminal, backend::CrosstermBackend, widgets::{Block, Borders, Paragraph}, layout::{Layout, Direction, Constraint}};
use sysinfo::{ProcessesToUpdate, System, Pid};
use super::start::{ensure_server_initialized};

pub async fn start_gui(name: String)
{
    println!("Starting GUI...");

    let dir = match get_dir() {
        Some(d) => d,
        None => return,
    };
    let server_path: std::path::PathBuf = std::path::Path::new(&dir).join(&name);
    let name_clone = name.clone();
    let path_clone = server_path.clone();
    tokio::task::spawn_blocking(move || {
        ensure_server_initialized(&name_clone, &path_clone);
    }).await.expect("first-time setup task panicked");

    let mut cmd = match start_async_server(name).await {
        Some(c) => c,
        None => return,
    };

    let pid = Pid::from_u32(cmd.id().expect("no PID — process already exited"));

    let mut stdin = cmd.stdin.take().expect("no stdin handle");
    let stdout = cmd.stdout.take().expect("no stdout handle");
    let stderr = cmd.stderr.take().expect("no stderr handle");

    let mut lines = BufReader::new(stdout).lines();
    let mut err_lines = BufReader::new(stderr).lines();
    let mut events = EventStream::new();

    enable_raw_mode().expect("failed to enable raw mode");
    let mut screen = std::io::stdout();
    execute!(screen, EnterAlternateScreen).expect("failed to enter alternate screen");
    let backend = CrosstermBackend::new(screen);
    let mut terminal = Terminal::new(backend).expect("failed to create terminal");

    let mut log_lines: Vec<String> = Vec::new();

    run_gui_loop(
        &mut terminal,
        &mut lines,
        &mut err_lines,
        &mut events,
        &mut log_lines,
        &mut stdin,
        &mut cmd,
        pid,
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
    child: &mut tokio::process::Child,
    pid: Pid,
)
{
    let mut scroll_offset: u16 = 0;
    let mut auto_scroll = true;

    let mut sys = System::new_all();
    let mut cpu_percent: f32 = 0.0;
    let mut ram_used_mb: f32 = 0.0;
    let total_cores = sys.cpus().len().max(1) as f32;

    let mut stats_timer = tokio::time::interval(std::time::Duration::from_secs(1));
    let mut input_buffer = String::new();

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

            _ = stats_timer.tick() => {
                sys.refresh_processes(ProcessesToUpdate::Some(&[pid]), true);
                if let Some(process) = sys.process(pid) {
                    cpu_percent = process.cpu_usage() / total_cores;
                    ram_used_mb = process.memory() as f32 / (1024.0 * 1024.0);
                }
            }

            event = events.next() => {
                if let Some(Ok(Event::Key(key))) = event {
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
                                            let visible_height = terminal.size().expect("failed to get size").height.saturating_sub(5);
                                            scroll_offset = (log_lines.len() as u16).saturating_sub(visible_height);
                                        }
                                        draw_terminal(terminal, log_lines, scroll_offset, cpu_percent, ram_used_mb, &input_buffer).await;
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
                        KeyCode::Char(c) =>
                        {
                            input_buffer.push(c);
                        }
                        KeyCode::Backspace =>
                        {
                            input_buffer.pop();
                        }
                        KeyCode::Enter =>
                        {
                            if !input_buffer.is_empty() {
                                let command_line = format!("{}\n", input_buffer);
                                stdin.write_all(command_line.as_bytes()).await.expect("failed to write to stdin");
                                log_lines.push(format!("> {}", input_buffer));
                                input_buffer.clear();
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        if auto_scroll
        {
            let visible_height = terminal.size().expect("failed to get size").height.saturating_sub(5) as u16;
            scroll_offset = (log_lines.len() as u16).saturating_sub(visible_height);
        }

        draw_terminal(terminal, log_lines, scroll_offset, cpu_percent, ram_used_mb, &input_buffer).await;
    }
}

async fn draw_terminal(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    log_lines: &Vec<String>,
    scroll_offset: u16,
    cpu_percent: f32,
    ram_used_mb: f32,
    input_buffer: &str,
)
{
    terminal.draw(|frame| {
        let area = frame.area();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // stats
                Constraint::Min(0),    // logs
                Constraint::Length(3), // input box
            ])
            .split(area);

        let stats_text = format!("CPU: {:.1}%   RAM: {:.0} MB", cpu_percent, ram_used_mb);
        let stats_widget = Paragraph::new(stats_text)
            .block(Block::default().title("Stats").borders(Borders::ALL));
        frame.render_widget(stats_widget, chunks[0]);

        let text = log_lines.join("\n");
        let log_widget = Paragraph::new(text)
            .block(Block::default().title("BlockCommander").borders(Borders::ALL))
            .scroll((scroll_offset, 0));
        frame.render_widget(log_widget, chunks[1]);

        let input_widget = Paragraph::new(input_buffer)
            .block(Block::default().title("Command (Enter to send)").borders(Borders::ALL));
        frame.render_widget(input_widget, chunks[2]);
    }).expect("failed to draw");
}

async fn start_async_server(name: String) -> Option<tokio::process::Child>
{
    let dir = match get_dir() {
        Some(d) => d,
        None => return None,
    };
    let server_path = std::path::Path::new(&dir).join(&name);

    let info_path = server_path.join("server_info.toml");
    let info = std::fs::read_to_string(&info_path)
        .ok()
        .and_then(|contents| toml::from_str::<crate::config::ServerInfo>(&contents).ok());
    let max_ram = info.as_ref().map(|i| i.max_ram_mb).filter(|&m| m > 0).unwrap_or(1024);

    let cmd = tokio::process::Command::new("java")
        .arg(format!("-Xmx{}M", max_ram))
        .arg(format!("-Xms{}M", max_ram))
        .arg("-jar")
        .arg(server_path.join("server.jar"))
        .arg("nogui")
        .current_dir(&server_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to start server process");

    Some(cmd)
}