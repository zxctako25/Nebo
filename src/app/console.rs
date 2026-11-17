use std::sync::Arc;

use tokio::io::{AsyncBufReadExt, BufReader};

use crate::network::server::ProxyServer;

pub async fn start_console(server: Arc<ProxyServer>) {
    let stdin = tokio::io::stdin();
    let mut reader = BufReader::new(stdin).lines();

    while let Ok(Some(line)) = reader.next_line().await {
        let cmd = line.trim().to_lowercase();
        match cmd.as_str() {
            "end" | "stop" | "exit" => {
                log::info!("Stop by console command");
                server.stop().await;
                return;
            }
            "list" => server.list_players(),
            "reload" => server.reload(),
            "help" => {
                println!("Commands:");
                println!("  end    - stop proxy");
                println!("  list   - list connected players");
                println!("  reload - reload config (TODO)");
                println!("  help   - this help");
            }
            "" => {}
            _ => println!("Unknown command: {}", cmd),
        }
    }
}
