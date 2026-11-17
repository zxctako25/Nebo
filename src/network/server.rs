use log::{error, info};
use std::collections::HashMap;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use super::client::handle_client;
use crate::config::ProxyConfig;
use crate::forwarding::ForwardingManager;
use crate::motd::MotdService;
use crate::limbo::texts::LimboTexts;
use crate::protocol::registry::ProtocolRegistry;
use crate::security::AntiBot;

pub struct ProxyServer {
    pub config: Arc<ProxyConfig>,
    pub forwarding: Arc<ForwardingManager>,
    pub active_connections: AtomicI32,
    pub active_players: AtomicI32,
    pub active_limbo_players: AtomicI32,
    pub anti_bot: Arc<AntiBot>,
    pub motd: MotdService,
    pub limbo_texts: LimboTexts,
    pub per_ip: Mutex<HashMap<String, i32>>,
    shutdown_tx: tokio::sync::watch::Sender<bool>,
}

impl ProxyServer {
    pub async fn new(config: Arc<ProxyConfig>) -> anyhow::Result<Arc<Self>> {
        let forwarding = Arc::new(ForwardingManager::new(config.clone())?);
        let anti_bot = Arc::new(AntiBot::load(&config.data_dir).await?);
        let motd = MotdService::load(&config.data_dir)?;
        let limbo_texts = LimboTexts::load(&config.data_dir);
        let (shutdown_tx, _) = tokio::sync::watch::channel(false);
        Ok(Arc::new(Self {
            config,
            forwarding,
            active_connections: AtomicI32::new(0),
            active_players: AtomicI32::new(0),
            active_limbo_players: AtomicI32::new(0),
            anti_bot,
            motd,
            limbo_texts,
            per_ip: Mutex::new(HashMap::new()),
            shutdown_tx,
        }))
    }

    pub async fn start(self: &Arc<Self>) -> anyhow::Result<JoinHandle<()>> {
        let addr = format!("{}:{}", self.config.bind_address, self.config.bind_port);
        let listener = TcpListener::bind(&addr).await?;
        info!("Proxy started on {}", addr);
        info!(
            "Supported protocols: {}",
            ProtocolRegistry::supported_list()
        );

        let this = self.clone();
        let mut shutdown_rx = self.shutdown_tx.subscribe();

        let handle = tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = shutdown_rx.changed() => {
                        if *shutdown_rx.borrow() { info!("Server loop shutting down"); break; }
                    }
                    accept = listener.accept() => {
                        match accept {
                            Ok((stream, peer)) => {
                                let server = this.clone();
                                let ip = peer.ip().to_string();
                                tokio::spawn(async move {
                                    if let Err(e) = handle_client(server, stream, ip).await {
                                        log::warn!("Client connection failed: {:#}", e);
                                    }
                                });
                            }
                            Err(e) => error!("Accept error: {}", e),
                        }
                    }
                }
            }
        });

        Ok(handle)
    }

    pub async fn try_register(&self, address: &str) -> bool {
        if self.config.max_connections_per_ip <= 0 {
            return self
                .active_connections
                .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |active| {
                    (active < self.config.max_total_connections).then_some(active + 1)
                })
                .is_ok();
        }

        let mut by_ip = self.per_ip.lock().await;
        let count = by_ip.entry(address.to_owned()).or_default();
        if *count >= self.config.max_connections_per_ip
            || self.active_connections.load(Ordering::Relaxed) >= self.config.max_total_connections
        {
            return false;
        }
        *count += 1;
        self.active_connections.fetch_add(1, Ordering::Relaxed);
        true
    }

    pub async fn unregister(&self, address: &str) {
        if self.config.max_connections_per_ip > 0 {
            let mut by_ip = self.per_ip.lock().await;
            if let Some(count) = by_ip.get_mut(address) {
                *count -= 1;
                if *count == 0 {
                    by_ip.remove(address);
                }
            }
        }
        self.active_connections.fetch_sub(1, Ordering::Relaxed);
    }

    pub async fn stop(&self) {
        info!("Stopping proxy...");
        let _ = self.shutdown_tx.send(true);
        info!("Proxy stopped.");
    }

    pub fn list_players(&self) {
        println!(
            "Active connections: {}",
            self.active_connections.load(Ordering::SeqCst)
        );
    }

    pub fn reload(&self) {
        info!("Reload not implemented yet (TODO)");
    }
}
