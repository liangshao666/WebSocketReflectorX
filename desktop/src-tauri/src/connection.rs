use std::collections::HashMap;
use std::sync::Arc;

use once_cell::sync::Lazy;
use serde::Deserialize;
use serde::Serialize;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::sync::RwLock;
use url::Url;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Connection {
    id: String,
    url: String,
    port: u16,
    latency: u32,
}

pub struct ConnectionManager {
    pub connections: HashMap<String, Connection>,
    pub dead_connections: HashMap<String, Connection>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            connections: HashMap::new(),
            dead_connections: HashMap::new(),
        }
    }
}

static CONNECTION_MANAGER: Lazy<Arc<RwLock<ConnectionManager>>> =
    Lazy::new(|| Arc::new(RwLock::new(ConnectionManager::new())));

pub async fn add_ws_connection(addr: impl AsRef<str>) -> anyhow::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let port = listener.local_addr()?.port();
    let url = Url::parse(addr.as_ref())?;
    let id = format!("{}#{}", url.host_str().unwrap(), port);
    let conn = Connection {
        id: id.clone(),
        url: addr.as_ref().to_string(),
        port,
        latency: 0,
    };
    CONNECTION_MANAGER
        .write()
        .await
        .connections
        .insert(id.clone(), conn.clone());
    let addr = addr.as_ref().to_string();
    tokio::spawn(async move {
        loop {
            let (tcp, _) = match listener.accept().await {
                Ok(tup) => tup,
                Err(err) => {
                    println!("failed to accept tcp connection: {:?}", err);
                    let mut cm = CONNECTION_MANAGER.write().await;
                    cm.connections.remove(&id);
                    cm.dead_connections.insert(id, conn.clone());
                    return;
                }
            };
            let id = id.clone();
            let addr = addr.clone();
            let conn = conn.clone();
            tokio::spawn(async move {
                match proxy_ws_addr(addr, tcp).await {
                    Ok(()) => (),
                    Err(err) => {
                        println!("failed to proxy tcp connection: {:?}", err);
                        let mut cm = CONNECTION_MANAGER.write().await;
                        cm.connections.remove(&id);
                        cm.dead_connections.insert(id, conn);
                        return;
                    }
                }
            });
        }
    });
    Ok(())
}

pub async fn remove_ws_connection(id: impl AsRef<str>) -> anyhow::Result<()> {
    let mut cm = CONNECTION_MANAGER.write().await;
    cm.connections.remove(id.as_ref());
    cm.dead_connections.remove(id.as_ref());
    Ok(())
}

pub async fn get_alive_ws_connections() -> anyhow::Result<Vec<Connection>> {
    let mut conns = Vec::new();
    for (_, conn) in CONNECTION_MANAGER.read().await.connections.iter() {
        conns.push(conn.clone());
    }
    Ok(conns)
}

pub async fn get_dead_ws_connections() -> anyhow::Result<Vec<Connection>> {
    let mut conns = Vec::new();
    for (_, conn) in CONNECTION_MANAGER.read().await.dead_connections.iter() {
        conns.push(conn.clone());
    }
    Ok(conns)
}

pub async fn refresh_latency() -> anyhow::Result<()> {
    let mut cm = CONNECTION_MANAGER.write().await;
    let mut dead_conns = Vec::new();
    for (_, conn) in cm.connections.iter_mut() {
        let start = std::time::Instant::now();
        let (mut ws, _) = match tokio_tungstenite::connect_async(conn.url.as_str()).await {
            Ok(tup) => tup,
            Err(err) => {
                println!("failed to connect to websocket: {:?}", err);
                dead_conns.push(conn.id.clone());
                continue;
            }
        };
        let _ = ws.close(None).await;
        conn.latency = start.elapsed().as_millis() as u32;
    }
    for id in dead_conns {
        let conn = cm.connections.remove(&id);
        if let Some(conn) = conn {
            cm.dead_connections.insert(id, conn);
        }
    }
    Ok(())
}

async fn proxy_ws_addr(addr: impl AsRef<str>, tcp: TcpStream) -> anyhow::Result<()> {
    let (ws, _) = tokio_tungstenite::connect_async(addr.as_ref()).await?;
    wsrx::proxy_ws(ws, tcp).await?;
    Ok(())
}