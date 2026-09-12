# BlockCommander

A terminal-based Minecraft server management and creation tool for Linux.

BlockCommander takes the hassle out of spinning up and running Minecraft servers from the command line. Instead of manually creating folders, downloading jars, editing `server.properties`, and juggling multiple terminal windows, you get a single tool that handles creation, configuration, and live management — including a lightweight in-terminal dashboard for watching a server run.

## Why this exists

Most Minecraft server tools are either full GUI applications (heavy, and often Windows-first) or a pile of manual shell scripts you have to maintain yourself. There wasn't a friendly, dedicated CLI for managing *multiple* servers side by side — creating them, configuring them, starting and stopping them independently, and checking on them at a glance. BlockCommander is an attempt to fill that gap.

## Features

- **Guided server creation** — pick a name and a Minecraft version, and BlockCommander fetches the correct server jar directly from Mojang's official version manifest.
- **EULA handling built in** — prompts for acceptance during creation instead of leaving you to edit a text file by hand.
- **Run many servers side by side** — each server gets its own folder, port, and RCON port, so nothing collides.
- **Two ways to run a server**:
  - **Background mode** — start a server and get your terminal back immediately, so you can launch several without babysitting any of them.
  - **Dashboard mode** — a live, scrollable terminal UI showing the server's log output, CPU and RAM usage, and a command box for sending input straight to the console.
- **Remote-friendly shutdown** — stop any running server by name, from a completely separate terminal session, using RCON under the hood.
- **Central server list** — one command shows every server you've created, and a sync command can rebuild that list from disk if anything ever gets out of step.

## Installation

Requires [Rust](https://www.rust-lang.org/tools/install) and a working Java installation (for actually running Minecraft servers).

```bash
git clone <your-repo-url>
cd BlockCommander
cargo build --release
```

The compiled binary will be at `target/release/BlockCommander`. Copy it somewhere on your `PATH` (e.g. `/usr/local/bin`) to run it as `BlockCommander` from anywhere.

## Getting started

```bash
BlockCommander config servers-dir /path/to/servers
BlockCommander create MyWorld 1.20.1
BlockCommander MyWorld start        # or: start-gui, for the live dashboard
BlockCommander MyWorld stop         # stops it from anywhere, no need to be attached
```

## Commands

```
BlockCommander
├── create <name> <version> [--loader <vanilla|fabric|forge|neoforge>]   Create a new server (defaults to vanilla)
├── delete <name>                                                       Delete a server and remove it from the list
├── list                                                                List every known server
│
├── <server-name>
│   ├── start                                                           Start in the background, return the terminal immediately
│   ├── start-gui                                                       Start with the live dashboard (logs, CPU/RAM, command box)
│   ├── stop                                                            Stop a running server remotely via RCON
│   ├── port <port>                                                     Change the server's game port
│   ├── rcon-port <port>                                                Change the server's RCON port
│   └── rcon-password <password>                                        Change the server's RCON password
│
└── config
    ├── servers-dir <dir>                                               Set the folder servers are created and stored in
    ├── defualt-port <port>                                             Set the default game port for new servers
    ├── default-rcon-port <port>                                        Set the default RCON port for new servers
    └── sync                                                            Rebuild the central server list from what's on disk
```

## Dashboard controls

While in `start-gui`:

| Key | Action |
|---|---|
| `↑` / `↓` | Scroll the log view up or down. |
| Type anything, then `Enter` | Send that text as a command to the server console. |
| `q` (with the command box empty) | Gracefully stop the server and exit the dashboard. |

## How it works

- Each server has its own folder containing `server.jar`, `server.properties`, and a `server_info.toml` describing its configuration.
- A central `server_list.toml` (stored in your config directory) caches every server's info for fast lookups — this is what `list` reads from, and what `config sync` rebuilds.
- RCON is enabled automatically on every created server, which is what allows `stop` (and future status checks) to reach an already-running server from a brand new command invocation.

## Status

This is an actively evolving personal project. Vanilla server support is fully working end to end; Fabric, Forge, and NeoForge support are planned but not yet implemented.