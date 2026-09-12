use rcon::Connection;
use tokio::net::TcpStream;

pub async fn list_active()
{
    let list = crate::config::load_server_list();

    if list.servers.is_empty() {
        println!("no servers configured");
        return;
    }

    let mut any_active = false;

    for server in &list.servers
    {
        let address = format!("127.0.0.1:{}", server.rcon_port);

        let result = Connection::<TcpStream>::builder()
            .enable_minecraft_quirks(true)
            .connect(&address, &server.rcon_password)
            .await;

        match result {
            Ok(_) => {
                println!("{} - active (port {})", server.name, server.port);
                any_active = true;
            }
            Err(_) => {
                // connection refused/timed out — treat as not running
            }
        }
    }

    if !any_active {
        println!("no active servers");
    }
}