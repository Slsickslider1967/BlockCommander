use rcon::Connection;
use tokio::net::TcpStream;

pub async fn stop(name: String)
{
    let list = crate::config::load_server_list();

    let info = match list.servers.iter().find(|s| s.name == name)
    {
        Some(info) => info,
        None => {
            eprintln!("no server named '{}' found", name);
            return;
        }
    };

    println!("stopping server '{}' with RCON, port '{}'", name, info.rcon_port);

    let address = format!("127.0.0.1:{}", info.rcon_port);

    let mut conn = match Connection::<TcpStream>::builder()
        .enable_minecraft_quirks(true)
        .connect(&address, &info.rcon_password)
        .await
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!("failed to connect to '{}' via RCON: {}", name, e);
            return;
        }
    };

    match conn.cmd("stop").await
    {
        Ok(response) => println!("server '{}' stopped: {}", name, response),
        Err(e) => eprintln!("failed to send stop command: {}", e),
    }
}